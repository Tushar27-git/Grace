use crate::glyphs::GlyphRenderer;
use crate::theme::Theme;
use crate::tools::{ActiveTool, MarqueeState, SelectionState, WireInProgress};
use eframe::egui::{
    self, Color32, CursorIcon, Key, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2,
};
use logic_core::{Circuit, ComponentId, GateKind, NetId, PortEndpoint, Signal, Simulator};

pub struct CanvasState {
    pub pan: Vec2,
    pub zoom: f32,
    pub show_grid: bool,
    pub snap_to_grid: bool,
    #[allow(dead_code)]
    pub active_tool: ActiveTool,
    pub selection: SelectionState,
    pub wire_in_progress: Option<WireInProgress>,
    pub marquee: Option<MarqueeState>,
    pub hovered_component: Option<ComponentId>,
    pub hovered_port: Option<(PortEndpoint, Pos2)>,
    pub hovered_net: Option<NetId>,
    pub drag_start_positions: Vec<(ComponentId, (f32, f32))>,
    pub is_dragging_components: bool,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            pan: Vec2::new(300.0, 200.0),
            zoom: 1.0,
            show_grid: true,
            snap_to_grid: true,
            active_tool: ActiveTool::Select,
            selection: SelectionState::default(),
            wire_in_progress: None,
            marquee: None,
            hovered_component: None,
            hovered_port: None,
            hovered_net: None,
            drag_start_positions: Vec::new(),
            is_dragging_components: false,
        }
    }
}

impl CanvasState {
    pub const GRID_SPACING: f32 = 20.0;

    pub fn screen_to_canvas(&self, screen_pos: Pos2) -> Pos2 {
        Pos2::new(
            (screen_pos.x - self.pan.x) / self.zoom,
            (screen_pos.y - self.pan.y) / self.zoom,
        )
    }

    pub fn canvas_to_screen(&self, canvas_pos: Pos2) -> Pos2 {
        Pos2::new(
            canvas_pos.x * self.zoom + self.pan.x,
            canvas_pos.y * self.zoom + self.pan.y,
        )
    }

    pub fn snap_point(&self, pt: Pos2) -> Pos2 {
        if self.snap_to_grid {
            let sx = (pt.x / Self::GRID_SPACING).round() * Self::GRID_SPACING;
            let sy = (pt.y / Self::GRID_SPACING).round() * Self::GRID_SPACING;
            Pos2::new(sx, sy)
        } else {
            pt
        }
    }
}

pub struct CanvasResponse {
    pub circuit_mutated: bool,
    pub placed_component: bool,
}

impl CanvasState {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        circuit: &mut Circuit,
        selected_for_placement: &mut Option<GateKind>,
    ) -> CanvasResponse {
        let mut response_meta = CanvasResponse {
            circuit_mutated: false,
            placed_component: false,
        };

        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        // 1. Handle Pan and Zoom Input
        self.handle_pan_zoom(ui, &response);

        // 2. Draw Background and Grid
        painter.rect_filled(rect, 0.0, Theme::BG_CANVAS);
        if self.show_grid {
            self.draw_grid(&painter, rect);
        }

        // 3. Find Hovered Elements
        let pointer_pos = response.hover_pos();
        let pointer_canvas = pointer_pos.map(|p| self.screen_to_canvas(p));

        self.update_hover_states(circuit, pointer_canvas, pointer_pos);

        // 4. Handle Canvas Interactions
        self.handle_interactions(
            ui,
            &response,
            circuit,
            selected_for_placement,
            pointer_canvas,
            &mut response_meta,
        );

        // 5. Draw Wires and Nets
        self.draw_wires(&painter, circuit);

        // 6. Draw In-Progress Wire
        if let Some(wip) = &self.wire_in_progress {
            let start = wip.source_pos;
            let end = if let Some((endpoint, port_screen_pos)) = self.hovered_port {
                if !endpoint.is_output {
                    port_screen_pos
                } else {
                    wip.current_cursor
                }
            } else {
                wip.current_cursor
            };
            self.draw_bezier_wire(&painter, start, end, Theme::ACCENT_PINK, 2.0 * self.zoom);
        }

        // 7. Draw Components via Glyphs
        for (id, comp) in &circuit.components {
            let is_sel = self.selection.selected_components.contains(&id);
            let is_hov = self.hovered_component == Some(id);
            GlyphRenderer::draw_component(
                &painter,
                comp,
                is_sel,
                is_hov,
                |p| self.canvas_to_screen(p),
                self.zoom,
            );
        }

        // 8. Draw Ghost Placement Preview
        if let Some(kind) = selected_for_placement.clone()
            && let Some(cpos) = pointer_canvas
        {
            let snapped = self.snap_point(cpos);
            let ghost_node = logic_core::ComponentNode::new(kind, (snapped.x, snapped.y));
            GlyphRenderer::draw_component(
                &painter,
                &ghost_node,
                true,
                false,
                |p| self.canvas_to_screen(p),
                self.zoom,
            );
        }

        // 9. Draw Marquee Selection Box
        if let Some(marquee) = &self.marquee {
            let min = Pos2::new(
                marquee.start.x.min(marquee.current.x),
                marquee.start.y.min(marquee.current.y),
            );
            let max = Pos2::new(
                marquee.start.x.max(marquee.current.x),
                marquee.start.y.max(marquee.current.y),
            );
            let box_rect = Rect::from_min_max(min, max);
            painter.rect_filled(box_rect, 0.0, Theme::ACCENT_PURPLE.gamma_multiply(0.15));
            painter.rect_stroke(
                box_rect,
                0.0,
                Stroke::new(1.0, Theme::ACCENT_PURPLE),
                egui::StrokeKind::Outside,
            );
        }

        // 10. Update cursor icon
        if response.hovered() {
            if selected_for_placement.is_some() {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            } else if self.wire_in_progress.is_some() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            } else if self.hovered_port.is_some() {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            } else if self.hovered_component.is_some() {
                ui.ctx().set_cursor_icon(CursorIcon::Grab);
            }
        }

        response_meta
    }

    fn handle_pan_zoom(&mut self, ui: &Ui, response: &Response) {
        // Middle mouse drag or space+drag for pan
        let space_down = ui.input(|i| i.key_down(Key::Space));
        if (response.dragged_by(egui::PointerButton::Middle))
            || (space_down && response.dragged_by(egui::PointerButton::Primary))
        {
            self.pan += response.drag_delta();
        }

        // Mouse wheel for zoom centered at mouse position
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0
                && let Some(mouse_screen) = response.hover_pos()
            {
                let old_zoom = self.zoom;
                let zoom_factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                let new_zoom = (old_zoom * zoom_factor).clamp(0.25, 4.0);

                // Zoom towards mouse:
                // canvas_pos = (mouse_screen - pan) / old_zoom
                // new_pan = mouse_screen - canvas_pos * new_zoom
                let canvas_pos = (mouse_screen - self.pan) / old_zoom;
                self.pan = mouse_screen - canvas_pos * new_zoom;
                self.zoom = new_zoom;
            }
        }
    }

    fn draw_grid(&self, painter: &Painter, rect: Rect) {
        let step = Self::GRID_SPACING * self.zoom;
        if step < 6.0 {
            return; // Don't draw if too small
        }

        let start_x = (rect.min.x - self.pan.x) % step + rect.min.x;
        let start_y = (rect.min.y - self.pan.y) % step + rect.min.y;

        let dot_radius = (1.0 * self.zoom).clamp(0.6, 1.8);

        let mut y = start_y - step;
        while y <= rect.max.y + step {
            let mut x = start_x - step;
            while x <= rect.max.x + step {
                if rect.contains(Pos2::new(x, y)) {
                    painter.circle_filled(Pos2::new(x, y), dot_radius, Theme::GRID_LINE);
                }
                x += step;
            }
            y += step;
        }
    }

    fn update_hover_states(
        &mut self,
        circuit: &Circuit,
        pointer_canvas: Option<Pos2>,
        pointer_pos: Option<Pos2>,
    ) {
        self.hovered_component = None;
        self.hovered_port = None;
        self.hovered_net = None;

        let (cpos, spos) = match (pointer_canvas, pointer_pos) {
            (Some(c), Some(s)) => (c, s),
            _ => return,
        };

        let port_hit_threshold_screen = 12.0;

        // Check ports first (higher priority than component body)
        for (id, comp) in &circuit.components {
            // Inputs
            for i in 0..comp.input_signals.len() {
                let world_pos = comp.port_world_pos(false, i);
                let screen_pos = self.canvas_to_screen(Pos2::new(world_pos.0, world_pos.1));
                if spos.distance(screen_pos) <= port_hit_threshold_screen {
                    self.hovered_port = Some((
                        PortEndpoint {
                            component_id: id,
                            is_output: false,
                            port_index: i,
                        },
                        screen_pos,
                    ));
                    return;
                }
            }

            // Outputs
            for i in 0..comp.output_signals.len() {
                let world_pos = comp.port_world_pos(true, i);
                let screen_pos = self.canvas_to_screen(Pos2::new(world_pos.0, world_pos.1));
                if spos.distance(screen_pos) <= port_hit_threshold_screen {
                    self.hovered_port = Some((
                        PortEndpoint {
                            component_id: id,
                            is_output: true,
                            port_index: i,
                        },
                        screen_pos,
                    ));
                    return;
                }
            }
        }

        // Check component bounding boxes
        for (id, comp) in &circuit.components {
            let (min, max) = comp.bounding_box();
            if cpos.x >= min.0 && cpos.x <= max.0 && cpos.y >= min.1 && cpos.y <= max.1 {
                self.hovered_component = Some(id);
                return;
            }
        }
    }

    fn handle_interactions(
        &mut self,
        ui: &mut Ui,
        response: &Response,
        circuit: &mut Circuit,
        selected_for_placement: &mut Option<GateKind>,
        pointer_canvas: Option<Pos2>,
        meta: &mut CanvasResponse,
    ) {
        let shift_down = ui.input(|i| i.modifiers.shift);
        let space_down = ui.input(|i| i.key_down(Key::Space));

        if space_down {
            return; // Space is reserved for panning
        }

        // Right-click or ESC cancels placement/wire
        if response.clicked_by(egui::PointerButton::Secondary)
            || ui.input(|i| i.key_pressed(Key::Escape))
        {
            *selected_for_placement = None;
            self.wire_in_progress = None;
            self.marquee = None;
            return;
        }

        // Mode A: Placing a new component
        if let Some(kind) = selected_for_placement.clone() {
            if response.clicked_by(egui::PointerButton::Primary)
                && let Some(cpos) = pointer_canvas
            {
                let snapped = self.snap_point(cpos);
                let id = circuit.add_component(kind, (snapped.x, snapped.y));
                Simulator::settle(circuit);
                self.selection.select_single(id);
                meta.circuit_mutated = true;
                meta.placed_component = true;
                *selected_for_placement = None;
            }
            return;
        }

        // Mode B: Interactive Wire Creation
        if let Some(wip) = &mut self.wire_in_progress {
            if let Some(spos) = response.hover_pos() {
                wip.current_cursor = spos;
            }

            // If released or clicked
            if response.drag_stopped_by(egui::PointerButton::Primary)
                || (response.clicked_by(egui::PointerButton::Primary)
                    && response.hover_pos() != Some(wip.source_pos))
            {
                if let Some((target_endpoint, _)) = self.hovered_port
                    && !target_endpoint.is_output
                {
                    // Connect output to input!
                    if circuit
                        .connect_ports(wip.source_endpoint, target_endpoint)
                        .is_some()
                    {
                        Simulator::settle(circuit);
                        meta.circuit_mutated = true;
                    }
                }
                self.wire_in_progress = None;
            }
            return;
        }

        // Mode C: Normal Tool (Select, Move, Wire Start, Toggle Switch Click)
        if response.drag_started_by(egui::PointerButton::Primary) {
            // Check if dragging started on an output port -> start wire tool
            if let Some((port_endpoint, screen_pos)) = self.hovered_port
                && port_endpoint.is_output
            {
                self.wire_in_progress = Some(WireInProgress {
                    source_endpoint: port_endpoint,
                    source_pos: screen_pos,
                    current_cursor: screen_pos,
                });
                return;
            }

            // Check if dragging started on a component -> move selection
            if let Some(comp_id) = self.hovered_component {
                if !self.selection.selected_components.contains(&comp_id) {
                    if !shift_down {
                        self.selection.clear();
                    }
                    self.selection.selected_components.insert(comp_id);
                }

                self.is_dragging_components = true;
                self.drag_start_positions = self
                    .selection
                    .selected_components
                    .iter()
                    .filter_map(|&id| circuit.components.get(id).map(|c| (id, c.pos)))
                    .collect();
            } else {
                // Dragging started on empty canvas -> start marquee selection
                if let Some(spos) = response.hover_pos() {
                    if !shift_down {
                        self.selection.clear();
                    }
                    self.marquee = Some(MarqueeState {
                        start: spos,
                        current: spos,
                    });
                }
            }
        }

        // Moving components
        if self.is_dragging_components && response.dragged_by(egui::PointerButton::Primary) {
            let delta_screen = response.drag_delta();
            let delta_canvas = delta_screen / self.zoom;
            for &id in &self.selection.selected_components {
                if let Some(comp) = circuit.components.get_mut(id) {
                    comp.pos.0 += delta_canvas.x;
                    comp.pos.1 += delta_canvas.y;
                }
            }
        }

        // Finished dragging components -> snap to grid
        if self.is_dragging_components && response.drag_stopped_by(egui::PointerButton::Primary) {
            self.is_dragging_components = false;
            for &id in &self.selection.selected_components {
                if let Some(comp) = circuit.components.get_mut(id) {
                    let snapped = self.snap_point(Pos2::new(comp.pos.0, comp.pos.1));
                    comp.pos = (snapped.x, snapped.y);
                }
            }
            meta.circuit_mutated = true;
        }

        // Marquee dragging
        if let Some(marquee) = &mut self.marquee {
            if let Some(spos) = response.hover_pos() {
                marquee.current = spos;
            }

            if response.drag_stopped_by(egui::PointerButton::Primary) {
                // Select all components inside marquee box
                let min_s = Pos2::new(
                    marquee.start.x.min(marquee.current.x),
                    marquee.start.y.min(marquee.current.y),
                );
                let max_s = Pos2::new(
                    marquee.start.x.max(marquee.current.x),
                    marquee.start.y.max(marquee.current.y),
                );
                let min_c = self.screen_to_canvas(min_s);
                let max_c = self.screen_to_canvas(max_s);

                for (id, comp) in &circuit.components {
                    if comp.pos.0 >= min_c.x
                        && comp.pos.0 <= max_c.x
                        && comp.pos.1 >= min_c.y
                        && comp.pos.1 <= max_c.y
                    {
                        self.selection.selected_components.insert(id);
                    }
                }
                self.marquee = None;
            }
        }

        // Single Click: Toggle Switch flip or Select component
        if response.clicked_by(egui::PointerButton::Primary) && !self.is_dragging_components {
            if let Some(comp_id) = self.hovered_component {
                if let Some(comp) = circuit.components.get_mut(comp_id)
                    && comp.kind == GateKind::ToggleSwitch
                {
                    // User clicked toggle switch -> flip and simulate!
                    comp.state_flag = !comp.state_flag;
                    Simulator::settle(circuit);
                    meta.circuit_mutated = true;
                    return;
                }

                if shift_down {
                    self.selection.toggle_component(comp_id);
                } else {
                    self.selection.select_single(comp_id);
                }
            } else if !shift_down && self.marquee.is_none() {
                self.selection.clear();
            }
        }
    }

    fn draw_wires(&self, painter: &Painter, circuit: &Circuit) {
        for (net_id, net) in &circuit.nets {
            let src_comp = match circuit.components.get(net.source.component_id) {
                Some(c) => c,
                None => continue,
            };
            let src_world = src_comp.port_world_pos(true, net.source.port_index);
            let src_screen = self.canvas_to_screen(Pos2::new(src_world.0, src_world.1));

            let sig = net.current_signal;
            let wire_color = match sig {
                Signal::Zero => Theme::SIGNAL_LOW,
                Signal::One => Theme::SIGNAL_HIGH,
                Signal::X => Theme::SIGNAL_X,
                Signal::Z => Theme::SIGNAL_LOW.gamma_multiply(0.5),
            };

            let is_net_hovered = self.hovered_net == Some(net_id);
            let is_bus = net
                .label
                .as_deref()
                .map(|l| l.to_uppercase().starts_with("BUS") || l.contains('/'))
                .unwrap_or(false);
            let base_width = if is_bus { 3.6 } else { 1.6 };
            let stroke_width = if is_net_hovered {
                (base_width + 1.4) * self.zoom
            } else if sig.is_high() {
                (base_width + 0.6) * self.zoom
            } else {
                base_width * self.zoom
            };

            for &sink in &net.sinks {
                let sink_comp = match circuit.components.get(sink.component_id) {
                    Some(c) => c,
                    None => continue,
                };
                let sink_world = sink_comp.port_world_pos(false, sink.port_index);
                let sink_screen = self.canvas_to_screen(Pos2::new(sink_world.0, sink_world.1));

                self.draw_bezier_wire(painter, src_screen, sink_screen, wire_color, stroke_width);

                // Draw label badge pill at midpoint if net has a label
                if let Some(ref lbl) = net.label {
                    let mid = Pos2::new(
                        (src_screen.x + sink_screen.x) * 0.5,
                        (src_screen.y + sink_screen.y) * 0.5,
                    );
                    let font_size = (10.0 * self.zoom).clamp(8.0, 13.0);
                    let pill_w = (lbl.len() as f32 * 6.5 + 10.0) * self.zoom.clamp(0.8, 1.2);
                    let pill_h = 14.0 * self.zoom.clamp(0.8, 1.2);
                    let tag_rect = Rect::from_center_size(mid, egui::Vec2::new(pill_w, pill_h));
                    painter.rect_filled(tag_rect, 3.0, Theme::BG_PANEL);
                    painter.rect_stroke(
                        tag_rect,
                        3.0,
                        Stroke::new(1.0, wire_color),
                        egui::StrokeKind::Inside,
                    );
                    painter.text(
                        mid,
                        egui::Align2::CENTER_CENTER,
                        lbl,
                        egui::FontId::monospace(font_size),
                        Theme::TEXT_PRIMARY,
                    );
                }
            }

            // Draw junction dot at source if multiple sinks
            if net.sinks.len() > 1 {
                let dot_r = if is_bus { 4.5 } else { 3.5 };
                painter.circle_filled(src_screen, dot_r * self.zoom, wire_color);
            }
        }
    }

    fn draw_bezier_wire(&self, painter: &Painter, p0: Pos2, p3: Pos2, color: Color32, width: f32) {
        let dx = (p3.x - p0.x).abs().max(40.0 * self.zoom);
        let p1 = Pos2::new(p0.x + dx * 0.5, p0.y);
        let p2 = Pos2::new(p3.x - dx * 0.5, p3.y);

        let segments = 24;
        let mut pts = Vec::with_capacity(segments + 1);
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let it = 1.0 - t;
            let x = it * it * it * p0.x
                + 3.0 * it * it * t * p1.x
                + 3.0 * it * t * t * p2.x
                + t * t * t * p3.x;
            let y = it * it * it * p0.y
                + 3.0 * it * it * t * p1.y
                + 3.0 * it * t * t * p2.y
                + t * t * t * p3.y;
            pts.push(Pos2::new(x, y));
        }

        for w in pts.windows(2) {
            painter.line_segment([w[0], w[1]], Stroke::new(width, color));
        }
    }
}

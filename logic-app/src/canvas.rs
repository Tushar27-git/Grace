use crate::glyphs::GlyphRenderer;
use crate::theme::{Theme, ThemeMode};
use crate::tools::{ActiveTool, MarqueeState, SelectionState, WireInProgress};
use eframe::egui::{
    self, Color32, CornerRadius, CursorIcon, Key, Painter, Pos2, Rect, Response, RichText, Sense,
    Stroke, Ui, Vec2, pos2,
};
use logic_core::{Circuit, ComponentId, GateKind, NetId, PortEndpoint, Signal, Simulator};

pub struct CanvasState {
    pub pan: Vec2,
    pub zoom: f32,
    pub show_grid: bool,
    pub snap_to_grid: bool,
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
            active_tool: ActiveTool::Normal,
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
        theme_mode: ThemeMode,
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
        painter.rect_filled(rect, 0.0, theme_mode.bg_canvas());
        if self.show_grid {
            self.draw_grid(&painter, rect, theme_mode);
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
        self.draw_wires(&painter, circuit, theme_mode);

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
                theme_mode,
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
                theme_mode,
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

        // 10. Draw glowing snap indicator for hovered port
        if let Some((_endpoint, port_screen_pos)) = self.hovered_port {
            let snap_r = match self.active_tool {
                ActiveTool::Connect => 7.5,
                _ => 6.0,
            };
            painter.circle_stroke(
                port_screen_pos,
                snap_r,
                Stroke::new(2.0, Theme::ACCENT_PINK),
            );
            painter.circle_filled(
                port_screen_pos,
                (snap_r - 2.5).max(1.0),
                Theme::ACCENT_PINK.gamma_multiply(0.4),
            );
        }

        // 11. Update cursor icon based on active tool
        if response.hovered() {
            if selected_for_placement.is_some() {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            } else if self.active_tool == ActiveTool::Pan {
                if response.dragged() {
                    ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                } else {
                    ui.ctx().set_cursor_icon(CursorIcon::Grab);
                }
            } else if self.active_tool == ActiveTool::Connect {
                if self.hovered_port.is_some() || self.wire_in_progress.is_some() {
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                } else {
                    ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                }
            } else if self.active_tool == ActiveTool::Marquee {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            } else if self.wire_in_progress.is_some() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            } else if self.hovered_port.is_some() {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            } else if self.hovered_component.is_some() {
                ui.ctx().set_cursor_icon(CursorIcon::Grab);
            }
        }

        // 12. Floating Right Toolbar Dock (Tools & Zoom Slider)
        self.show_right_toolbar(ui.ctx(), rect);

        response_meta
    }

    fn handle_pan_zoom(&mut self, ui: &Ui, response: &Response) {
        // Middle mouse drag, space+drag, or Pan Tool drag
        let space_down = ui.input(|i| i.key_down(Key::Space));
        let pan_tool_drag = (self.active_tool == ActiveTool::Pan)
            && response.dragged_by(egui::PointerButton::Primary);

        if (response.dragged_by(egui::PointerButton::Middle))
            || (space_down && response.dragged_by(egui::PointerButton::Primary))
            || pan_tool_drag
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

    fn draw_grid(&self, painter: &Painter, rect: Rect, theme_mode: ThemeMode) {
        let step = Self::GRID_SPACING * self.zoom;
        if step < 5.0 {
            return;
        }

        // Adaptive stride so dots remain crisp without moire or lag when zoomed out
        let stride = if self.zoom < 0.35 {
            4
        } else if self.zoom < 0.7 {
            2
        } else {
            1
        };

        let spacing = Self::GRID_SPACING * stride as f32;

        // Map visible screen rect to canvas coordinate space
        let c_min = self.screen_to_canvas(rect.min);
        let c_max = self.screen_to_canvas(rect.max);

        let min_x = c_min.x.min(c_max.x);
        let max_x = c_min.x.max(c_max.x);
        let min_y = c_min.y.min(c_max.y);
        let max_y = c_min.y.max(c_max.y);

        let start_col = (min_x / spacing).floor() as i32;
        let end_col = (max_x / spacing).ceil() as i32;
        let start_row = (min_y / spacing).floor() as i32;
        let end_row = (max_y / spacing).ceil() as i32;

        let base_radius = (1.3 * self.zoom * stride as f32).clamp(1.0, 2.5);
        let grid_color = theme_mode.grid_line();
        let major_color = if theme_mode.is_dark() {
            Theme::ACCENT_PURPLE.gamma_multiply(0.60)
        } else {
            Color32::from_rgb(0x8C, 0x80, 0x70).gamma_multiply(0.75)
        };

        for col in start_col..=end_col {
            let canvas_x = col as f32 * spacing;
            let is_major_x = (col * stride) % 5 == 0;
            for row in start_row..=end_row {
                let canvas_y = row as f32 * spacing;
                let is_major = is_major_x && (row * stride) % 5 == 0;

                let screen_pos = self.canvas_to_screen(Pos2::new(canvas_x, canvas_y));
                if rect.contains(screen_pos) {
                    let r = if is_major { base_radius + 0.5 } else { base_radius };
                    let color = if is_major { major_color } else { grid_color };
                    painter.circle_filled(screen_pos, r, color);
                }
            }
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

        let base_threshold = 12.0;
        let port_hit_threshold_screen = match self.active_tool {
            ActiveTool::Connect => base_threshold * 1.05, // +5% in connection mode: 12.60px
            ActiveTool::Normal => base_threshold * 1.02,  // +2% in normal view mode: 12.24px
            ActiveTool::Marquee => 0.0,                   // Marquee only selects components
            ActiveTool::Pan => 0.0,                       // Pan only moves canvas
        };

        if port_hit_threshold_screen > 0.0 {
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
        }

        // Check component bounding boxes (for hover)
        if self.active_tool != ActiveTool::Pan {
            for (id, comp) in &circuit.components {
                let (min, max) = comp.bounding_box();
                if cpos.x >= min.0 && cpos.x <= max.0 && cpos.y >= min.1 && cpos.y <= max.1 {
                    self.hovered_component = Some(id);
                    return;
                }
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

        if space_down || self.active_tool == ActiveTool::Pan {
            return; // Space or Pan tool is reserved for panning
        }

        // Right-click or ESC cancels placement/wire/marquee
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

        // Mode B: Interactive Wire Creation (Connection Mode or Normal Mode wire drag)
        if let Some(wip) = &mut self.wire_in_progress {
            if let Some(spos) = response.hover_pos() {
                wip.current_cursor = spos;
            }

            // If released or clicked
            if response.drag_stopped_by(egui::PointerButton::Primary)
                || (response.clicked_by(egui::PointerButton::Primary)
                    && response.hover_pos() != Some(wip.source_pos))
            {
                if let Some((target_endpoint, _)) = self.hovered_port {
                    // Check if one is output and one is input
                    if wip.source_endpoint.is_output != target_endpoint.is_output {
                        let (src, sink) = if wip.source_endpoint.is_output {
                            (wip.source_endpoint, target_endpoint)
                        } else {
                            (target_endpoint, wip.source_endpoint)
                        };

                        if circuit.connect_ports(src, sink).is_some() {
                            Simulator::settle(circuit);
                            meta.circuit_mutated = true;
                        }
                    }
                }
                self.wire_in_progress = None;
            }
            return;
        }

        // Tool 1: Marquee / Selection Tool ("only select the component")
        if self.active_tool == ActiveTool::Marquee {
            if response.drag_started_by(egui::PointerButton::Primary) {
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

            if let Some(marquee) = &mut self.marquee {
                if let Some(spos) = response.hover_pos() {
                    marquee.current = spos;
                }

                if response.drag_stopped_by(egui::PointerButton::Primary) {
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

            if response.clicked_by(egui::PointerButton::Primary) && self.marquee.is_none() {
                if let Some(comp_id) = self.hovered_component {
                    if shift_down {
                        self.selection.toggle_component(comp_id);
                    } else {
                        self.selection.select_single(comp_id);
                    }
                } else if !shift_down {
                    self.selection.clear();
                }
            }
            return;
        }

        // Tool 2: Connection Tool (dedicated wiring with +5% snap radius)
        if self.active_tool == ActiveTool::Connect {
            if response.drag_started_by(egui::PointerButton::Primary)
                || response.clicked_by(egui::PointerButton::Primary)
            {
                if let Some((port_endpoint, screen_pos)) = self.hovered_port {
                    self.wire_in_progress = Some(WireInProgress {
                        source_endpoint: port_endpoint,
                        source_pos: screen_pos,
                        current_cursor: screen_pos,
                    });
                }
            }
            return;
        }

        // Tool 4: Normal Mode (Standard pointer & editing)
        // Dragging started
        if response.drag_started_by(egui::PointerButton::Primary) {
            // Check if dragging started on an output port -> start wire
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

        // Moving components (Normal mode only)
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

        // Marquee dragging (Normal mode)
        if let Some(marquee) = &mut self.marquee {
            if let Some(spos) = response.hover_pos() {
                marquee.current = spos;
            }

            if response.drag_stopped_by(egui::PointerButton::Primary) {
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

        // Single Click: Toggle Switch flip or Select component (Normal mode)
        if response.clicked_by(egui::PointerButton::Primary) && !self.is_dragging_components {
            if let Some(comp_id) = self.hovered_component {
                if let Some(comp) = circuit.components.get_mut(comp_id)
                    && (comp.kind == GateKind::ToggleSwitch || comp.kind == GateKind::BitSwitch)
                {
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

    fn show_right_toolbar(&mut self, ctx: &egui::Context, rect: Rect) {
        let mut dock_frame = Theme::glass_modal();
        dock_frame.inner_margin = egui::Margin::symmetric(6, 8);
        dock_frame.corner_radius = egui::CornerRadius::same(8);

        egui::Window::new("canvas_right_dock")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .frame(dock_frame)
            .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(-16.0, 20.0))
            .show(ctx, |ui| {
                ui.set_width(42.0);
                ui.vertical_centered(|ui| {
                    // Tool Buttons
                    let tools = [
                        (
                            ActiveTool::Normal,
                            "Normal Mode (1 / V)\nSelect, move, switch toggle & wire.\nPort snap radius +2%",
                        ),
                        (
                            ActiveTool::Marquee,
                            "Marquee Selection Tool (2 / M)\nDrag on canvas to box-select components only",
                        ),
                        (
                            ActiveTool::Connect,
                            "Connection Tool (3 / C)\nDedicated wire creation tool.\nPort snap radius +5%",
                        ),
                        (
                            ActiveTool::Pan,
                            "Pan / Move Canvas (4 / H)\nClick and drag anywhere to move view",
                        ),
                    ];

                    for (tool, tooltip) in tools {
                        let is_active = self.active_tool == tool;
                        let (btn_rect, btn_resp) =
                            ui.allocate_exact_size(Vec2::new(36.0, 32.0), Sense::click());

                        let bg_color = if is_active {
                            Theme::ACCENT_PURPLE.gamma_multiply(0.45)
                        } else if btn_resp.hovered() {
                            Theme::BG_PANEL_RAISED
                        } else {
                            Theme::BG_PANEL
                        };

                        let stroke_color = if is_active {
                            Theme::ACCENT_PINK
                        } else if btn_resp.hovered() {
                            Theme::ACCENT_PINK.gamma_multiply(0.7)
                        } else {
                            Theme::ACCENT_PURPLE.gamma_multiply(0.4)
                        };

                        let stroke_width = if is_active { 1.5 } else { 1.0 };
                        ui.painter().rect(
                            btn_rect,
                            CornerRadius::same(5),
                            bg_color,
                            Stroke::new(stroke_width, stroke_color),
                            egui::StrokeKind::Inside,
                        );

                        let icon_color = if is_active {
                            Theme::ACCENT_PINK
                        } else if btn_resp.hovered() {
                            Color32::WHITE
                        } else {
                            Theme::TEXT_PRIMARY
                        };

                        let c = btn_rect.center();
                        let p = ui.painter();

                        match tool {
                            ActiveTool::Normal => {
                                // Pointer / Cursor arrow pointing northwest
                                let tip = pos2(c.x - 5.0, c.y - 6.0);
                                let p1 = pos2(c.x - 5.0, c.y + 6.0);
                                let p2 = pos2(c.x - 1.5, c.y + 2.5);
                                let p3 = pos2(c.x + 3.5, c.y + 7.5);
                                let p4 = pos2(c.x + 5.5, c.y + 5.5);
                                let p5 = pos2(c.x + 0.5, c.y + 0.5);
                                let p6 = pos2(c.x + 5.0, c.y - 0.5);
                                p.add(egui::epaint::PathShape::convex_polygon(
                                    vec![tip, p1, p2, p3, p4, p5, p6],
                                    icon_color.gamma_multiply(0.3),
                                    Stroke::new(1.3, icon_color),
                                ));
                            }
                            ActiveTool::Marquee => {
                                // Box selection marquee icon: 4 corner brackets + center dot
                                let hw = 7.0;
                                let hh = 6.0;
                                let b = 3.5;
                                let s = Stroke::new(1.4, icon_color);
                                // Top-left
                                p.line_segment(
                                    [pos2(c.x - hw, c.y - hh + b), pos2(c.x - hw, c.y - hh)],
                                    s,
                                );
                                p.line_segment(
                                    [pos2(c.x - hw, c.y - hh), pos2(c.x - hw + b, c.y - hh)],
                                    s,
                                );
                                // Top-right
                                p.line_segment(
                                    [pos2(c.x + hw - b, c.y - hh), pos2(c.x + hw, c.y - hh)],
                                    s,
                                );
                                p.line_segment(
                                    [pos2(c.x + hw, c.y - hh), pos2(c.x + hw, c.y - hh + b)],
                                    s,
                                );
                                // Bottom-right
                                p.line_segment(
                                    [pos2(c.x + hw, c.y + hh - b), pos2(c.x + hw, c.y + hh)],
                                    s,
                                );
                                p.line_segment(
                                    [pos2(c.x + hw, c.y + hh), pos2(c.x + hw - b, c.y + hh)],
                                    s,
                                );
                                // Bottom-left
                                p.line_segment(
                                    [pos2(c.x - hw + b, c.y + hh), pos2(c.x - hw, c.y + hh)],
                                    s,
                                );
                                p.line_segment(
                                    [pos2(c.x - hw, c.y + hh), pos2(c.x - hw, c.y + hh - b)],
                                    s,
                                );
                                // Center alignment dot
                                p.circle_filled(c, 1.3, icon_color.gamma_multiply(0.8));
                            }
                            ActiveTool::Connect => {
                                // Circuit wiring connection icon: two port terminals with an orthogonal interconnect wire
                                let p_left = pos2(c.x - 7.0, c.y + 4.0);
                                let p_right = pos2(c.x + 7.0, c.y - 4.0);
                                let s = Stroke::new(1.5, icon_color);
                                // Stepped wire
                                p.line_segment([p_left, pos2(c.x, c.y + 4.0)], s);
                                p.line_segment([pos2(c.x, c.y + 4.0), pos2(c.x, c.y - 4.0)], s);
                                p.line_segment([pos2(c.x, c.y - 4.0), p_right], s);
                                // Terminal circular ports
                                p.circle_filled(p_left, 2.5, icon_color);
                                p.circle_filled(p_right, 2.5, icon_color);
                                // Central wire junction dot
                                p.circle_filled(pos2(c.x, c.y), 1.6, icon_color);
                            }
                            ActiveTool::Pan => {
                                // 4-way pan / move cross navigation arrows
                                let len = 7.0;
                                let s = Stroke::new(1.4, icon_color);
                                p.line_segment([pos2(c.x - len, c.y), pos2(c.x + len, c.y)], s);
                                p.line_segment([pos2(c.x, c.y - len), pos2(c.x, c.y + len)], s);
                                // Left arrowhead
                                p.line_segment(
                                    [pos2(c.x - len + 2.5, c.y - 2.5), pos2(c.x - len, c.y)],
                                    s,
                                );
                                p.line_segment(
                                    [pos2(c.x - len + 2.5, c.y + 2.5), pos2(c.x - len, c.y)],
                                    s,
                                );
                                // Right arrowhead
                                p.line_segment(
                                    [pos2(c.x + len - 2.5, c.y - 2.5), pos2(c.x + len, c.y)],
                                    s,
                                );
                                p.line_segment(
                                    [pos2(c.x + len - 2.5, c.y + 2.5), pos2(c.x + len, c.y)],
                                    s,
                                );
                                // Up arrowhead
                                p.line_segment(
                                    [pos2(c.x - 2.5, c.y - len + 2.5), pos2(c.x, c.y - len)],
                                    s,
                                );
                                p.line_segment(
                                    [pos2(c.x + 2.5, c.y - len + 2.5), pos2(c.x, c.y - len)],
                                    s,
                                );
                                // Down arrowhead
                                p.line_segment(
                                    [pos2(c.x - 2.5, c.y + len - 2.5), pos2(c.x, c.y + len)],
                                    s,
                                );
                                p.line_segment(
                                    [pos2(c.x + 2.5, c.y + len - 2.5), pos2(c.x, c.y + len)],
                                    s,
                                );
                            }
                        }

                        if btn_resp.on_hover_text(tooltip).clicked() {
                            self.active_tool = tool;
                            if tool != ActiveTool::Connect {
                                self.wire_in_progress = None;
                            }
                            if tool != ActiveTool::Marquee {
                                self.marquee = None;
                            }
                        }
                        ui.add_space(2.0);
                    }

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Zoom Controls
                    // [+] button
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("+")
                                    .font(Theme::font_bold(14.0))
                                    .color(Theme::TEXT_PRIMARY),
                            )
                            .min_size(Vec2::new(36.0, 24.0)),
                        )
                        .on_hover_text("Zoom In (+10%)")
                        .clicked()
                    {
                        let old_zoom = self.zoom;
                        let new_zoom = (old_zoom * 1.1).clamp(0.25, 4.0);
                        let center = rect.center();
                        let canvas_center = (center - self.pan) / old_zoom;
                        self.pan = center - canvas_center * new_zoom;
                        self.zoom = new_zoom;
                    }

                    ui.add_space(2.0);

                    // Vertical Zoom Slider
                    let old_zoom = self.zoom;
                    let mut current_zoom = self.zoom;
                    let slider = egui::Slider::new(&mut current_zoom, 0.25..=4.0)
                        .vertical()
                        .show_value(false);

                    let slider_resp = ui.add_sized(Vec2::new(36.0, 70.0), slider);
                    if slider_resp.changed() {
                        let center = rect.center();
                        let canvas_center = (center - self.pan) / old_zoom;
                        self.pan = center - canvas_center * current_zoom;
                        self.zoom = current_zoom;
                    }
                    slider_resp.on_hover_text(format!("Zoom: {:.0}%", self.zoom * 100.0));

                    ui.add_space(2.0);

                    // [-] button
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("-")
                                    .font(Theme::font_bold(14.0))
                                    .color(Theme::TEXT_PRIMARY),
                            )
                            .min_size(Vec2::new(36.0, 24.0)),
                        )
                        .on_hover_text("Zoom Out (-10%)")
                        .clicked()
                    {
                        let old_zoom = self.zoom;
                        let new_zoom = (old_zoom * 0.9).clamp(0.25, 4.0);
                        let center = rect.center();
                        let canvas_center = (center - self.pan) / old_zoom;
                        self.pan = center - canvas_center * new_zoom;
                        self.zoom = new_zoom;
                    }

                    ui.add_space(2.0);

                    // Percentage Reset button
                    let zoom_pct = format!("{:.0}%", self.zoom * 100.0);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(zoom_pct)
                                    .font(Theme::font_bold(10.0))
                                    .color(Theme::ACCENT_PINK),
                            )
                            .min_size(Vec2::new(36.0, 20.0)),
                        )
                        .on_hover_text("Click to reset zoom to 100%")
                        .clicked()
                    {
                        let old_zoom = self.zoom;
                        let new_zoom = 1.0;
                        let center = rect.center();
                        let canvas_center = (center - self.pan) / old_zoom;
                        self.pan = center - canvas_center * new_zoom;
                        self.zoom = new_zoom;
                    }
                });
            });
    }

    fn draw_wires(&self, painter: &Painter, circuit: &Circuit, theme_mode: ThemeMode) {
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
                    painter.rect_filled(tag_rect, 3.0, theme_mode.gate_fill());
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
                        Theme::font_bold(font_size),
                        theme_mode.text_on_canvas(),
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

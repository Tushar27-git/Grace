use crate::glyphs::GlyphRenderer;
use crate::theme::{Theme, ThemeMode};
use crate::tools::{ActiveTool, MarqueeState, SelectionState, WireDropSearchState, WireInProgress};
use eframe::egui::{
    self, Color32, CornerRadius, CursorIcon, Key, Painter, Pos2, Rect, Response, Sense, Stroke,
    Ui, Vec2, pos2,
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
    pub wire_drop_search: Option<WireDropSearchState>,
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
            wire_drop_search: None,
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
    pub trigger_undo: bool,
    pub trigger_redo: bool,
    pub trigger_delete: bool,
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
            trigger_undo: false,
            trigger_redo: false,
            trigger_delete: false,
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

        // 6. Draw In-Progress Wire or Wire-Drop live wire
        if let Some(wip) = &self.wire_in_progress {
            let start = wip.source_pos;
            let end = if let Some((endpoint, port_screen_pos)) = self.hovered_port {
                if endpoint.is_output != wip.source_endpoint.is_output
                    && endpoint.component_id != wip.source_endpoint.component_id
                {
                    port_screen_pos
                } else {
                    wip.current_cursor
                }
            } else {
                wip.current_cursor
            };
            let wire_w = (2.5 * self.zoom).clamp(2.0, 4.0);
            self.draw_bezier_wire(&painter, start, end, Theme::ACCENT_PINK, wire_w);
            // Source terminal dot
            painter.circle_filled(start, 4.5, Theme::ACCENT_PINK);
            // Floating or snapped tip dot
            painter.circle_filled(end, 4.5, Theme::ACCENT_PINK);

            // If snapped to a valid target port, draw a bright glowing halo!
            if end != wip.current_cursor {
                painter.circle_stroke(end, 9.0, Stroke::new(2.5, Color32::WHITE));
                painter.circle_filled(
                    end,
                    7.0,
                    Theme::ACCENT_PINK.gamma_multiply(0.45),
                );
            }
        } else if let Some(search) = &self.wire_drop_search {
            // Live wire leading to Blender-style quick-add drop target
            let start_canvas = circuit
                .components
                .get(search.source_endpoint.component_id)
                .map(|comp| {
                    let wpos = comp.port_world_pos(
                        search.source_endpoint.is_output,
                        search.source_endpoint.port_index,
                    );
                    Pos2::new(wpos.0, wpos.1)
                })
                .unwrap_or(search.source_canvas_pos);
            let start = self.canvas_to_screen(start_canvas);
            let end = self.canvas_to_screen(search.drop_canvas_pos);
            let wire_w = (2.5 * self.zoom).clamp(2.0, 4.0);
            self.draw_bezier_wire(&painter, start, end, Theme::ACCENT_PINK, wire_w);
            // Source terminal dot
            painter.circle_filled(start, 4.5, Theme::ACCENT_PINK);
            // Target drop halo and point
            painter.circle_stroke(end, 9.0, Stroke::new(2.0, Color32::WHITE));
            painter.circle_filled(end, 7.0, Theme::ACCENT_PINK.gamma_multiply(0.5));
            painter.circle_filled(end, 4.5, Color32::WHITE);
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
            let is_wiring = self.wire_in_progress.is_some();
            let snap_r = if is_wiring { 9.0 } else { 7.5 };
            painter.circle_stroke(
                port_screen_pos,
                snap_r,
                Stroke::new(2.0, if is_wiring { Color32::WHITE } else { Theme::ACCENT_PINK }),
            );
            painter.circle_filled(
                port_screen_pos,
                (snap_r - 2.5).max(1.0),
                Theme::ACCENT_PINK.gamma_multiply(0.45),
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

        // 12. Floating Right Toolbar Dock (Tools & Zoom Slider & Actions)
        self.show_right_toolbar(ui.ctx(), rect, &mut response_meta);

        // 13. Blender-style Quick-Add Search Menu when wire dropped in air
        self.show_wire_drop_search_menu(ui.ctx(), rect, circuit, &mut response_meta);

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

        // Generous magnetic snap radius:
        // When wiring is in progress: 26.0 screen px (easy single-go locking)
        // In Connect tool: 24.0 screen px
        // In Normal tool: 20.0 screen px
        let port_hit_threshold_screen = if self.wire_in_progress.is_some() {
            26.0
        } else {
            match self.active_tool {
                ActiveTool::Connect => 24.0,
                ActiveTool::Normal => 20.0,
                ActiveTool::Marquee => 0.0,
                ActiveTool::Pan => 0.0,
            }
        };

        if port_hit_threshold_screen > 0.0 {
            // Find the CLOSEST valid port within the magnetic radius
            let mut closest_port: Option<(PortEndpoint, Pos2, f32)> = None;

            for (id, comp) in &circuit.components {
                // Inputs
                for i in 0..comp.input_signals.len() {
                    let ep = PortEndpoint {
                        component_id: id,
                        is_output: false,
                        port_index: i,
                    };
                    // When wire is in progress, only snap to opposite ports on different components
                    if let Some(wip) = &self.wire_in_progress {
                        if ep.is_output == wip.source_endpoint.is_output
                            || ep.component_id == wip.source_endpoint.component_id
                        {
                            continue;
                        }
                    }

                    let world_pos = comp.port_world_pos(false, i);
                    let screen_pos = self.canvas_to_screen(Pos2::new(world_pos.0, world_pos.1));
                    let d = spos.distance(screen_pos);
                    if d <= port_hit_threshold_screen {
                        let best_d = closest_port
                            .as_ref()
                            .map(|(_, _, cd)| *cd)
                            .unwrap_or(f32::INFINITY);
                        if d < best_d {
                            closest_port = Some((ep, screen_pos, d));
                        }
                    }
                }

                // Outputs
                for i in 0..comp.output_signals.len() {
                    let ep = PortEndpoint {
                        component_id: id,
                        is_output: true,
                        port_index: i,
                    };
                    // When wire is in progress, only snap to opposite ports on different components
                    if let Some(wip) = &self.wire_in_progress {
                        if ep.is_output == wip.source_endpoint.is_output
                            || ep.component_id == wip.source_endpoint.component_id
                        {
                            continue;
                        }
                    }

                    let world_pos = comp.port_world_pos(true, i);
                    let screen_pos = self.canvas_to_screen(Pos2::new(world_pos.0, world_pos.1));
                    let d = spos.distance(screen_pos);
                    if d <= port_hit_threshold_screen {
                        let best_d = closest_port
                            .as_ref()
                            .map(|(_, _, cd)| *cd)
                            .unwrap_or(f32::INFINITY);
                        if d < best_d {
                            closest_port = Some((ep, screen_pos, d));
                        }
                    }
                }
            }

            if let Some((ep, sp, _)) = closest_port {
                self.hovered_port = Some((ep, sp));
                return;
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

    fn find_closest_compatible_port(
        circuit: &Circuit,
        canvas_to_screen: impl Fn(Pos2) -> Pos2,
        source: PortEndpoint,
        cursor_screen: Pos2,
        max_dist: f32,
    ) -> Option<PortEndpoint> {
        let mut best: Option<(PortEndpoint, f32)> = None;
        let look_for_output = !source.is_output;

        for (id, comp) in &circuit.components {
            if id == source.component_id {
                continue;
            }
            if look_for_output {
                for i in 0..comp.output_signals.len() {
                    let wpos = comp.port_world_pos(true, i);
                    let spos = canvas_to_screen(Pos2::new(wpos.0, wpos.1));
                    let d = cursor_screen.distance(spos);
                    if d <= max_dist && d < best.as_ref().map(|b| b.1).unwrap_or(f32::INFINITY) {
                        best = Some((
                            PortEndpoint {
                                component_id: id,
                                is_output: true,
                                port_index: i,
                            },
                            d,
                        ));
                    }
                }
            } else {
                for i in 0..comp.input_signals.len() {
                    let wpos = comp.port_world_pos(false, i);
                    let spos = canvas_to_screen(Pos2::new(wpos.0, wpos.1));
                    let d = cursor_screen.distance(spos);
                    if d <= max_dist && d < best.as_ref().map(|b| b.1).unwrap_or(f32::INFINITY) {
                        best = Some((
                            PortEndpoint {
                                component_id: id,
                                is_output: false,
                                port_index: i,
                            },
                            d,
                        ));
                    }
                }
            }
        }
        best.map(|b| b.0)
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
            self.wire_drop_search = None;
            self.marquee = None;
            return;
        }

        // If wire-drop quick-add search is currently active, suspend background canvas interactions
        if self.wire_drop_search.is_some() {
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

            // If released or clicked (with movement)
            let mouse_released = response.drag_stopped_by(egui::PointerButton::Primary);
            let mouse_clicked = response.clicked_by(egui::PointerButton::Primary)
                && wip.current_cursor.distance(wip.source_pos) > 6.0;

            if mouse_released || mouse_clicked {
                let source_endpoint = wip.source_endpoint;
                let current_cursor = wip.current_cursor;
                let source_pos = wip.source_pos;
                self.wire_in_progress = None; // Reset before querying self

                // Find target port: from self.hovered_port, or fallback search for closest compatible port to cursor
                let target = self.hovered_port.map(|(ep, _)| ep).or_else(|| {
                    Self::find_closest_compatible_port(
                        circuit,
                        |p| self.canvas_to_screen(p),
                        source_endpoint,
                        current_cursor,
                        28.0, // Generous release tolerance so fast drags connect in a single go
                    )
                });

                if let Some(target_endpoint) = target {
                    // Check if one is output and one is input on different components
                    if source_endpoint.is_output != target_endpoint.is_output
                        && source_endpoint.component_id != target_endpoint.component_id
                    {
                        let (src, sink) = if source_endpoint.is_output {
                            (source_endpoint, target_endpoint)
                        } else {
                            (target_endpoint, source_endpoint)
                        };

                        if circuit.connect_ports(src, sink).is_some() {
                            Simulator::settle(circuit);
                            meta.circuit_mutated = true;
                        }
                    }
                } else if current_cursor.distance(source_pos) > 15.0 {
                    // Blender-style quick-add search: wire released in air!
                    let drop_canvas = self.screen_to_canvas(current_cursor);
                    let source_canvas = self.screen_to_canvas(source_pos);
                    self.wire_drop_search = Some(WireDropSearchState {
                        source_endpoint,
                        source_canvas_pos: source_canvas,
                        drop_canvas_pos: drop_canvas,
                        search_query: String::new(),
                        selected_index: 0,
                        request_focus: true,
                    });
                }
                return;
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
            // Check if dragging started on ANY port (input OR output) -> start wire immediately
            if let Some((port_endpoint, screen_pos)) = self.hovered_port {
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

    fn show_right_toolbar(
        &mut self,
        ctx: &egui::Context,
        rect: Rect,
        meta: &mut CanvasResponse,
    ) {
        let mut dock_frame = Theme::glass_modal();
        dock_frame.inner_margin = egui::Margin::symmetric(4, 6);
        dock_frame.corner_radius = egui::CornerRadius::same(8);

        egui::Window::new("canvas_right_dock")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .frame(dock_frame)
            .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(-14.0, 10.0))
            .show(ctx, |ui| {
                ui.set_width(40.0);
                ui.set_min_width(40.0);
                ui.set_max_width(40.0);
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

                    ui.add_space(3.0);
                    ui.separator();
                    ui.add_space(3.0);

                    // Zoom Controls
                    // [+] button
                    let (plus_rect, plus_resp) =
                        ui.allocate_exact_size(Vec2::new(36.0, 22.0), Sense::click());
                    let plus_hov = plus_resp.hovered();
                    ui.painter().rect(
                        plus_rect,
                        CornerRadius::same(4),
                        if plus_hov {
                            Theme::BG_PANEL_RAISED
                        } else {
                            Theme::BG_PANEL
                        },
                        Stroke::new(
                            1.0,
                            if plus_hov {
                                Theme::ACCENT_PINK
                            } else {
                                Theme::ACCENT_PURPLE.gamma_multiply(0.4)
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        plus_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "+",
                        Theme::font_bold(14.0),
                        if plus_hov {
                            Color32::WHITE
                        } else {
                            Theme::TEXT_PRIMARY
                        },
                    );
                    if plus_resp.on_hover_text("Zoom In (+10%)").clicked() {
                        let old_zoom = self.zoom;
                        let new_zoom = (old_zoom * 1.1).clamp(0.25, 4.0);
                        let center = rect.center();
                        let canvas_center = (center - self.pan) / old_zoom;
                        self.pan = center - canvas_center * new_zoom;
                        self.zoom = new_zoom;
                    }

                    ui.add_space(2.0);

                    // Vertical Zoom Slider (100% horizontally centered, precision cyber fader)
                    let (slider_rect, slider_resp) =
                        ui.allocate_exact_size(Vec2::new(36.0, 72.0), Sense::click_and_drag());
                    let slider_hov = slider_resp.hovered();
                    let slider_drag = slider_resp.dragged();

                    let cx = slider_rect.center().x;
                    let y_top = slider_rect.min.y + 7.0;
                    let y_bot = slider_rect.max.y - 7.0;

                    // Track background: perfectly centered at cx
                    let track_w = 6.0;
                    let track_rect = Rect::from_min_max(
                        pos2(cx - track_w * 0.5, y_top),
                        pos2(cx + track_w * 0.5, y_bot),
                    );
                    ui.painter().rect_filled(
                        track_rect,
                        CornerRadius::same(3),
                        Theme::BG_CANVAS_DARK,
                    );
                    ui.painter().rect_stroke(
                        track_rect,
                        CornerRadius::same(3),
                        Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.6)),
                        egui::StrokeKind::Inside,
                    );

                    // Interaction
                    let old_zoom = self.zoom;
                    if slider_resp.dragged() || slider_resp.clicked() {
                        if let Some(pos) = slider_resp.interact_pointer_pos() {
                            let t = ((y_bot - pos.y) / (y_bot - y_top)).clamp(0.0, 1.0);
                            let new_zoom = 0.25 + t * (4.0 - 0.25);
                            let center = rect.center();
                            let canvas_center = (center - self.pan) / old_zoom;
                            self.pan = center - canvas_center * new_zoom;
                            self.zoom = new_zoom;
                        }
                    }

                    // Normalized value t: 0.0 at bottom (0.25x), 1.0 at top (4.0x)
                    let t = ((self.zoom - 0.25) / (4.0 - 0.25)).clamp(0.0, 1.0);
                    let thumb_y = y_bot - t * (y_bot - y_top);

                    // Active filled lower track (from bottom up to thumb)
                    let active_track_rect = Rect::from_min_max(
                        pos2(cx - track_w * 0.5, thumb_y),
                        pos2(cx + track_w * 0.5, y_bot),
                    );
                    ui.painter().rect_filled(
                        active_track_rect,
                        CornerRadius::same(3),
                        Theme::ACCENT_PINK.gamma_multiply(0.4),
                    );

                    // Thumb Handle: pill centered at (cx, thumb_y)
                    let thumb_w = 24.0;
                    let thumb_h = 13.0;
                    let thumb_rect =
                        Rect::from_center_size(pos2(cx, thumb_y), Vec2::new(thumb_w, thumb_h));
                    let thumb_bg = if slider_drag {
                        Theme::ACCENT_PINK
                    } else if slider_hov {
                        Theme::ACCENT_PURPLE
                    } else {
                        Theme::BG_PANEL_RAISED
                    };
                    let thumb_stroke = if slider_drag || slider_hov {
                        Stroke::new(1.5, Color32::WHITE)
                    } else {
                        Stroke::new(1.2, Theme::ACCENT_PINK)
                    };

                    ui.painter().rect(
                        thumb_rect,
                        CornerRadius::same(6),
                        thumb_bg,
                        thumb_stroke,
                        egui::StrokeKind::Inside,
                    );

                    // Center grip notch line
                    let grip_col = if slider_drag || slider_hov {
                        Color32::WHITE
                    } else {
                        Theme::TEXT_PRIMARY.gamma_multiply(0.8)
                    };
                    ui.painter().line_segment(
                        [pos2(cx - 5.0, thumb_y), pos2(cx + 5.0, thumb_y)],
                        Stroke::new(1.5, grip_col),
                    );

                    slider_resp.on_hover_text(format!(
                        "Zoom: {:.0}%\nDrag to adjust zoom scale",
                        self.zoom * 100.0
                    ));

                    ui.add_space(2.0);

                    // [-] button
                    let (minus_rect, minus_resp) =
                        ui.allocate_exact_size(Vec2::new(36.0, 22.0), Sense::click());
                    let minus_hov = minus_resp.hovered();
                    ui.painter().rect(
                        minus_rect,
                        CornerRadius::same(4),
                        if minus_hov {
                            Theme::BG_PANEL_RAISED
                        } else {
                            Theme::BG_PANEL
                        },
                        Stroke::new(
                            1.0,
                            if minus_hov {
                                Theme::ACCENT_PINK
                            } else {
                                Theme::ACCENT_PURPLE.gamma_multiply(0.4)
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        minus_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "-",
                        Theme::font_bold(14.0),
                        if minus_hov {
                            Color32::WHITE
                        } else {
                            Theme::TEXT_PRIMARY
                        },
                    );
                    if minus_resp.on_hover_text("Zoom Out (-10%)").clicked() {
                        let old_zoom = self.zoom;
                        let new_zoom = (old_zoom * 0.9).clamp(0.25, 4.0);
                        let center = rect.center();
                        let canvas_center = (center - self.pan) / old_zoom;
                        self.pan = center - canvas_center * new_zoom;
                        self.zoom = new_zoom;
                    }

                    ui.add_space(2.0);

                    // Fixed-size single-line percentage box (No line-wrapping, strictly 36x20px, zero jitter)
                    let (pct_rect, pct_resp) =
                        ui.allocate_exact_size(Vec2::new(36.0, 20.0), Sense::click());
                    let pct_hov = pct_resp.hovered();
                    ui.painter().rect(
                        pct_rect,
                        CornerRadius::same(4),
                        if pct_hov {
                            Theme::ACCENT_PURPLE.gamma_multiply(0.45)
                        } else {
                            Theme::BG_PANEL
                        },
                        Stroke::new(
                            1.0,
                            if pct_hov {
                                Theme::ACCENT_PINK
                            } else {
                                Theme::ACCENT_PURPLE.gamma_multiply(0.4)
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    let zoom_pct = format!("{:.0}%", self.zoom * 100.0);
                    ui.painter().text(
                        pct_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &zoom_pct,
                        Theme::font_bold(9.5),
                        if pct_hov {
                            Color32::WHITE
                        } else {
                            Theme::ACCENT_PINK
                        },
                    );
                    if pct_resp
                        .on_hover_text(format!("Zoom: {}\nClick to reset to 100%", zoom_pct))
                        .clicked()
                    {
                        let old_zoom = self.zoom;
                        let new_zoom = 1.0;
                        let center = rect.center();
                        let canvas_center = (center - self.pan) / old_zoom;
                        self.pan = center - canvas_center * new_zoom;
                        self.zoom = new_zoom;
                    }

                    ui.add_space(3.0);
                    ui.separator();
                    ui.add_space(3.0);

                    // Actions: Undo, Redo, Delete
                    // 1. Undo Button (Ctrl+Z)
                    let (undo_rect, undo_resp) =
                        ui.allocate_exact_size(Vec2::new(36.0, 28.0), Sense::click());
                    let undo_hov = undo_resp.hovered();
                    ui.painter().rect(
                        undo_rect,
                        CornerRadius::same(5),
                        if undo_hov {
                            Theme::BG_PANEL_RAISED
                        } else {
                            Theme::BG_PANEL
                        },
                        Stroke::new(
                            1.0,
                            if undo_hov {
                                Theme::ACCENT_PINK
                            } else {
                                Theme::ACCENT_PURPLE.gamma_multiply(0.4)
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    let undo_col = if undo_hov {
                        Color32::WHITE
                    } else {
                        Theme::TEXT_PRIMARY
                    };
                    let uc = undo_rect.center();
                    let up = ui.painter();
                    let us = Stroke::new(1.4, undo_col);
                    // Smooth curved arc for undo: bending up and to the left
                    up.line_segment(
                        [pos2(uc.x + 5.0, uc.y + 4.0), pos2(uc.x + 4.5, uc.y - 1.0)],
                        us,
                    );
                    up.line_segment(
                        [pos2(uc.x + 4.5, uc.y - 1.0), pos2(uc.x + 1.5, uc.y - 4.0)],
                        us,
                    );
                    up.line_segment(
                        [pos2(uc.x + 1.5, uc.y - 4.0), pos2(uc.x - 3.5, uc.y - 4.0)],
                        us,
                    );
                    // Arrowhead pointing down-left
                    up.line_segment(
                        [pos2(uc.x - 4.5, uc.y - 4.0), pos2(uc.x - 1.0, uc.y - 7.0)],
                        us,
                    );
                    up.line_segment(
                        [pos2(uc.x - 4.5, uc.y - 4.0), pos2(uc.x - 1.0, uc.y - 1.0)],
                        us,
                    );

                    if undo_resp.on_hover_text("Undo (Ctrl+Z)").clicked() {
                        meta.trigger_undo = true;
                    }

                    ui.add_space(2.0);

                    // 2. Redo Button (Ctrl+Y)
                    let (redo_rect, redo_resp) =
                        ui.allocate_exact_size(Vec2::new(36.0, 28.0), Sense::click());
                    let redo_hov = redo_resp.hovered();
                    ui.painter().rect(
                        redo_rect,
                        CornerRadius::same(5),
                        if redo_hov {
                            Theme::BG_PANEL_RAISED
                        } else {
                            Theme::BG_PANEL
                        },
                        Stroke::new(
                            1.0,
                            if redo_hov {
                                Theme::ACCENT_PINK
                            } else {
                                Theme::ACCENT_PURPLE.gamma_multiply(0.4)
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    let redo_col = if redo_hov {
                        Color32::WHITE
                    } else {
                        Theme::TEXT_PRIMARY
                    };
                    let rc = redo_rect.center();
                    let rp = ui.painter();
                    let rs = Stroke::new(1.4, redo_col);
                    // Smooth curved arc for redo: bending up and to the right
                    rp.line_segment(
                        [pos2(rc.x - 5.0, rc.y + 4.0), pos2(rc.x - 4.5, rc.y - 1.0)],
                        rs,
                    );
                    rp.line_segment(
                        [pos2(rc.x - 4.5, rc.y - 1.0), pos2(rc.x - 1.5, rc.y - 4.0)],
                        rs,
                    );
                    rp.line_segment(
                        [pos2(rc.x - 1.5, rc.y - 4.0), pos2(rc.x + 3.5, rc.y - 4.0)],
                        rs,
                    );
                    // Arrowhead pointing down-right
                    rp.line_segment(
                        [pos2(rc.x + 4.5, rc.y - 4.0), pos2(rc.x + 1.0, rc.y - 7.0)],
                        rs,
                    );
                    rp.line_segment(
                        [pos2(rc.x + 4.5, rc.y - 4.0), pos2(rc.x + 1.0, rc.y - 1.0)],
                        rs,
                    );

                    if redo_resp.on_hover_text("Redo (Ctrl+Y)").clicked() {
                        meta.trigger_redo = true;
                    }

                    ui.add_space(2.0);

                    // 3. Delete Button (Del)
                    let has_sel = !self.selection.is_empty();
                    let (del_rect, del_resp) =
                        ui.allocate_exact_size(Vec2::new(36.0, 28.0), Sense::click());
                    let del_hov = del_resp.hovered() && has_sel;
                    let del_bg = if del_hov {
                        Theme::ACCENT_RED.gamma_multiply(0.35)
                    } else if has_sel {
                        Theme::BG_PANEL
                    } else {
                        Theme::BG_PANEL.gamma_multiply(0.6)
                    };
                    let del_stroke = if del_hov {
                        Stroke::new(1.5, Theme::ACCENT_RED)
                    } else if has_sel {
                        Stroke::new(1.0, Theme::ACCENT_RED.gamma_multiply(0.7))
                    } else {
                        Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.25))
                    };
                    ui.painter().rect(
                        del_rect,
                        CornerRadius::same(5),
                        del_bg,
                        del_stroke,
                        egui::StrokeKind::Inside,
                    );
                    let del_col = if del_hov {
                        Color32::WHITE
                    } else if has_sel {
                        Theme::ACCENT_RED
                    } else {
                        Theme::TEXT_MUTED.gamma_multiply(0.4)
                    };
                    let dc = del_rect.center();
                    let dp = ui.painter();
                    let ds = Stroke::new(1.3, del_col);
                    // Trash can: Lid
                    dp.line_segment(
                        [pos2(dc.x - 6.0, dc.y - 3.0), pos2(dc.x + 6.0, dc.y - 3.0)],
                        ds,
                    );
                    dp.line_segment(
                        [pos2(dc.x - 2.0, dc.y - 5.0), pos2(dc.x + 2.0, dc.y - 5.0)],
                        ds,
                    );
                    // Body
                    dp.line_segment(
                        [pos2(dc.x - 4.5, dc.y - 1.5), pos2(dc.x - 3.5, dc.y + 6.0)],
                        ds,
                    );
                    dp.line_segment(
                        [pos2(dc.x - 3.5, dc.y + 6.0), pos2(dc.x + 3.5, dc.y + 6.0)],
                        ds,
                    );
                    dp.line_segment(
                        [pos2(dc.x + 3.5, dc.y + 6.0), pos2(dc.x + 4.5, dc.y - 1.5)],
                        ds,
                    );
                    // Internal ribs
                    dp.line_segment(
                        [pos2(dc.x - 1.5, dc.y), pos2(dc.x - 1.2, dc.y + 4.5)],
                        ds,
                    );
                    dp.line_segment(
                        [pos2(dc.x + 1.5, dc.y), pos2(dc.x + 1.2, dc.y + 4.5)],
                        ds,
                    );

                    if del_resp
                        .on_hover_text(if has_sel {
                            "Delete Selected (Del)"
                        } else {
                            "Delete (No components selected)"
                        })
                        .clicked()
                        && has_sel
                    {
                        meta.trigger_delete = true;
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

    pub fn get_wire_search_candidates(
        source_is_output: bool,
        subcircuits: &[String],
    ) -> Vec<GateKind> {
        let mut all: Vec<GateKind> = vec![
            // Logic Gates
            GateKind::And,
            GateKind::Or,
            GateKind::Not,
            GateKind::Nand,
            GateKind::Nor,
            GateKind::Xor,
            GateKind::Xnor,
            // Input / Output
            GateKind::ToggleSwitch,
            GateKind::BitSwitch,
            GateKind::Led,
            GateKind::Clock,
            // Displays
            GateKind::SingleBitDisplay,
            GateKind::BinaryDisplay4,
            GateKind::HexDisplay,
            GateKind::SevenSegment,
            // Arithmetic
            GateKind::HalfAdder,
            GateKind::FullAdder,
            GateKind::HalfSubtractor,
            GateKind::FullSubtractor,
            GateKind::RippleCarryAdder4,
            GateKind::CarryLookaheadAdder4,
            // Sequential
            GateKind::SrLatch,
            GateKind::DLatch,
            GateKind::DFlipFlop,
            GateKind::JkFlipFlop,
            GateKind::TFlipFlop,
            GateKind::Register4,
            GateKind::ShiftRegister4,
            GateKind::Counter4,
            GateKind::ClockDivider,
            // Routing & Selection
            GateKind::Mux2,
            GateKind::Mux4,
            GateKind::Demux2,
            GateKind::Demux4,
            GateKind::Encoder4to2,
            GateKind::Decoder2to4,
            GateKind::Comparator2,
            // Computer Blocks
            GateKind::Alu4,
            GateKind::Rom16x4,
            GateKind::Ram16x4,
        ];

        for name in subcircuits {
            all.push(GateKind::SubcircuitInstance(name.clone()));
        }

        all.into_iter()
            .filter(|k| {
                if source_is_output {
                    k.input_count() > 0
                } else {
                    k.output_count() > 0
                }
            })
            .collect()
    }

    pub fn score_component(kind: &GateKind, query: &str) -> Option<u32> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Some(100);
        }
        let name = kind.display_name().to_lowercase();
        let code = kind.short_code().to_lowercase();
        let cat = kind.category().to_lowercase();

        // 0: Exact match on short code or full name (highest priority)
        if code == q || name == q {
            return Some(0);
        }
        // 1: Name or short code starts with query
        if code.starts_with(&q) || name.starts_with(&q) {
            return Some(1);
        }
        // 2: Any individual word in the name starts with query (e.g. "Gate", "Adder", "Switch")
        if name.split(|c: char| !c.is_alphanumeric()).any(|word| word.starts_with(&q)) {
            return Some(2);
        }
        // 3: Substring in short code or full name
        if code.contains(&q) || name.contains(&q) {
            return Some(3);
        }
        // 4: Category contains query
        if cat.contains(&q) {
            return Some(4);
        }
        None
    }

    fn show_wire_drop_search_menu(
        &mut self,
        ctx: &egui::Context,
        canvas_rect: Rect,
        circuit: &mut Circuit,
        meta: &mut CanvasResponse,
    ) {
        let Some(mut search) = self.wire_drop_search.take() else {
            return;
        };

        let subcircuits: Vec<String> = circuit.subcircuits.keys().cloned().collect();
        let candidates = Self::get_wire_search_candidates(search.source_endpoint.is_output, &subcircuits);

        // Filter and score candidates
        let mut scored: Vec<(u32, usize, GateKind)> = candidates
            .into_iter()
            .enumerate()
            .filter_map(|(orig_idx, kind)| {
                Self::score_component(&kind, &search.search_query)
                    .map(|score| (score, orig_idx, kind))
            })
            .collect();

        // Sort by score ascending, then original index
        scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        let filtered_kinds: Vec<GateKind> = scored.into_iter().map(|(_, _, k)| k).collect();

        // Clamp selected index
        if search.selected_index >= filtered_kinds.len() {
            search.selected_index = filtered_kinds.len().saturating_sub(1);
        }

        let mut close_menu = false;
        let mut chosen_kind: Option<GateKind> = None;
        let just_opened = search.request_focus;

        let drop_screen = self.canvas_to_screen(search.drop_canvas_pos);
        let popup_w = 280.0;
        let popup_h = 320.0;

        let mut popup_pos = drop_screen + Vec2::new(14.0, -20.0);
        if popup_pos.x + popup_w > canvas_rect.max.x - 12.0 {
            popup_pos.x = (drop_screen.x - popup_w - 14.0).max(canvas_rect.min.x + 12.0);
        }
        if popup_pos.y + popup_h > canvas_rect.max.y - 12.0 {
            popup_pos.y = (canvas_rect.max.y - popup_h - 12.0).max(canvas_rect.min.y + 12.0);
        }

        let area_id = egui::Id::new("quick_add_search_popup_area");
        let area_response = egui::Area::new(area_id)
            .order(egui::Order::Foreground)
            .fixed_pos(popup_pos)
            .show(ctx, |ui| {
                let frame = egui::Frame::new()
                    .fill(Color32::from_rgb(18, 16, 26))
                    .corner_radius(CornerRadius::same(8))
                    .stroke(Stroke::new(1.5, Theme::ACCENT_PINK.gamma_multiply(0.85)))
                    .shadow(egui::Shadow {
                        offset: [0, 6],
                        blur: 16,
                        spread: 0,
                        color: Color32::from_black_alpha(190),
                    })
                    .inner_margin(egui::Margin::symmetric(10, 8));

                frame.show(ui, |ui| {
                    ui.set_width(popup_w - 20.0);

                    // Header badge
                    let badge_text = if search.source_endpoint.is_output {
                        "CONNECT TO INPUT -> QUICK ADD"
                    } else {
                        "PROVIDE SIGNAL FROM -> QUICK ADD"
                    };
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(badge_text)
                                .font(Theme::font_bold(10.0))
                                .color(Theme::ACCENT_PINK),
                        );
                    });
                    ui.add_space(4.0);

                    // Search input box
                    let prev_query = search.search_query.clone();
                    let text_edit = egui::TextEdit::singleline(&mut search.search_query)
                        .hint_text("Search...")
                        .font(Theme::font_bold(12.0))
                        .text_color(Theme::TEXT_PRIMARY)
                        .desired_width(ui.available_width());

                    let edit_output = ui.add(text_edit);
                    if search.request_focus {
                        edit_output.request_focus();
                        search.request_focus = false;
                    }

                    // Reset selected index if query changed
                    if search.search_query != prev_query {
                        search.selected_index = 0;
                    }

                    // Global key navigation
                    let enter_pressed = (edit_output.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)))
                        || ctx.input(|i| i.key_pressed(Key::Enter));
                    let escape_pressed = ctx.input(|i| i.key_pressed(Key::Escape));
                    let arrow_up = ctx.input(|i| i.key_pressed(Key::ArrowUp));
                    let arrow_down = ctx.input(|i| i.key_pressed(Key::ArrowDown));
                    let tab_pressed = ctx.input(|i| i.key_pressed(Key::Tab));
                    let navigated_by_keyboard = arrow_down || arrow_up || tab_pressed;

                    if escape_pressed {
                        close_menu = true;
                    }

                    if arrow_down || tab_pressed {
                        if !filtered_kinds.is_empty() {
                            search.selected_index = (search.selected_index + 1) % filtered_kinds.len();
                        }
                    }
                    if arrow_up {
                        if !filtered_kinds.is_empty() {
                            if search.selected_index == 0 {
                                search.selected_index = filtered_kinds.len().saturating_sub(1);
                            } else {
                                search.selected_index -= 1;
                            }
                        }
                    }

                    if enter_pressed && !filtered_kinds.is_empty() {
                        chosen_kind = Some(filtered_kinds[search.selected_index].clone());
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Candidate list in scroll area
                    egui::ScrollArea::vertical()
                        .max_height(190.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if filtered_kinds.is_empty() {
                                ui.add_space(16.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new("No matching components")
                                            .font(Theme::font_regular(11.0))
                                            .color(Theme::TEXT_MUTED),
                                    );
                                });
                                ui.add_space(16.0);
                            } else {
                                for (i, kind) in filtered_kinds.iter().enumerate() {
                                    let is_selected = i == search.selected_index;
                                    let (rect, row_resp) = ui.allocate_exact_size(
                                        Vec2::new(ui.available_width(), 26.0),
                                        Sense::click(),
                                    );

                                    if row_resp.clicked() {
                                        chosen_kind = Some(kind.clone());
                                    }

                                    // Only scroll to item when navigated by keyboard keys, not on mouse movement
                                    if is_selected && navigated_by_keyboard {
                                        row_resp.scroll_to_me(Some(egui::Align::Center));
                                    }

                                    // Background
                                    let bg = if is_selected {
                                        Theme::ACCENT_PURPLE.gamma_multiply(0.4)
                                    } else if row_resp.hovered() {
                                        Theme::BG_PANEL_RAISED
                                    } else {
                                        Color32::TRANSPARENT
                                    };
                                    ui.painter().rect_filled(rect, CornerRadius::same(4), bg);

                                    if is_selected {
                                        // Left indicator bar
                                        let ind_rect = Rect::from_min_size(
                                            rect.min,
                                            Vec2::new(3.0, rect.height()),
                                        );
                                        ui.painter().rect_filled(ind_rect, CornerRadius::same(2), Theme::ACCENT_PINK);
                                    }

                                    // Badge with short code: surrounded with purple outline, no white bubble
                                    let code = kind.short_code();
                                    let badge_rect = Rect::from_min_size(
                                        Pos2::new(rect.min.x + 8.0, rect.min.y + 4.0),
                                        Vec2::new(42.0, 18.0),
                                    );
                                    let badge_stroke = if is_selected {
                                        Stroke::new(1.2, Theme::ACCENT_PINK)
                                    } else if row_resp.hovered() {
                                        Stroke::new(1.2, Theme::ACCENT_PURPLE)
                                    } else {
                                        Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.75))
                                    };
                                    ui.painter().rect_filled(
                                        badge_rect,
                                        CornerRadius::same(4),
                                        Color32::from_rgba_premultiplied(32, 22, 48, 180),
                                    );
                                    ui.painter().rect_stroke(
                                        badge_rect,
                                        CornerRadius::same(4),
                                        badge_stroke,
                                        egui::StrokeKind::Inside,
                                    );
                                    ui.painter().text(
                                        badge_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        code,
                                        Theme::font_bold(9.5),
                                        if is_selected { Color32::WHITE } else { Theme::ACCENT_PINK },
                                    );

                                    // Display name
                                    let name_pos = Pos2::new(rect.min.x + 56.0, rect.center().y);
                                    ui.painter().text(
                                        name_pos,
                                        egui::Align2::LEFT_CENTER,
                                        kind.display_name(),
                                        Theme::font_bold(11.0),
                                        if is_selected { Color32::WHITE } else { Theme::TEXT_PRIMARY },
                                    );

                                    // Category on the right
                                    let cat_pos = Pos2::new(rect.max.x - 6.0, rect.center().y);
                                    ui.painter().text(
                                        cat_pos,
                                        egui::Align2::RIGHT_CENTER,
                                        kind.category(),
                                        Theme::font_regular(9.0),
                                        Theme::TEXT_MUTED,
                                    );

                                    ui.add_space(2.0);
                                }
                            }
                        });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(2.0);

                    // Footer hint
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Enter: Select  *  Esc: Cancel")
                                .font(Theme::font_regular(9.5))
                                .color(Theme::TEXT_MUTED),
                        );
                    });
                });
            });

        // Check if user clicked outside the popup
        if !just_opened
            && ctx.input(|i| i.pointer.primary_clicked() || i.pointer.secondary_clicked())
        {
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                if !area_response.response.rect.contains(pos) {
                    close_menu = true;
                }
            }
        }

        // Apply choice if any
        if let Some(kind) = chosen_kind {
            let id = circuit.add_component(kind, (0.0, 0.0));
            let target_is_output = !search.source_endpoint.is_output;
            let port_offset = if let Some(comp) = circuit.components.get(id) {
                let wpos = comp.port_world_pos(target_is_output, 0);
                (wpos.0, wpos.1)
            } else {
                (0.0, 0.0)
            };

            let desired_port_canvas = self.snap_point(search.drop_canvas_pos);
            let comp_pos = Pos2::new(
                desired_port_canvas.x - port_offset.0,
                desired_port_canvas.y - port_offset.1,
            );
            let comp_pos_snapped = self.snap_point(comp_pos);
            if let Some(comp) = circuit.components.get_mut(id) {
                comp.pos = (comp_pos_snapped.x, comp_pos_snapped.y);
            }

            let target_endpoint = PortEndpoint {
                component_id: id,
                is_output: target_is_output,
                port_index: 0,
            };

            let (src, sink) = if search.source_endpoint.is_output {
                (search.source_endpoint, target_endpoint)
            } else {
                (target_endpoint, search.source_endpoint)
            };

            circuit.connect_ports(src, sink);
            Simulator::settle(circuit);

            self.selection.select_single(id);
            meta.circuit_mutated = true;
            meta.placed_component = true;
            self.wire_drop_search = None;
            return;
        }

        if close_menu {
            self.wire_drop_search = None;
        } else {
            self.wire_drop_search = Some(search);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logic_core::GateKind;

    #[test]
    fn test_wire_search_candidates_filtering() {
        // Output wire looking for input component
        let output_candidates = CanvasState::get_wire_search_candidates(true, &[]);
        assert!(output_candidates.contains(&GateKind::And));
        assert!(output_candidates.contains(&GateKind::Led));
        assert!(!output_candidates.contains(&GateKind::ToggleSwitch));
        assert!(!output_candidates.contains(&GateKind::Clock));

        // Input wire looking for output/source component
        let input_candidates = CanvasState::get_wire_search_candidates(false, &[]);
        assert!(input_candidates.contains(&GateKind::ToggleSwitch));
        assert!(input_candidates.contains(&GateKind::Clock));
        assert!(input_candidates.contains(&GateKind::And));
        assert!(!input_candidates.contains(&GateKind::Led));
        assert!(!input_candidates.contains(&GateKind::SevenSegment));
    }

    #[test]
    fn test_score_component_ranking() {
        // "and" matches AND Gate with score 0 ahead of NAND Gate (score 3)
        let and_score = CanvasState::score_component(&GateKind::And, "and").unwrap();
        let nand_score = CanvasState::score_component(&GateKind::Nand, "and").unwrap();
        assert_eq!(and_score, 0);
        assert_eq!(nand_score, 3);
        assert!(and_score < nand_score);

        // "or" matches OR Gate with score 0 ahead of NOR Gate (score 3)
        let or_score = CanvasState::score_component(&GateKind::Or, "or").unwrap();
        let nor_score = CanvasState::score_component(&GateKind::Nor, "or").unwrap();
        assert_eq!(or_score, 0);
        assert_eq!(nor_score, 3);
        assert!(or_score < nor_score);

        // "xor" matches XOR Gate with score 0
        let xor_score = CanvasState::score_component(&GateKind::Xor, "xor").unwrap();
        assert_eq!(xor_score, 0);

        // "xnor" matches XNOR Gate with score 0
        let xnor_score = CanvasState::score_component(&GateKind::Xnor, "xnor").unwrap();
        assert_eq!(xnor_score, 0);

        // "not" matches NOT Gate with score 0
        assert_eq!(CanvasState::score_component(&GateKind::Not, "not"), Some(0));

        // "dff" matches D Flip-Flop with score 0
        assert_eq!(CanvasState::score_component(&GateKind::DFlipFlop, "dff"), Some(0));

        // "led" matches LED with score 0
        assert_eq!(CanvasState::score_component(&GateKind::Led, "led"), Some(0));
    }

    #[test]
    fn test_quick_add_auto_connection() {
        let mut circuit = Circuit::new();
        let sw_id = circuit.add_component(GateKind::ToggleSwitch, (100.0, 100.0));
        let sw_out = PortEndpoint {
            component_id: sw_id,
            is_output: true,
            port_index: 0,
        };

        // Simulate wire release in air
        let mut canvas = CanvasState::default();
        canvas.wire_drop_search = Some(WireDropSearchState {
            source_endpoint: sw_out,
            source_canvas_pos: Pos2::new(140.0, 100.0),
            drop_canvas_pos: Pos2::new(300.0, 100.0),
            search_query: "and".to_string(),
            selected_index: 0,
            request_focus: false,
        });

        assert!(canvas.wire_drop_search.is_some());

        // Quick add AND Gate
        let and_id = circuit.add_component(GateKind::And, (360.0, 120.0));
        let and_in0 = PortEndpoint {
            component_id: and_id,
            is_output: false,
            port_index: 0,
        };
        let net_id = circuit.connect_ports(sw_out, and_in0);
        assert!(net_id.is_some());
        Simulator::settle(&mut circuit);

        let net = circuit.nets.get(net_id.unwrap()).unwrap();
        assert_eq!(net.source, sw_out);
        assert!(net.sinks.contains(&and_in0));
    }
}

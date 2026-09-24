use crate::theme::{Theme, ThemeMode};
use eframe::egui::{Color32, CornerRadius, Painter, Pos2, Rect, Stroke, Vec2};
use logic_core::{ComponentNode, GateKind, Signal};

pub struct GlyphRenderer;

impl GlyphRenderer {
    pub fn draw_component(
        painter: &Painter,
        comp: &ComponentNode,
        is_selected: bool,
        is_hovered: bool,
        to_screen: impl Fn(Pos2) -> Pos2,
        zoom: f32,
        theme_mode: ThemeMode,
    ) {
        let center_canvas = Pos2::new(comp.pos.0, comp.pos.1);
        let rotation = comp.rotation;

        // Coordinate transformer from component local space (pixels at zoom 1.0) to screen space
        let local_to_screen = |lx: f32, ly: f32| -> Pos2 {
            let rotated = rotation.rotate_point((lx, ly));
            let canvas_p = Pos2::new(center_canvas.x + rotated.0, center_canvas.y + rotated.1);
            to_screen(canvas_p)
        };

        // Determine outline stroke
        let stroke_color = if is_selected {
            Theme::ACCENT_PINK
        } else if is_hovered {
            Theme::ACCENT_PINK.gamma_multiply(0.8)
        } else {
            theme_mode.gate_stroke()
        };
        let stroke_width = if is_selected { 2.0 * zoom } else { 1.3 * zoom };
        let body_stroke = Stroke::new(stroke_width, stroke_color);
        let body_fill = theme_mode.gate_fill();

        // 1. Draw input pin stubs
        let in_count = comp.input_signals.len();
        let (hw, hh) = comp.half_dimensions();
        for i in 0..in_count {
            let port_offset = comp.port_offset_unrotated(false, i);
            let y = port_offset.1;
            let body_edge_x = match comp.kind {
                GateKind::And | GateKind::Nand => -25.0,
                GateKind::Or | GateKind::Nor => {
                    let s = (y / hh).clamp(-1.0, 1.0);
                    -25.0 + 12.0 * (1.0 - s * s)
                }
                GateKind::Xor | GateKind::Xnor => {
                    let s = (y / hh).clamp(-1.0, 1.0);
                    -32.0 + 12.0 * (1.0 - s * s)
                }
                GateKind::Not => -20.0,
                GateKind::Led => -18.0,
                GateKind::ToggleSwitch | GateKind::Clock => -20.0,
                _ => -hw,
            };

            let p_pin = local_to_screen(port_offset.0, y);
            let p_body = local_to_screen(body_edge_x, y);

            let sig = comp.input_signals.get(i).copied().unwrap_or(Signal::Zero);
            let pin_color = Self::signal_color(sig);

            painter.line_segment([p_pin, p_body], Stroke::new(1.5 * zoom, pin_color));
            Self::draw_pin_dot(painter, p_pin, sig, zoom, theme_mode);
        }

        // 2. Draw output pin stubs
        let out_count = comp.output_signals.len();
        for i in 0..out_count {
            let port_offset = comp.port_offset_unrotated(true, i);
            let y = port_offset.1;
            let body_edge_x = match comp.kind {
                GateKind::Not => 25.0, // right edge of bubble at center 21.5 + radius 3.5
                GateKind::Nand | GateKind::Nor | GateKind::Xnor => 32.0, // right edge of bubble at center 28.5 + radius 3.5
                GateKind::And | GateKind::Or | GateKind::Xor => 25.0,    // nose tip of gate
                GateKind::ToggleSwitch | GateKind::Clock => 20.0,
                _ => hw,
            };

            let p_pin = local_to_screen(port_offset.0, y);
            let p_body = local_to_screen(body_edge_x, y);

            let sig = comp.output_signals.get(i).copied().unwrap_or(Signal::Zero);
            let pin_color = Self::signal_color(sig);

            painter.line_segment([p_body, p_pin], Stroke::new(1.5 * zoom, pin_color));
            Self::draw_pin_dot(painter, p_pin, sig, zoom, theme_mode);
        }

        // 3. Draw gate/block body symbol with crisp function name labels
        match &comp.kind {
            GateKind::And => {
                Self::draw_and_shape(painter, &local_to_screen, body_fill, body_stroke, hh);
                painter.text(
                    local_to_screen(-5.0, 0.0),
                    egui::Align2::CENTER_CENTER,
                    "AND",
                    Theme::font_bold(10.0 * zoom.clamp(0.7, 1.4)),
                    Theme::TEXT_PRIMARY,
                );
            }
            GateKind::Nand => {
                Self::draw_and_shape(painter, &local_to_screen, body_fill, body_stroke, hh);
                Self::draw_bubble(
                    painter,
                    local_to_screen(28.5, 0.0),
                    body_stroke,
                    zoom,
                    theme_mode.bubble_fill(),
                );
                painter.text(
                    local_to_screen(-5.0, 0.0),
                    egui::Align2::CENTER_CENTER,
                    "NAND",
                    Theme::font_bold(9.0 * zoom.clamp(0.7, 1.4)),
                    Theme::TEXT_PRIMARY,
                );
            }
            GateKind::Or => {
                Self::draw_or_shape(painter, &local_to_screen, body_fill, body_stroke, hh);
                painter.text(
                    local_to_screen(2.0, 0.0),
                    egui::Align2::CENTER_CENTER,
                    "OR",
                    Theme::font_bold(10.0 * zoom.clamp(0.7, 1.4)),
                    Theme::TEXT_PRIMARY,
                );
            }
            GateKind::Nor => {
                Self::draw_or_shape(painter, &local_to_screen, body_fill, body_stroke, hh);
                Self::draw_bubble(
                    painter,
                    local_to_screen(28.5, 0.0),
                    body_stroke,
                    zoom,
                    theme_mode.bubble_fill(),
                );
                painter.text(
                    local_to_screen(2.0, 0.0),
                    egui::Align2::CENTER_CENTER,
                    "NOR",
                    Theme::font_bold(9.0 * zoom.clamp(0.7, 1.4)),
                    Theme::TEXT_PRIMARY,
                );
            }
            GateKind::Xor => {
                Self::draw_xor_shape(painter, &local_to_screen, body_fill, body_stroke, hh);
                painter.text(
                    local_to_screen(2.0, 0.0),
                    egui::Align2::CENTER_CENTER,
                    "XOR",
                    Theme::font_bold(10.0 * zoom.clamp(0.7, 1.4)),
                    Theme::TEXT_PRIMARY,
                );
            }
            GateKind::Xnor => {
                Self::draw_xor_shape(painter, &local_to_screen, body_fill, body_stroke, hh);
                Self::draw_bubble(
                    painter,
                    local_to_screen(28.5, 0.0),
                    body_stroke,
                    zoom,
                    theme_mode.bubble_fill(),
                );
                painter.text(
                    local_to_screen(2.0, 0.0),
                    egui::Align2::CENTER_CENTER,
                    "XNOR",
                    Theme::font_bold(9.0 * zoom.clamp(0.7, 1.4)),
                    Theme::TEXT_PRIMARY,
                );
            }
            GateKind::Not => {
                Self::draw_not_shape(painter, &local_to_screen, body_fill, body_stroke);
                Self::draw_bubble(
                    painter,
                    local_to_screen(21.5, 0.0),
                    body_stroke,
                    zoom,
                    theme_mode.bubble_fill(),
                );
                painter.text(
                    local_to_screen(-5.0, 0.0),
                    egui::Align2::CENTER_CENTER,
                    "NOT",
                    Theme::font_bold(9.0 * zoom.clamp(0.7, 1.4)),
                    Theme::TEXT_PRIMARY,
                );
            }
            GateKind::ToggleSwitch => {
                Self::draw_toggle_switch(
                    painter,
                    &local_to_screen,
                    comp,
                    body_stroke,
                    zoom,
                    theme_mode,
                );
            }
            GateKind::Led => {
                Self::draw_led(
                    painter,
                    &local_to_screen,
                    comp,
                    body_stroke,
                    zoom,
                    theme_mode,
                );
            }
            GateKind::Clock => {
                Self::draw_clock(
                    painter,
                    &local_to_screen,
                    comp,
                    body_stroke,
                    zoom,
                    theme_mode,
                );
            }
            GateKind::BinaryDisplay4 => {
                Self::draw_binary_display(
                    painter,
                    &local_to_screen,
                    comp,
                    body_stroke,
                    zoom,
                    theme_mode,
                );
            }
            GateKind::HexDisplay => {
                Self::draw_hex_display(
                    painter,
                    &local_to_screen,
                    comp,
                    body_stroke,
                    zoom,
                    theme_mode,
                );
            }
            GateKind::SevenSegment => {
                Self::draw_seven_segment(
                    painter,
                    &local_to_screen,
                    comp,
                    body_stroke,
                    zoom,
                    theme_mode,
                );
            }
            _ => {
                let title = comp.kind.short_code();
                Self::draw_chip_block(
                    painter,
                    &local_to_screen,
                    comp,
                    &title,
                    body_stroke,
                    zoom,
                    theme_mode,
                );
            }
        }

        // 4. Draw custom component label if set (e.g. "U1", "ENABLE", "G1")
        if !comp.label.is_empty() {
            let label_pos = local_to_screen(0.0, -hh - 8.0);
            painter.text(
                label_pos,
                egui::Align2::CENTER_BOTTOM,
                &comp.label,
                Theme::font_bold(10.5 * zoom.clamp(0.7, 1.4)),
                Theme::TEXT_PRIMARY,
            );
        }
    }

    fn signal_color(sig: Signal) -> Color32 {
        match sig {
            Signal::Zero => Theme::SIGNAL_LOW,
            Signal::One => Theme::SIGNAL_HIGH,
            Signal::X => Theme::SIGNAL_X,
            Signal::Z => Color32::TRANSPARENT,
        }
    }

    fn draw_pin_dot(painter: &Painter, pos: Pos2, sig: Signal, zoom: f32, theme_mode: ThemeMode) {
        let radius = 3.0 * zoom;
        match sig {
            Signal::Zero => {
                painter.circle_filled(pos, radius, Theme::SIGNAL_LOW);
                painter.circle_stroke(pos, radius, Stroke::new(1.0 * zoom, theme_mode.bg_canvas()));
            }
            Signal::One => {
                painter.circle_filled(pos, radius, Theme::SIGNAL_HIGH);
                painter.circle_stroke(
                    pos,
                    radius + 1.5 * zoom,
                    Stroke::new(1.0 * zoom, Theme::ACCENT_PINK.gamma_multiply(0.4)),
                );
            }
            Signal::X => {
                painter.circle_filled(pos, radius, Theme::SIGNAL_X);
            }
            Signal::Z => {
                painter.circle_stroke(pos, radius, Stroke::new(1.2 * zoom, Theme::SIGNAL_LOW));
            }
        }
    }

    fn draw_bubble(
        painter: &Painter,
        center: Pos2,
        stroke: Stroke,
        zoom: f32,
        bubble_fill: Color32,
    ) {
        let radius = 3.5 * zoom;
        painter.circle(center, radius, bubble_fill, stroke);
    }

    fn draw_and_shape(
        painter: &Painter,
        local_to_screen: &impl Fn(f32, f32) -> Pos2,
        fill: Color32,
        stroke: Stroke,
        half_h: f32,
    ) {
        let mut pts = Vec::with_capacity(22);
        pts.push(local_to_screen(-25.0, -half_h));
        pts.push(local_to_screen(0.0, -half_h));

        let segments = 16;
        for i in 1..segments {
            let t =
                -std::f32::consts::FRAC_PI_2 + (i as f32 / segments as f32) * std::f32::consts::PI;
            let x = 25.0 * t.cos();
            let y = half_h * t.sin();
            pts.push(local_to_screen(x, y));
        }

        pts.push(local_to_screen(0.0, half_h));
        pts.push(local_to_screen(-25.0, half_h));

        painter.add(egui::epaint::PathShape::convex_polygon(pts, fill, stroke));
    }

    fn draw_or_shape(
        painter: &Painter,
        local_to_screen: &impl Fn(f32, f32) -> Pos2,
        fill: Color32,
        stroke: Stroke,
        half_h: f32,
    ) {
        let segments = 24;

        // 1. Construct the exact perimeter points in continuous counter-clockwise order
        let mut perimeter = Vec::with_capacity(segments * 3 + 2);

        // Top curve from (-25.0, -half_h) to (25.0, 0.0)
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let x = -25.0 + 50.0 * t;
            let y = -half_h * (1.0 - t * t);
            perimeter.push(local_to_screen(x, y));
        }

        // Bottom curve from (25.0, 0.0) to (-25.0, half_h)
        for i in 1..=segments {
            let t = i as f32 / segments as f32;
            let x = 25.0 - 50.0 * t;
            let y = half_h * (1.0 - (1.0 - t) * (1.0 - t));
            perimeter.push(local_to_screen(x, y));
        }

        // Back concave curve from (-25.0, half_h) inward to (-13.0, 0.0) and to (-25.0, -half_h)
        for i in 1..segments {
            let t = i as f32 / segments as f32;
            let y = half_h - 2.0 * half_h * t;
            let s = y / half_h;
            let x = -25.0 + 12.0 * (1.0 - s * s);
            perimeter.push(local_to_screen(x, y));
        }

        // 2. Fill the interior using an indexed triangle fan from internal star-center (5.0, 0.0)
        let mut mesh = egui::Mesh::default();
        let center = local_to_screen(5.0, 0.0);
        let c_idx = 0u32;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center,
            uv: egui::epaint::WHITE_UV,
            color: fill,
        });

        for pt in &perimeter {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: *pt,
                uv: egui::epaint::WHITE_UV,
                color: fill,
            });
        }

        let n = perimeter.len() as u32;
        for i in 0..n {
            let next = (i + 1) % n;
            mesh.add_triangle(c_idx, 1 + i, 1 + next);
        }

        painter.add(mesh);

        // 3. Draw the stroke outline around the exact same perimeter
        painter.add(egui::epaint::PathShape::closed_line(perimeter, stroke));
    }

    fn draw_xor_shape(
        painter: &Painter,
        local_to_screen: &impl Fn(f32, f32) -> Pos2,
        fill: Color32,
        stroke: Stroke,
        half_h: f32,
    ) {
        Self::draw_or_shape(painter, local_to_screen, fill, stroke, half_h);

        // Draw parallel outer input arc
        let segments = 24;
        let mut arc_pts = Vec::with_capacity(segments + 1);
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let y = -half_h + 2.0 * half_h * t;
            let s = y / half_h;
            let x = -32.0 + 12.0 * (1.0 - s * s);
            arc_pts.push(local_to_screen(x, y));
        }
        painter.add(egui::epaint::PathShape::line(arc_pts, stroke));
    }

    fn draw_not_shape(
        painter: &Painter,
        local_to_screen: &impl Fn(f32, f32) -> Pos2,
        fill: Color32,
        stroke: Stroke,
    ) {
        let pts = vec![
            local_to_screen(-20.0, -18.0),
            local_to_screen(18.0, 0.0),
            local_to_screen(-20.0, 18.0),
        ];
        painter.add(egui::epaint::PathShape::convex_polygon(pts, fill, stroke));
    }

    fn draw_toggle_switch(
        painter: &Painter,
        local_to_screen: &impl Fn(f32, f32) -> Pos2,
        comp: &ComponentNode,
        stroke: Stroke,
        zoom: f32,
        theme_mode: ThemeMode,
    ) {
        let tl = local_to_screen(-20.0, -15.0);
        let br = local_to_screen(20.0, 15.0);
        let min_x = tl.x.min(br.x);
        let max_x = tl.x.max(br.x);
        let min_y = tl.y.min(br.y);
        let max_y = tl.y.max(br.y);
        let rect = Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));

        painter.rect(
            rect,
            CornerRadius::same(4),
            theme_mode.gate_fill(),
            stroke,
            egui::StrokeKind::Inside,
        );

        let is_on = comp.state_flag;

        // Recessed cavity
        let slot_w = 26.0 * zoom;
        let slot_h = 14.0 * zoom;
        let slot_rect = Rect::from_center_size(rect.center(), Vec2::new(slot_w, slot_h));
        painter.rect_filled(
            slot_rect,
            CornerRadius::same(7),
            Color32::from_rgb(0x0C, 0x0A, 0x0E),
        );
        painter.rect_stroke(
            slot_rect,
            CornerRadius::same(7),
            Stroke::new(1.0 * zoom, Color32::from_rgb(0x28, 0x22, 0x30)),
            egui::StrokeKind::Inside,
        );

        // Status LED indicator dot
        let dot_pos = Pos2::new(rect.max.x - 5.0 * zoom, rect.min.y + 5.0 * zoom);
        if is_on {
            painter.circle_filled(dot_pos, 3.0 * zoom, Theme::ACCENT_PINK.gamma_multiply(0.4));
            painter.circle_filled(dot_pos, 1.8 * zoom, Theme::ACCENT_PINK);
        } else {
            painter.circle_filled(dot_pos, 1.8 * zoom, Theme::SIGNAL_LOW);
        }

        // Mechanical toggle lever arm
        let knob_x = if is_on {
            slot_rect.center().x + 6.0 * zoom
        } else {
            slot_rect.center().x - 6.0 * zoom
        };
        let knob_center = Pos2::new(knob_x, slot_rect.center().y);
        let lever_base = slot_rect.center();
        painter.line_segment(
            [lever_base, knob_center],
            Stroke::new(
                3.0 * zoom,
                if is_on {
                    Theme::ACCENT_PINK.gamma_multiply(0.6)
                } else {
                    Color32::from_rgb(0x5A, 0x52, 0x64)
                },
            ),
        );

        // Metallic toggle knob head
        let knob_r = 5.2 * zoom;
        let knob_fill = if is_on {
            Theme::ACCENT_PINK
        } else {
            Color32::from_rgb(0xD0, 0xCA, 0xD8)
        };
        painter.circle_filled(knob_center, knob_r, knob_fill);
        painter.circle_stroke(
            knob_center,
            knob_r,
            Stroke::new(
                1.0 * zoom,
                if is_on {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(0x8C, 0x82, 0x98)
                },
            ),
        );
        // Reflection highlight
        painter.circle_filled(
            Pos2::new(knob_center.x - 1.5 * zoom, knob_center.y - 1.5 * zoom),
            1.5 * zoom,
            Color32::WHITE.gamma_multiply(0.8),
        );
    }

    fn draw_led(
        painter: &Painter,
        local_to_screen: &impl Fn(f32, f32) -> Pos2,
        comp: &ComponentNode,
        stroke: Stroke,
        zoom: f32,
        theme_mode: ThemeMode,
    ) {
        let tl = local_to_screen(-18.0, -18.0);
        let br = local_to_screen(18.0, 18.0);
        let min_x = tl.x.min(br.x);
        let max_x = tl.x.max(br.x);
        let min_y = tl.y.min(br.y);
        let max_y = tl.y.max(br.y);
        let rect = Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));

        painter.rect(
            rect,
            CornerRadius::same(6),
            theme_mode.gate_fill(),
            stroke,
            egui::StrokeKind::Inside,
        );

        let is_high = comp
            .input_signals
            .first()
            .map(|s| s.is_high())
            .unwrap_or(false);
        let lamp_center = rect.center();
        let lamp_radius = 9.0 * zoom;

        if is_high {
            painter.circle_filled(
                lamp_center,
                lamp_radius + 3.0 * zoom,
                Theme::ACCENT_PINK.gamma_multiply(0.3),
            );
            painter.circle_filled(lamp_center, lamp_radius, Theme::ACCENT_PINK);
            painter.circle_filled(
                Pos2::new(lamp_center.x - 2.0 * zoom, lamp_center.y - 2.0 * zoom),
                3.0 * zoom,
                Color32::WHITE.gamma_multiply(0.8),
            );
        } else {
            painter.circle_filled(lamp_center, lamp_radius, Theme::SIGNAL_LOW);
        }
        painter.circle_stroke(
            lamp_center,
            lamp_radius,
            Stroke::new(1.0 * zoom, Color32::from_rgb(0x30, 0x2A, 0x38)),
        );
    }

    fn draw_clock(
        painter: &Painter,
        local_to_screen: &impl Fn(f32, f32) -> Pos2,
        comp: &ComponentNode,
        stroke: Stroke,
        zoom: f32,
        theme_mode: ThemeMode,
    ) {
        let tl = local_to_screen(-20.0, -15.0);
        let br = local_to_screen(20.0, 15.0);
        let min_x = tl.x.min(br.x);
        let max_x = tl.x.max(br.x);
        let min_y = tl.y.min(br.y);
        let max_y = tl.y.max(br.y);
        let rect = Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));

        painter.rect(
            rect,
            CornerRadius::same(4),
            theme_mode.gate_fill(),
            stroke,
            egui::StrokeKind::Inside,
        );

        let center = rect.center();
        let w = 7.0 * zoom;
        let h = 5.0 * zoom;
        let wave_pts = [
            Pos2::new(center.x - 2.0 * w, center.y + h),
            Pos2::new(center.x - w, center.y + h),
            Pos2::new(center.x - w, center.y - h),
            Pos2::new(center.x + w, center.y - h),
            Pos2::new(center.x + w, center.y + h),
            Pos2::new(center.x + 2.0 * w, center.y + h),
        ];
        for pair in wave_pts.windows(2) {
            painter.line_segment(
                [pair[0], pair[1]],
                Stroke::new(1.5 * zoom, Theme::TEXT_PRIMARY),
            );
        }

        if comp.state_flag {
            painter.circle_filled(
                Pos2::new(rect.max.x - 5.0 * zoom, rect.min.y + 5.0 * zoom),
                2.5 * zoom,
                Theme::ACCENT_PINK,
            );
        }
    }

    fn draw_binary_display(
        painter: &Painter,
        local_to_screen: &impl Fn(f32, f32) -> Pos2,
        comp: &ComponentNode,
        stroke: Stroke,
        zoom: f32,
        theme_mode: ThemeMode,
    ) {
        let (hw, hh) = comp.half_dimensions();
        let tl = local_to_screen(-hw, -hh);
        let br = local_to_screen(hw, hh);
        let rect = Rect::from_min_max(
            Pos2::new(tl.x.min(br.x), tl.y.min(br.y)),
            Pos2::new(tl.x.max(br.x), tl.y.max(br.y)),
        );

        painter.rect(
            rect,
            CornerRadius::same(6),
            theme_mode.gate_fill(),
            stroke,
            egui::StrokeKind::Inside,
        );

        // Draw 4 bit dots
        let spacing = 16.0 * zoom;
        let start_x = rect.center().x - 1.5 * spacing;
        for i in 0..4 {
            let sig = comp
                .input_signals
                .get(3 - i)
                .copied()
                .unwrap_or(Signal::Zero);
            let pos = Pos2::new(start_x + (i as f32) * spacing, rect.center().y);
            let color = if sig.is_high() {
                Theme::ACCENT_PINK
            } else {
                Theme::SIGNAL_LOW
            };
            painter.circle_filled(pos, 4.5 * zoom, color);
        }
    }

    fn draw_hex_display(
        painter: &Painter,
        local_to_screen: &impl Fn(f32, f32) -> Pos2,
        comp: &ComponentNode,
        stroke: Stroke,
        zoom: f32,
        theme_mode: ThemeMode,
    ) {
        let (hw, hh) = comp.half_dimensions();
        let tl = local_to_screen(-hw, -hh);
        let br = local_to_screen(hw, hh);
        let rect = Rect::from_min_max(
            Pos2::new(tl.x.min(br.x), tl.y.min(br.y)),
            Pos2::new(tl.x.max(br.x), tl.y.max(br.y)),
        );

        painter.rect(
            rect,
            CornerRadius::same(6),
            theme_mode.gate_fill(),
            stroke,
            egui::StrokeKind::Inside,
        );

        // Decode 4-bit value to hex
        let mut val = 0u8;
        for i in 0..4 {
            if comp
                .input_signals
                .get(i)
                .map(|s| s.is_high())
                .unwrap_or(false)
            {
                val |= 1 << i;
            }
        }
        let hex_char = format!("{:X}", val);

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            hex_char,
            Theme::font_bold(24.0 * zoom.clamp(0.6, 1.8)),
            Theme::ACCENT_PINK,
        );
    }

    fn draw_seven_segment(
        painter: &Painter,
        local_to_screen: &impl Fn(f32, f32) -> Pos2,
        comp: &ComponentNode,
        stroke: Stroke,
        zoom: f32,
        theme_mode: ThemeMode,
    ) {
        let (hw, hh) = comp.half_dimensions();
        let tl = local_to_screen(-hw, -hh);
        let br = local_to_screen(hw, hh);
        let rect = Rect::from_min_max(
            Pos2::new(tl.x.min(br.x), tl.y.min(br.y)),
            Pos2::new(tl.x.max(br.x), tl.y.max(br.y)),
        );

        painter.rect(
            rect,
            CornerRadius::same(6),
            theme_mode.gate_fill(),
            stroke,
            egui::StrokeKind::Inside,
        );

        let center = rect.center();
        let sw = 14.0 * zoom;
        let sh = 14.0 * zoom;
        let seg_w = 2.5 * zoom;

        // Segments: A (top), B (top right), C (bottom right), D (bottom), E (bottom left), F (top left), G (middle)
        let seg_color = |idx: usize| -> Color32 {
            if comp
                .input_signals
                .get(idx)
                .map(|s| s.is_high())
                .unwrap_or(false)
            {
                Theme::ACCENT_PINK
            } else {
                Theme::SIGNAL_LOW.gamma_multiply(0.4)
            }
        };

        // A (top)
        painter.line_segment(
            [
                Pos2::new(center.x - sw * 0.7, center.y - sh),
                Pos2::new(center.x + sw * 0.7, center.y - sh),
            ],
            Stroke::new(seg_w, seg_color(0)),
        );
        // B (top right)
        painter.line_segment(
            [
                Pos2::new(center.x + sw * 0.7, center.y - sh),
                Pos2::new(center.x + sw * 0.7, center.y),
            ],
            Stroke::new(seg_w, seg_color(1)),
        );
        // C (bottom right)
        painter.line_segment(
            [
                Pos2::new(center.x + sw * 0.7, center.y),
                Pos2::new(center.x + sw * 0.7, center.y + sh),
            ],
            Stroke::new(seg_w, seg_color(2)),
        );
        // D (bottom)
        painter.line_segment(
            [
                Pos2::new(center.x - sw * 0.7, center.y + sh),
                Pos2::new(center.x + sw * 0.7, center.y + sh),
            ],
            Stroke::new(seg_w, seg_color(3)),
        );
        // E (bottom left)
        painter.line_segment(
            [
                Pos2::new(center.x - sw * 0.7, center.y),
                Pos2::new(center.x - sw * 0.7, center.y + sh),
            ],
            Stroke::new(seg_w, seg_color(4)),
        );
        // F (top left)
        painter.line_segment(
            [
                Pos2::new(center.x - sw * 0.7, center.y - sh),
                Pos2::new(center.x - sw * 0.7, center.y),
            ],
            Stroke::new(seg_w, seg_color(5)),
        );
        // G (middle)
        painter.line_segment(
            [
                Pos2::new(center.x - sw * 0.7, center.y),
                Pos2::new(center.x + sw * 0.7, center.y),
            ],
            Stroke::new(seg_w, seg_color(6)),
        );
    }

    fn draw_chip_block(
        painter: &Painter,
        local_to_screen: &impl Fn(f32, f32) -> Pos2,
        comp: &ComponentNode,
        title: &str,
        stroke: Stroke,
        zoom: f32,
        theme_mode: ThemeMode,
    ) {
        let (hw, hh) = comp.half_dimensions();
        let tl = local_to_screen(-hw, -hh);
        let br = local_to_screen(hw, hh);
        let rect = Rect::from_min_max(
            Pos2::new(tl.x.min(br.x), tl.y.min(br.y)),
            Pos2::new(tl.x.max(br.x), tl.y.max(br.y)),
        );

        painter.rect(
            rect,
            CornerRadius::same(6),
            theme_mode.gate_fill(),
            stroke,
            egui::StrokeKind::Inside,
        );

        // Top orientation notch
        let notch_pos = Pos2::new(rect.center().x, rect.min.y);
        painter.circle_filled(notch_pos, 3.5 * zoom, theme_mode.bg_canvas());

        // Chip title
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            title.to_uppercase(),
            Theme::font_bold(12.0 * zoom.clamp(0.7, 1.4)),
            Theme::TEXT_PRIMARY,
        );

        // Input pin labels
        let in_count = comp.kind.input_count();
        for i in 0..in_count {
            let offset = comp.port_offset_unrotated(false, i);
            let lbl_pos = local_to_screen(offset.0 + 16.0, offset.1);
            let lbl = comp.kind.port_label_in(i);
            painter.text(
                lbl_pos,
                egui::Align2::LEFT_CENTER,
                lbl.to_uppercase(),
                Theme::font_bold(8.0 * zoom.clamp(0.6, 1.2)),
                Theme::ACCENT_PURPLE,
            );
        }

        // Output pin labels
        let out_count = comp.kind.output_count();
        for i in 0..out_count {
            let offset = comp.port_offset_unrotated(true, i);
            let lbl_pos = local_to_screen(offset.0 - 16.0, offset.1);
            let lbl = comp.kind.port_label_out(i);
            painter.text(
                lbl_pos,
                egui::Align2::RIGHT_CENTER,
                lbl.to_uppercase(),
                Theme::font_bold(8.0 * zoom.clamp(0.6, 1.2)),
                Theme::ACCENT_PINK,
            );
        }
    }
}

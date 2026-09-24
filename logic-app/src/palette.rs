use crate::theme::Theme;
use eframe::egui::{self, CornerRadius, Sense, Stroke, Ui, Vec2};
use logic_core::{Circuit, GateKind};

pub struct Palette;

impl Palette {
    pub const ALL_GATES: &'static [GateKind] = &[
        GateKind::And,
        GateKind::Or,
        GateKind::Not,
        GateKind::Nand,
        GateKind::Nor,
        GateKind::Xor,
        GateKind::Xnor,
    ];

    pub const ALL_IO: &'static [GateKind] =
        &[GateKind::ToggleSwitch, GateKind::Led, GateKind::Clock];

    pub const ALL_DISPLAYS: &'static [GateKind] = &[
        GateKind::BinaryDisplay4,
        GateKind::HexDisplay,
        GateKind::SevenSegment,
    ];

    pub const ALL_ARITHMETIC: &'static [GateKind] = &[
        GateKind::HalfAdder,
        GateKind::FullAdder,
        GateKind::HalfSubtractor,
        GateKind::FullSubtractor,
        GateKind::RippleCarryAdder4,
        GateKind::CarryLookaheadAdder4,
    ];

    pub const ALL_SEQUENTIAL: &'static [GateKind] = &[
        GateKind::SrLatch,
        GateKind::DLatch,
        GateKind::DFlipFlop,
        GateKind::JkFlipFlop,
        GateKind::TFlipFlop,
        GateKind::Register4,
        GateKind::ShiftRegister4,
        GateKind::Counter4,
        GateKind::ClockDivider,
    ];

    pub const ALL_ROUTING: &'static [GateKind] = &[
        GateKind::Mux2,
        GateKind::Mux4,
        GateKind::Demux2,
        GateKind::Demux4,
        GateKind::Encoder4to2,
        GateKind::Decoder2to4,
        GateKind::Comparator2,
    ];

    pub const ALL_COMPUTER: &'static [GateKind] =
        &[GateKind::Alu4, GateKind::Rom16x4, GateKind::Ram16x4];

    pub fn show(
        ui: &mut Ui,
        selected_for_placement: &mut Option<GateKind>,
        circuit: &Circuit,
    ) -> Option<GateKind> {
        let mut clicked_kind = None;

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Components")
                    .size(14.0)
                    .strong()
                    .color(Theme::TEXT_PRIMARY),
            );
        });
        ui.add_space(6.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 1. Logic Gates
            Self::category_section(
                ui,
                "Logic Gates",
                Self::ALL_GATES,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 2. Input / Output
            Self::category_section(
                ui,
                "Input / Output",
                Self::ALL_IO,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 3. Displays
            Self::category_section(
                ui,
                "Displays",
                Self::ALL_DISPLAYS,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 4. Arithmetic
            Self::category_section(
                ui,
                "Arithmetic",
                Self::ALL_ARITHMETIC,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 5. Sequential
            Self::category_section(
                ui,
                "Sequential",
                Self::ALL_SEQUENTIAL,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 6. Routing & Multiplexers
            Self::category_section(
                ui,
                "Routing & Multiplexers",
                Self::ALL_ROUTING,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 7. Computer Blocks
            Self::category_section(
                ui,
                "Computer Blocks",
                Self::ALL_COMPUTER,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 8. Custom Subcircuits
            if !circuit.subcircuits.is_empty() {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Custom Subcircuits")
                            .size(11.0)
                            .color(Theme::ACCENT_PINK)
                            .strong(),
                    );
                    let avail_w = ui.available_width();
                    if avail_w > 12.0 {
                        let y = ui.cursor().center().y;
                        let min_x = ui.cursor().min.x + 4.0;
                        let max_x = min_x + avail_w - 6.0;
                        ui.painter().line_segment(
                            [egui::pos2(min_x, y), egui::pos2(max_x, y)],
                            egui::Stroke::new(1.0, Theme::ACCENT_PINK.gamma_multiply(0.2)),
                        );
                    }
                });
                ui.add_space(3.0);

                for name in circuit.subcircuits.keys() {
                    let kind = GateKind::SubcircuitInstance(name.clone());
                    let is_active = *selected_for_placement == Some(kind.clone());
                    if Self::palette_item_button(ui, &kind, is_active) {
                        if is_active {
                            *selected_for_placement = None;
                        } else {
                            *selected_for_placement = Some(kind.clone());
                            clicked_kind = Some(kind);
                        }
                    }
                    ui.add_space(2.0);
                }
            }

            ui.add_space(16.0);
            if selected_for_placement.is_some() {
                ui.separator();
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Click on canvas to place.\nRight-click or Esc to cancel.")
                        .size(11.0)
                        .color(Theme::ACCENT_PINK),
                );
            }
        });

        clicked_kind
    }

    fn category_section(
        ui: &mut Ui,
        title: &str,
        items: &[GateKind],
        selected: &mut Option<GateKind>,
        clicked: &mut Option<GateKind>,
    ) {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(11.0)
                    .color(Theme::ACCENT_PURPLE)
                    .strong(),
            );
            let avail_w = ui.available_width();
            if avail_w > 12.0 {
                let y = ui.cursor().center().y;
                let min_x = ui.cursor().min.x + 4.0;
                let max_x = min_x + avail_w - 6.0;
                ui.painter().line_segment(
                    [egui::pos2(min_x, y), egui::pos2(max_x, y)],
                    egui::Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.2)),
                );
            }
        });
        ui.add_space(3.0);

        for kind in items {
            let is_active = *selected == Some(kind.clone());
            if Self::palette_item_button(ui, kind, is_active) {
                if is_active {
                    *selected = None;
                } else {
                    *selected = Some(kind.clone());
                    *clicked = Some(kind.clone());
                }
            }
            ui.add_space(2.0);
        }
    }

    fn palette_item_button(ui: &mut Ui, kind: &GateKind, is_active: bool) -> bool {
        let text = kind.display_name();

        let width = ui.available_width();
        let height = 28.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

        let bg = if is_active {
            Theme::ACCENT_PURPLE.gamma_multiply(0.4)
        } else if response.hovered() {
            Theme::BG_PANEL_RAISED
        } else {
            Theme::BG_PANEL
        };

        let stroke = if is_active {
            Stroke::new(1.5, Theme::ACCENT_PINK)
        } else if response.hovered() {
            Stroke::new(1.0, Theme::ACCENT_PINK.gamma_multiply(0.6))
        } else {
            Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.4))
        };

        ui.painter().rect(
            rect,
            CornerRadius::same(4),
            bg,
            stroke,
            egui::StrokeKind::Inside,
        );

        // Miniature vector preview icon box
        let icon_box_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 5.0, rect.min.y + 4.0),
            Vec2::new(26.0, 20.0),
        );
        let icon_box_bg = Theme::BG_CANVAS_DARK;
        let icon_box_stroke = if is_active {
            Stroke::new(1.0, Theme::ACCENT_PINK)
        } else if response.hovered() {
            Stroke::new(1.0, Theme::ACCENT_PURPLE)
        } else {
            Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.3))
        };
        ui.painter().rect(
            icon_box_rect,
            CornerRadius::same(3),
            icon_box_bg,
            icon_box_stroke,
            egui::StrokeKind::Inside,
        );

        let icon_stroke = if is_active {
            Stroke::new(1.4, Theme::ACCENT_PINK)
        } else if response.hovered() {
            Stroke::new(1.2, Theme::TEXT_PRIMARY)
        } else {
            Stroke::new(1.2, Theme::ACCENT_PURPLE)
        };

        Self::draw_mini_glyph(
            ui.painter(),
            kind,
            icon_box_rect.center(),
            icon_stroke,
            Theme::BG_CANVAS_DARK,
        );

        // Component name
        let text_pos = egui::pos2(rect.min.x + 36.0, rect.center().y);
        let text_color = if is_active {
            Theme::ACCENT_PINK
        } else if response.hovered() {
            Theme::TEXT_PRIMARY
        } else {
            Theme::TEXT_PRIMARY.gamma_multiply(0.9)
        };
        ui.painter().text(
            text_pos,
            egui::Align2::LEFT_CENTER,
            &text,
            egui::FontId::proportional(11.5),
            text_color,
        );

        response.clicked()
    }

    fn draw_mini_glyph(
        painter: &egui::Painter,
        kind: &GateKind,
        c: egui::Pos2,
        stroke: Stroke,
        fill_bg: egui::Color32,
    ) {
        match kind {
            GateKind::And => {
                let mut pts = Vec::with_capacity(12);
                pts.push(egui::pos2(c.x - 7.0, c.y - 5.5));
                pts.push(egui::pos2(c.x - 1.0, c.y - 5.5));
                for i in 0..=6 {
                    let th = -std::f32::consts::FRAC_PI_2 + (i as f32 / 6.0) * std::f32::consts::PI;
                    pts.push(egui::pos2(c.x - 1.0 + 7.5 * th.cos(), c.y + 5.5 * th.sin()));
                }
                pts.push(egui::pos2(c.x - 7.0, c.y + 5.5));
                painter.add(egui::epaint::PathShape::convex_polygon(
                    pts, fill_bg, stroke,
                ));
            }
            GateKind::Nand => {
                let gx = c.x - 2.0;
                let mut pts = Vec::with_capacity(12);
                pts.push(egui::pos2(gx - 6.0, c.y - 5.5));
                pts.push(egui::pos2(gx - 1.0, c.y - 5.5));
                for i in 0..=6 {
                    let th = -std::f32::consts::FRAC_PI_2 + (i as f32 / 6.0) * std::f32::consts::PI;
                    pts.push(egui::pos2(gx - 1.0 + 6.5 * th.cos(), c.y + 5.5 * th.sin()));
                }
                pts.push(egui::pos2(gx - 6.0, c.y + 5.5));
                painter.add(egui::epaint::PathShape::convex_polygon(
                    pts, fill_bg, stroke,
                ));
                painter.circle(egui::pos2(c.x + 6.5, c.y), 1.8, fill_bg, stroke);
            }
            GateKind::Or => {
                let mut pts = Vec::with_capacity(18);
                for i in 0..=5 {
                    let t = i as f32 / 5.0;
                    pts.push(egui::pos2(
                        (c.x - 7.0) + 14.5 * t,
                        (c.y - 5.5) + 5.5 * (t * t),
                    ));
                }
                for i in 0..=5 {
                    let t = i as f32 / 5.0;
                    pts.push(egui::pos2(
                        (c.x + 7.5) - 14.5 * t,
                        c.y + 5.5 * (1.0 - (1.0 - t).powi(2)),
                    ));
                }
                for i in 0..=4 {
                    let t = i as f32 / 4.0;
                    let y = (c.y + 5.5) - 11.0 * t;
                    let x = (c.x - 7.0) + 2.5 * (1.0 - (2.0 * t - 1.0).powi(2));
                    pts.push(egui::pos2(x, y));
                }
                painter.add(egui::epaint::PathShape::closed_line(pts, stroke));
            }
            GateKind::Nor => {
                let gx = c.x - 2.0;
                let mut pts = Vec::with_capacity(18);
                for i in 0..=5 {
                    let t = i as f32 / 5.0;
                    pts.push(egui::pos2(
                        (gx - 6.0) + 12.5 * t,
                        (c.y - 5.5) + 5.5 * (t * t),
                    ));
                }
                for i in 0..=5 {
                    let t = i as f32 / 5.0;
                    pts.push(egui::pos2(
                        (gx + 6.5) - 12.5 * t,
                        c.y + 5.5 * (1.0 - (1.0 - t).powi(2)),
                    ));
                }
                for i in 0..=4 {
                    let t = i as f32 / 4.0;
                    let y = (c.y + 5.5) - 11.0 * t;
                    let x = (gx - 6.0) + 2.5 * (1.0 - (2.0 * t - 1.0).powi(2));
                    pts.push(egui::pos2(x, y));
                }
                painter.add(egui::epaint::PathShape::closed_line(pts, stroke));
                painter.circle(egui::pos2(c.x + 6.5, c.y), 1.8, fill_bg, stroke);
            }
            GateKind::Xor => {
                let mut pts = Vec::with_capacity(18);
                for i in 0..=5 {
                    let t = i as f32 / 5.0;
                    pts.push(egui::pos2(
                        (c.x - 5.5) + 13.0 * t,
                        (c.y - 5.5) + 5.5 * (t * t),
                    ));
                }
                for i in 0..=5 {
                    let t = i as f32 / 5.0;
                    pts.push(egui::pos2(
                        (c.x + 7.5) - 13.0 * t,
                        c.y + 5.5 * (1.0 - (1.0 - t).powi(2)),
                    ));
                }
                for i in 0..=4 {
                    let t = i as f32 / 4.0;
                    let y = (c.y + 5.5) - 11.0 * t;
                    let x = (c.x - 5.5) + 2.5 * (1.0 - (2.0 * t - 1.0).powi(2));
                    pts.push(egui::pos2(x, y));
                }
                painter.add(egui::epaint::PathShape::closed_line(pts, stroke));
                // Extra rear arc
                let mut arc = Vec::with_capacity(5);
                for i in 0..=4 {
                    let t = i as f32 / 4.0;
                    let y = (c.y - 5.5) + 11.0 * t;
                    let x = (c.x - 8.5) + 2.5 * (1.0 - (2.0 * t - 1.0).powi(2));
                    arc.push(egui::pos2(x, y));
                }
                painter.add(egui::epaint::PathShape::line(arc, stroke));
            }
            GateKind::Xnor => {
                let gx = c.x - 2.0;
                let mut pts = Vec::with_capacity(18);
                for i in 0..=5 {
                    let t = i as f32 / 5.0;
                    pts.push(egui::pos2(
                        (gx - 5.0) + 11.5 * t,
                        (c.y - 5.5) + 5.5 * (t * t),
                    ));
                }
                for i in 0..=5 {
                    let t = i as f32 / 5.0;
                    pts.push(egui::pos2(
                        (gx + 6.5) - 11.5 * t,
                        c.y + 5.5 * (1.0 - (1.0 - t).powi(2)),
                    ));
                }
                for i in 0..=4 {
                    let t = i as f32 / 4.0;
                    let y = (c.y + 5.5) - 11.0 * t;
                    let x = (gx - 5.0) + 2.5 * (1.0 - (2.0 * t - 1.0).powi(2));
                    pts.push(egui::pos2(x, y));
                }
                painter.add(egui::epaint::PathShape::closed_line(pts, stroke));
                // Extra rear arc
                let mut arc = Vec::with_capacity(5);
                for i in 0..=4 {
                    let t = i as f32 / 4.0;
                    let y = (c.y - 5.5) + 11.0 * t;
                    let x = (gx - 7.5) + 2.5 * (1.0 - (2.0 * t - 1.0).powi(2));
                    arc.push(egui::pos2(x, y));
                }
                painter.add(egui::epaint::PathShape::line(arc, stroke));
                painter.circle(egui::pos2(c.x + 6.5, c.y), 1.8, fill_bg, stroke);
            }
            GateKind::Not => {
                let pts = vec![
                    egui::pos2(c.x - 6.5, c.y - 5.5),
                    egui::pos2(c.x + 3.0, c.y),
                    egui::pos2(c.x - 6.5, c.y + 5.5),
                ];
                painter.add(egui::epaint::PathShape::convex_polygon(
                    pts, fill_bg, stroke,
                ));
                painter.circle(egui::pos2(c.x + 5.5, c.y), 1.8, fill_bg, stroke);
            }
            GateKind::ToggleSwitch => {
                let plate = egui::Rect::from_center_size(c, Vec2::new(14.0, 11.0));
                painter.rect_filled(plate, CornerRadius::same(2), fill_bg);
                painter.rect_stroke(
                    plate,
                    CornerRadius::same(2),
                    stroke,
                    egui::StrokeKind::Inside,
                );
                // Slanted lever arm
                painter.line_segment(
                    [
                        egui::pos2(c.x - 1.0, c.y + 2.0),
                        egui::pos2(c.x + 3.5, c.y - 3.0),
                    ],
                    Stroke::new(1.4, stroke.color),
                );
                painter.circle_filled(egui::pos2(c.x + 3.5, c.y - 3.0), 1.8, stroke.color);
                // Status dot
                painter.circle_filled(egui::pos2(c.x - 4.0, c.y), 1.4, Theme::ACCENT_PINK);
            }
            GateKind::Led => {
                painter.circle(c, 5.0, fill_bg, stroke);
                painter.circle_filled(c, 2.5, Theme::ACCENT_PINK);
                let ray_s = Stroke::new(1.0, Theme::ACCENT_PINK.gamma_multiply(0.8));
                painter.line_segment(
                    [
                        egui::pos2(c.x + 4.0, c.y - 4.0),
                        egui::pos2(c.x + 7.0, c.y - 7.0),
                    ],
                    ray_s,
                );
                painter.line_segment(
                    [
                        egui::pos2(c.x + 5.5, c.y - 1.5),
                        egui::pos2(c.x + 8.5, c.y - 4.5),
                    ],
                    ray_s,
                );
            }
            GateKind::Clock => {
                let pts = vec![
                    egui::pos2(c.x - 8.0, c.y + 3.5),
                    egui::pos2(c.x - 4.0, c.y + 3.5),
                    egui::pos2(c.x - 4.0, c.y - 3.5),
                    egui::pos2(c.x + 1.0, c.y - 3.5),
                    egui::pos2(c.x + 1.0, c.y + 3.5),
                    egui::pos2(c.x + 5.0, c.y + 3.5),
                    egui::pos2(c.x + 5.0, c.y - 3.5),
                    egui::pos2(c.x + 8.0, c.y - 3.5),
                ];
                painter.add(egui::epaint::PathShape::line(pts, stroke));
            }
            GateKind::BinaryDisplay4 | GateKind::HexDisplay | GateKind::SevenSegment => {
                let r = egui::Rect::from_center_size(c, Vec2::new(13.0, 13.0));
                painter.rect_filled(r, CornerRadius::same(2), fill_bg);
                painter.rect_stroke(
                    r,
                    CornerRadius::same(2),
                    Stroke::new(1.0, stroke.color.gamma_multiply(0.4)),
                    egui::StrokeKind::Inside,
                );
                let s = Stroke::new(1.2, stroke.color);
                painter.line_segment(
                    [
                        egui::pos2(c.x - 2.5, c.y - 4.5),
                        egui::pos2(c.x + 2.5, c.y - 4.5),
                    ],
                    s,
                );
                painter.line_segment(
                    [egui::pos2(c.x - 2.5, c.y - 4.5), egui::pos2(c.x - 2.5, c.y)],
                    s,
                );
                painter.line_segment(
                    [egui::pos2(c.x + 2.5, c.y - 4.5), egui::pos2(c.x + 2.5, c.y)],
                    s,
                );
                painter.line_segment([egui::pos2(c.x - 2.5, c.y), egui::pos2(c.x + 2.5, c.y)], s);
                painter.line_segment(
                    [egui::pos2(c.x - 2.5, c.y), egui::pos2(c.x - 2.5, c.y + 4.5)],
                    s,
                );
                painter.line_segment(
                    [egui::pos2(c.x + 2.5, c.y), egui::pos2(c.x + 2.5, c.y + 4.5)],
                    s,
                );
                painter.line_segment(
                    [
                        egui::pos2(c.x - 2.5, c.y + 4.5),
                        egui::pos2(c.x + 2.5, c.y + 4.5),
                    ],
                    s,
                );
            }
            GateKind::HalfAdder | GateKind::FullAdder => {
                let r = egui::Rect::from_center_size(c, Vec2::new(14.0, 12.0));
                painter.rect_filled(r, CornerRadius::same(2), fill_bg);
                painter.rect_stroke(r, CornerRadius::same(2), stroke, egui::StrokeKind::Inside);
                let s = Stroke::new(1.4, stroke.color);
                painter.line_segment([egui::pos2(c.x - 3.5, c.y), egui::pos2(c.x + 3.5, c.y)], s);
                painter.line_segment([egui::pos2(c.x, c.y - 3.5), egui::pos2(c.x, c.y + 3.5)], s);
            }
            GateKind::HalfSubtractor | GateKind::FullSubtractor => {
                let r = egui::Rect::from_center_size(c, Vec2::new(14.0, 12.0));
                painter.rect_filled(r, CornerRadius::same(2), fill_bg);
                painter.rect_stroke(r, CornerRadius::same(2), stroke, egui::StrokeKind::Inside);
                let s = Stroke::new(1.4, stroke.color);
                painter.line_segment([egui::pos2(c.x - 3.5, c.y), egui::pos2(c.x + 3.5, c.y)], s);
            }
            GateKind::RippleCarryAdder4 | GateKind::CarryLookaheadAdder4 => {
                let r = egui::Rect::from_center_size(c, Vec2::new(15.0, 13.0));
                painter.rect_filled(r, CornerRadius::same(2), fill_bg);
                painter.rect_stroke(r, CornerRadius::same(2), stroke, egui::StrokeKind::Inside);
                let s = Stroke::new(1.3, stroke.color);
                painter.line_segment([egui::pos2(c.x - 4.5, c.y), egui::pos2(c.x - 0.5, c.y)], s);
                painter.line_segment(
                    [
                        egui::pos2(c.x - 2.5, c.y - 2.0),
                        egui::pos2(c.x - 2.5, c.y + 2.0),
                    ],
                    s,
                );
                painter.text(
                    egui::pos2(c.x + 2.5, c.y),
                    egui::Align2::CENTER_CENTER,
                    "4",
                    egui::FontId::monospace(8.0),
                    stroke.color,
                );
            }
            GateKind::SrLatch
            | GateKind::DLatch
            | GateKind::DFlipFlop
            | GateKind::JkFlipFlop
            | GateKind::TFlipFlop
            | GateKind::Register4
            | GateKind::ShiftRegister4
            | GateKind::Counter4
            | GateKind::ClockDivider => {
                let r = egui::Rect::from_center_size(c, Vec2::new(14.0, 13.0));
                painter.rect_filled(r, CornerRadius::same(2), fill_bg);
                painter.rect_stroke(r, CornerRadius::same(2), stroke, egui::StrokeKind::Inside);
                // Clock chevron on left edge
                let s = Stroke::new(1.2, stroke.color);
                painter.line_segment(
                    [
                        egui::pos2(r.min.x, c.y - 3.0),
                        egui::pos2(r.min.x + 3.0, c.y),
                    ],
                    s,
                );
                painter.line_segment(
                    [
                        egui::pos2(r.min.x + 3.0, c.y),
                        egui::pos2(r.min.x, c.y + 3.0),
                    ],
                    s,
                );
                // Output pin stub on right
                painter.line_segment(
                    [
                        egui::pos2(r.max.x - 1.0, c.y - 2.0),
                        egui::pos2(r.max.x + 2.0, c.y - 2.0),
                    ],
                    s,
                );
            }
            GateKind::Mux2 | GateKind::Mux4 => {
                let pts = vec![
                    egui::pos2(c.x - 6.0, c.y - 5.5),
                    egui::pos2(c.x + 5.0, c.y - 3.0),
                    egui::pos2(c.x + 5.0, c.y + 3.0),
                    egui::pos2(c.x - 6.0, c.y + 5.5),
                ];
                painter.add(egui::epaint::PathShape::convex_polygon(
                    pts, fill_bg, stroke,
                ));
            }
            GateKind::Demux2 | GateKind::Demux4 => {
                let pts = vec![
                    egui::pos2(c.x - 5.0, c.y - 3.0),
                    egui::pos2(c.x + 6.0, c.y - 5.5),
                    egui::pos2(c.x + 6.0, c.y + 5.5),
                    egui::pos2(c.x - 5.0, c.y + 3.0),
                ];
                painter.add(egui::epaint::PathShape::convex_polygon(
                    pts, fill_bg, stroke,
                ));
            }
            GateKind::Encoder4to2 => {
                let r = egui::Rect::from_center_size(c, Vec2::new(14.0, 12.0));
                painter.rect_filled(r, CornerRadius::same(2), fill_bg);
                painter.rect_stroke(r, CornerRadius::same(2), stroke, egui::StrokeKind::Inside);
                let s = Stroke::new(1.2, stroke.color);
                painter.line_segment(
                    [egui::pos2(c.x - 3.5, c.y - 2.5), egui::pos2(c.x + 2.5, c.y)],
                    s,
                );
                painter.line_segment(
                    [egui::pos2(c.x - 3.5, c.y + 2.5), egui::pos2(c.x + 2.5, c.y)],
                    s,
                );
            }
            GateKind::Decoder2to4 => {
                let r = egui::Rect::from_center_size(c, Vec2::new(14.0, 12.0));
                painter.rect_filled(r, CornerRadius::same(2), fill_bg);
                painter.rect_stroke(r, CornerRadius::same(2), stroke, egui::StrokeKind::Inside);
                let s = Stroke::new(1.2, stroke.color);
                painter.line_segment(
                    [egui::pos2(c.x - 2.5, c.y), egui::pos2(c.x + 3.5, c.y - 2.5)],
                    s,
                );
                painter.line_segment(
                    [egui::pos2(c.x - 2.5, c.y), egui::pos2(c.x + 3.5, c.y + 2.5)],
                    s,
                );
            }
            GateKind::Comparator2 => {
                let r = egui::Rect::from_center_size(c, Vec2::new(14.0, 12.0));
                painter.rect_filled(r, CornerRadius::same(2), fill_bg);
                painter.rect_stroke(r, CornerRadius::same(2), stroke, egui::StrokeKind::Inside);
                let s = Stroke::new(1.3, stroke.color);
                painter.line_segment(
                    [
                        egui::pos2(c.x - 3.5, c.y - 1.5),
                        egui::pos2(c.x + 3.5, c.y - 1.5),
                    ],
                    s,
                );
                painter.line_segment(
                    [
                        egui::pos2(c.x - 3.5, c.y + 1.5),
                        egui::pos2(c.x + 3.5, c.y + 1.5),
                    ],
                    s,
                );
            }
            GateKind::Alu4 => {
                let pts = vec![
                    egui::pos2(c.x - 6.5, c.y - 5.5),
                    egui::pos2(c.x - 2.0, c.y - 5.5),
                    egui::pos2(c.x, c.y - 2.0),
                    egui::pos2(c.x + 2.0, c.y - 5.5),
                    egui::pos2(c.x + 6.5, c.y - 5.5),
                    egui::pos2(c.x + 4.5, c.y + 5.5),
                    egui::pos2(c.x - 4.5, c.y + 5.5),
                ];
                painter.add(egui::epaint::PathShape::closed_line(pts, stroke));
            }
            GateKind::Rom16x4 | GateKind::Ram16x4 => {
                let r = egui::Rect::from_center_size(c, Vec2::new(15.0, 13.0));
                painter.rect_filled(r, CornerRadius::same(2), fill_bg);
                painter.rect_stroke(r, CornerRadius::same(2), stroke, egui::StrokeKind::Inside);
                let txt = if matches!(kind, GateKind::Rom16x4) {
                    "ROM"
                } else {
                    "RAM"
                };
                painter.text(
                    c,
                    egui::Align2::CENTER_CENTER,
                    txt,
                    egui::FontId::monospace(7.0),
                    stroke.color,
                );
            }
            GateKind::SubcircuitInstance(_) => {
                let r = egui::Rect::from_center_size(c, Vec2::new(14.0, 12.0));
                painter.rect_filled(r, CornerRadius::same(2), fill_bg);
                painter.rect_stroke(r, CornerRadius::same(2), stroke, egui::StrokeKind::Inside);
                let s = Stroke::new(1.0, stroke.color);
                painter.line_segment(
                    [
                        egui::pos2(r.min.x - 2.0, c.y - 2.5),
                        egui::pos2(r.min.x, c.y - 2.5),
                    ],
                    s,
                );
                painter.line_segment(
                    [
                        egui::pos2(r.min.x - 2.0, c.y + 2.5),
                        egui::pos2(r.min.x, c.y + 2.5),
                    ],
                    s,
                );
                painter.line_segment(
                    [
                        egui::pos2(r.max.x, c.y - 2.5),
                        egui::pos2(r.max.x + 2.0, c.y - 2.5),
                    ],
                    s,
                );
                painter.line_segment(
                    [
                        egui::pos2(r.max.x, c.y + 2.5),
                        egui::pos2(r.max.x + 2.0, c.y + 2.5),
                    ],
                    s,
                );
                painter.circle_filled(c, 1.6, stroke.color);
            }
        }
    }
}

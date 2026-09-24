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
            ui.visuals_mut().override_text_color = Some(Theme::TEXT_PRIMARY);
            ui.heading("Components");
        });
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 1. Logic Gates
            Self::category_section(
                ui,
                "LOGIC GATES",
                Self::ALL_GATES,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 2. Input / Output
            Self::category_section(
                ui,
                "INPUT / OUTPUT",
                Self::ALL_IO,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 3. Displays
            Self::category_section(
                ui,
                "DISPLAYS",
                Self::ALL_DISPLAYS,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 4. Arithmetic
            Self::category_section(
                ui,
                "ARITHMETIC",
                Self::ALL_ARITHMETIC,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 5. Sequential
            Self::category_section(
                ui,
                "SEQUENTIAL",
                Self::ALL_SEQUENTIAL,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 6. Routing & Multiplexers
            Self::category_section(
                ui,
                "ROUTING & MULTIPLEXERS",
                Self::ALL_ROUTING,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 7. Computer Blocks
            Self::category_section(
                ui,
                "COMPUTER BLOCKS",
                Self::ALL_COMPUTER,
                selected_for_placement,
                &mut clicked_kind,
            );

            // 8. Custom Subcircuits
            if !circuit.subcircuits.is_empty() {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("SUBCIRCUITS")
                        .size(11.0)
                        .color(Theme::ACCENT_PINK),
                );
                ui.add_space(4.0);

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
                    ui.add_space(3.0);
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
        ui.label(
            egui::RichText::new(title)
                .size(10.0)
                .color(Theme::ACCENT_PURPLE),
        );
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
        let short = kind.short_code();

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

        // Badge with short name
        let badge_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 4.0, rect.min.y + 5.0),
            Vec2::new(38.0, 18.0),
        );
        let badge_bg = if is_active {
            Theme::ACCENT_PINK
        } else {
            Theme::BG_PANEL_RAISED
        };
        let badge_text_color = if is_active {
            Theme::BG_PANEL
        } else {
            Theme::TEXT_PRIMARY
        };
        ui.painter()
            .rect_filled(badge_rect, CornerRadius::same(3), badge_bg);
        ui.painter().text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            &short,
            egui::FontId::monospace(9.0),
            badge_text_color,
        );

        // Component name
        let text_pos = egui::pos2(rect.min.x + 46.0, rect.center().y);
        let text_color = if is_active {
            Theme::ACCENT_PINK
        } else {
            Theme::TEXT_PRIMARY
        };
        ui.painter().text(
            text_pos,
            egui::Align2::LEFT_CENTER,
            &text,
            egui::FontId::proportional(11.0),
            text_color,
        );

        response.clicked()
    }
}

use crate::theme::Theme;
use eframe::egui::{self, CornerRadius, Rect, RichText, Sense, Stroke, Ui, Vec2, pos2};
use logic_core::{Circuit, ComponentId, GateKind, Rotation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftPanelTab {
    Components,
    Properties,
}

fn draw_triangle_arrow_button(
    ui: &mut egui::Ui,
    pointing_down: bool,
    tooltip: &str,
) -> bool {
    let size = egui::vec2(24.0, 22.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hovered = resp.hovered();

    let bg_color = if hovered {
        Theme::ACCENT_PURPLE.gamma_multiply(0.6)
    } else {
        Theme::ACCENT_PURPLE.gamma_multiply(0.35)
    };
    let stroke_color = if hovered {
        Theme::ACCENT_PINK
    } else {
        Theme::ACCENT_PINK.gamma_multiply(0.8)
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(4),
        bg_color,
        Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );

    let c = rect.center();
    let color = if hovered {
        egui::Color32::WHITE
    } else {
        Theme::ACCENT_PINK
    };

    if pointing_down {
        let p1 = pos2(c.x - 5.0, c.y - 2.5);
        let p2 = pos2(c.x + 5.0, c.y - 2.5);
        let p3 = pos2(c.x, c.y + 3.5);
        ui.painter().add(egui::epaint::PathShape::convex_polygon(
            vec![p1, p2, p3],
            color,
            Stroke::NONE,
        ));
    } else {
        let p1 = pos2(c.x - 5.0, c.y + 2.5);
        let p2 = pos2(c.x + 5.0, c.y + 2.5);
        let p3 = pos2(c.x, c.y - 3.5);
        ui.painter().add(egui::epaint::PathShape::convex_polygon(
            vec![p1, p2, p3],
            color,
            Stroke::NONE,
        ));
    }

    resp.on_hover_text(tooltip).clicked()
}

fn draw_close_button(ui: &mut egui::Ui, tooltip: &str) -> bool {
    let size = egui::vec2(22.0, 22.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hovered = resp.hovered();

    let bg_color = if hovered {
        Theme::ACCENT_RED.gamma_multiply(0.4)
    } else {
        Theme::BG_PANEL_RAISED
    };
    let stroke_color = if hovered {
        Theme::ACCENT_RED
    } else {
        Theme::ACCENT_PURPLE.gamma_multiply(0.5)
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(4),
        bg_color,
        Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );

    let c = rect.center();
    let cross_color = if hovered {
        egui::Color32::WHITE
    } else {
        Theme::TEXT_MUTED
    };
    let s = Stroke::new(1.5, cross_color);
    let d = 4.0;
    ui.painter().line_segment([pos2(c.x - d, c.y - d), pos2(c.x + d, c.y + d)], s);
    ui.painter().line_segment([pos2(c.x + d, c.y - d), pos2(c.x - d, c.y + d)], s);

    resp.on_hover_text(tooltip).clicked()
}

pub struct PropertiesUi;

impl PropertiesUi {
    /// Renders the Properties tab inside a panel (e.g. left sidebar tab or right sidebar)
    pub fn show_panel(
        ui: &mut Ui,
        circuit: &mut Circuit,
        selected_id: Option<ComponentId>,
        is_expanded: &mut bool,
    ) -> bool {
        let mut mutated = false;

        ui.add_space(6.0);

        let Some(comp_id) = selected_id else {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("PROPERTIES")
                        .font(Theme::font_bold(14.5))
                        .color(Theme::TEXT_PRIMARY),
                );
            });
            ui.add_space(8.0);
            Self::show_empty_placeholder(ui, circuit);
            return false;
        };

        let comp = match circuit.components.get(comp_id) {
            Some(c) => c.clone(),
            None => {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("PROPERTIES")
                            .font(Theme::font_bold(14.5))
                            .color(Theme::TEXT_PRIMARY),
                    );
                });
                ui.add_space(8.0);
                Self::show_empty_placeholder(ui, circuit);
                return false;
            }
        };

        // Collapsible header bar with arrow toggle
        ui.horizontal(|ui| {
            let arrow_tooltip = if *is_expanded {
                "Collapse properties"
            } else {
                "Down the arrow to make menu open fully"
            };

            if draw_triangle_arrow_button(ui, !*is_expanded, arrow_tooltip) {
                *is_expanded = !*is_expanded;
            }

            ui.label(
                RichText::new("PROPERTIES")
                    .font(Theme::font_bold(14.5))
                    .color(Theme::TEXT_PRIMARY),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !*is_expanded {
                    if ui
                        .button(
                            RichText::new("Open")
                                .font(Theme::font_bold(11.0))
                                .color(Theme::ACCENT_PINK),
                        )
                        .on_hover_text("Make the menu open fully")
                        .clicked()
                    {
                        *is_expanded = true;
                    }
                } else if ui
                    .button(
                        RichText::new("Collapse")
                            .font(Theme::font_bold(11.0))
                            .color(Theme::TEXT_MUTED),
                    )
                    .on_hover_text("Collapse properties")
                    .clicked()
                {
                    *is_expanded = false;
                }
            });
        });

        ui.add_space(6.0);

        if !*is_expanded {
            // Collapsed banner card in left panel
            let (rect, resp) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 64.0), Sense::click());
            let is_hov = resp.hovered();
            ui.painter().rect(
                rect,
                CornerRadius::same(6),
                if is_hov {
                    Theme::BG_PANEL_RAISED
                } else {
                    Theme::BG_PANEL
                },
                Stroke::new(
                    1.0,
                    if is_hov {
                        Theme::ACCENT_PINK
                    } else {
                        Theme::ACCENT_PURPLE.gamma_multiply(0.4)
                    },
                ),
                egui::StrokeKind::Inside,
            );

            let badge_rect = Rect::from_min_size(
                pos2(rect.min.x + 8.0, rect.min.y + 8.0),
                Vec2::new(32.0, 22.0),
            );
            ui.painter()
                .rect_filled(badge_rect, CornerRadius::same(3), Theme::BG_CANVAS_DARK);
            ui.painter().rect_stroke(
                badge_rect,
                CornerRadius::same(3),
                Stroke::new(1.0, Theme::ACCENT_PINK),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                comp.kind.short_code(),
                Theme::font_bold(9.0),
                Theme::ACCENT_PINK,
            );

            let title = if comp.label.is_empty() {
                comp.kind.display_name().to_string()
            } else {
                format!("{} ({})", comp.kind.display_name(), comp.label)
            };
            ui.painter().text(
                pos2(rect.min.x + 48.0, rect.min.y + 19.0),
                egui::Align2::LEFT_CENTER,
                title,
                Theme::font_bold(13.0),
                Theme::TEXT_PRIMARY,
            );

            ui.painter().text(
                pos2(rect.min.x + 8.0, rect.min.y + 45.0),
                egui::Align2::LEFT_CENTER,
                "Click arrow to make menu open fully",
                Theme::font_regular(11.0),
                Theme::ACCENT_PINK,
            );

            if resp.clicked() {
                *is_expanded = true;
            }
            return false;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            mutated |= Self::render_component_properties(ui, circuit, comp_id);
        });

        mutated
    }

    /// Renders a floating glass inspector window in the top-right / middle of the canvas
    pub fn show_floating_window(
        ctx: &egui::Context,
        circuit: &mut Circuit,
        selected_id: Option<ComponentId>,
        is_open: &mut bool,
        is_expanded: &mut bool,
    ) -> bool {
        if !*is_open {
            return false;
        }

        let Some(comp_id) = selected_id else {
            return false;
        };

        let comp = match circuit.components.get(comp_id) {
            Some(c) => c.clone(),
            None => return false,
        };

        let mut mutated = false;
        let modal_frame = Theme::glass_modal();

        if !*is_expanded {
            // Collapsed manner: sleek compact floating bar with down arrow
            egui::Window::new("PROPERTIES_COLLAPSED_WINDOW")
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .frame(modal_frame)
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-20.0, 60.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        // Down arrow button to open fully
                        if draw_triangle_arrow_button(
                            ui,
                            true,
                            "Down the arrow to make the menu open fully",
                        ) {
                            *is_expanded = true;
                        }

                        // Badge
                        let (badge_rect, _) =
                            ui.allocate_exact_size(Vec2::new(30.0, 22.0), Sense::hover());
                        ui.painter().rect_filled(
                            badge_rect,
                            CornerRadius::same(3),
                            Theme::BG_CANVAS_DARK,
                        );
                        ui.painter().rect_stroke(
                            badge_rect,
                            CornerRadius::same(3),
                            Stroke::new(1.0, Theme::ACCENT_PINK),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            comp.kind.short_code(),
                            Theme::font_bold(9.0),
                            Theme::ACCENT_PINK,
                        );

                        // Component title
                        let display_title = if comp.label.is_empty() {
                            format!("{} Properties", comp.kind.display_name())
                        } else {
                            format!("{} ({})", comp.kind.display_name(), comp.label)
                        };

                        if ui
                            .selectable_label(
                                false,
                                RichText::new(display_title)
                                    .font(Theme::font_bold(13.0))
                                    .color(Theme::TEXT_PRIMARY),
                            )
                            .on_hover_text("Click to make menu open fully")
                            .clicked()
                        {
                            *is_expanded = true;
                        }

                        ui.add_space(4.0);

                        if ui
                            .button(
                                RichText::new("Open")
                                    .font(Theme::font_bold(11.0))
                                    .color(Theme::ACCENT_PINK),
                            )
                            .on_hover_text("Down the arrow and make the menu open fully")
                            .clicked()
                        {
                            *is_expanded = true;
                        }

                        if draw_close_button(ui, "Close properties") {
                            *is_open = false;
                        }
                    });
                });

            return false;
        }

        // Expanded manner: full properties menu
        egui::Window::new(
            RichText::new("COMPONENT PROPERTIES")
                .font(Theme::font_bold(15.0))
                .color(Theme::TEXT_PRIMARY),
        )
        .open(is_open)
        .frame(modal_frame)
        .resizable(true)
        .default_size(Vec2::new(330.0, 440.0))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-20.0, 60.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if draw_triangle_arrow_button(ui, false, "Collapse properties menu") {
                    *is_expanded = false;
                }
                if ui
                    .button(
                        RichText::new("Collapse Menu")
                            .font(Theme::font_bold(11.5))
                            .color(Theme::ACCENT_PINK),
                    )
                    .on_hover_text("Collapse properties menu")
                    .clicked()
                {
                    *is_expanded = false;
                }
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                mutated |= Self::render_component_properties(ui, circuit, comp_id);
            });
        });

        mutated
    }

    fn show_empty_placeholder(ui: &mut Ui, circuit: &Circuit) {
        ui.add_space(16.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("CIRCUIT OVERVIEW")
                    .font(Theme::font_bold(12.0))
                    .color(Theme::ACCENT_PURPLE),
            );
            ui.add_space(8.0);

            let box_rect = ui.available_rect_before_wrap();
            let h = 110.0;
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(box_rect.width().max(160.0), h), Sense::hover());
            ui.painter().rect(
                rect,
                CornerRadius::same(6),
                Theme::BG_PANEL,
                Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.3)),
                egui::StrokeKind::Inside,
            );

            let comp_count = circuit.components.len();
            let net_count = circuit.nets.len();
            let sub_count = circuit.subcircuits.len();

            let text_start_y = rect.min.y + 14.0;
            let center_x = rect.center().x;

            ui.painter().text(
                egui::pos2(center_x, text_start_y),
                egui::Align2::CENTER_TOP,
                format!("Components: {}", comp_count),
                Theme::font_regular(12.0),
                Theme::TEXT_PRIMARY,
            );
            ui.painter().text(
                egui::pos2(center_x, text_start_y + 24.0),
                egui::Align2::CENTER_TOP,
                format!("Active Nets: {}", net_count),
                Theme::font_regular(12.0),
                Theme::TEXT_PRIMARY,
            );
            ui.painter().text(
                egui::pos2(center_x, text_start_y + 48.0),
                egui::Align2::CENTER_TOP,
                format!("Subcircuits: {}", sub_count),
                Theme::font_regular(12.0),
                Theme::TEXT_PRIMARY,
            );

            ui.add_space(14.0);
            ui.label(
                RichText::new(
                    "Select any component on the canvas to inspect and edit its properties.",
                )
                .font(Theme::font_regular(11.0))
                .color(Theme::TEXT_MUTED),
            );
        });
    }

    fn render_component_properties(
        ui: &mut Ui,
        circuit: &mut Circuit,
        comp_id: ComponentId,
    ) -> bool {
        let mut mutated = false;

        let comp = match circuit.components.get(comp_id) {
            Some(c) => c.clone(),
            None => return false,
        };

        // 1. Header Card: Type & Short Code
        ui.horizontal(|ui| {
            let badge_rect = ui
                .allocate_exact_size(Vec2::new(32.0, 22.0), Sense::hover())
                .0;
            ui.painter()
                .rect_filled(badge_rect, CornerRadius::same(3), Theme::BG_CANVAS_DARK);
            ui.painter().rect_stroke(
                badge_rect,
                CornerRadius::same(3),
                Stroke::new(1.0, Theme::ACCENT_PINK),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                comp.kind.short_code(),
                Theme::font_bold(9.0),
                Theme::ACCENT_PINK,
            );

            ui.vertical(|ui| {
                ui.label(
                    RichText::new(comp.kind.display_name())
                        .font(Theme::font_bold(14.0))
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new(format!("Pos: ({:.0}, {:.0})", comp.pos.0, comp.pos.1))
                        .font(Theme::font_regular(11.0))
                        .color(Theme::TEXT_MUTED),
                );
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        // 2. Custom Label
        ui.label(
            RichText::new("COMPONENT NAME")
                .font(Theme::font_bold(11.0))
                .color(Theme::ACCENT_PURPLE),
        );
        ui.add_space(2.0);
        let mut label = comp.label.clone();
        if ui
            .add(egui::TextEdit::singleline(&mut label).hint_text("Enter name..."))
            .changed()
        {
            circuit.set_component_label(comp_id, label);
            mutated = true;
        }

        ui.add_space(10.0);

        // 3. Direction / Orientation (East, South, West, North)
        ui.label(
            RichText::new("DIRECTION / ORIENTATION")
                .font(Theme::font_bold(11.0))
                .color(Theme::ACCENT_PURPLE),
        );
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            let dirs = [
                (Rotation::R0, "> East (0 deg)"),
                (Rotation::R90, "v South (90 deg)"),
                (Rotation::R180, "< West (180 deg)"),
                (Rotation::R270, "^ North (270 deg)"),
            ];

            for (rot, name) in dirs {
                let is_current = comp.rotation == rot;
                let btn = if is_current {
                    egui::Button::new(
                        RichText::new(name)
                            .font(Theme::font_bold(10.5))
                            .color(Theme::ACCENT_PINK),
                    )
                    .stroke(Stroke::new(1.0, Theme::ACCENT_PINK))
                } else {
                    egui::Button::new(RichText::new(name).font(Theme::font_regular(10.5)))
                };

                if ui.add(btn).clicked() {
                    circuit.set_component_rotation(comp_id, rot);
                    mutated = true;
                }
            }
        });

        ui.add_space(10.0);

        // 4. Number of Inputs (for Logic Gates and configurable blocks)
        let is_logic_gate = matches!(
            comp.kind,
            GateKind::And
                | GateKind::Or
                | GateKind::Nand
                | GateKind::Nor
                | GateKind::Xor
                | GateKind::Xnor
        );

        if is_logic_gate {
            ui.label(
                RichText::new("NUMBER OF INPUTS")
                    .font(Theme::font_bold(10.0))
                    .color(Theme::ACCENT_PURPLE),
            );
            ui.add_space(3.0);

            let current_inputs = comp.input_signals.len();
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        current_inputs > 2,
                        egui::Button::new(RichText::new(" - ").font(Theme::font_bold(12.0))),
                    )
                    .clicked()
                {
                    circuit.set_component_input_count(comp_id, current_inputs - 1);
                    mutated = true;
                }

                ui.label(
                    RichText::new(format!("{} Inputs", current_inputs))
                        .font(Theme::font_bold(12.0))
                        .color(Theme::TEXT_PRIMARY),
                );

                if ui
                    .add_enabled(
                        current_inputs < 8,
                        egui::Button::new(RichText::new(" + ").font(Theme::font_bold(12.0))),
                    )
                    .clicked()
                {
                    circuit.set_component_input_count(comp_id, current_inputs + 1);
                    mutated = true;
                }
            });

            // Quick preset pills: 2, 3, 4, 8 inputs
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for count in [2, 3, 4, 5, 8] {
                    let is_active = current_inputs == count;
                    let text = format!("{} In", count);
                    let btn = if is_active {
                        egui::Button::new(
                            RichText::new(text)
                                .font(Theme::font_bold(9.5))
                                .color(Theme::ACCENT_PINK),
                        )
                        .stroke(Stroke::new(1.0, Theme::ACCENT_PINK))
                    } else {
                        egui::Button::new(RichText::new(text).font(Theme::font_regular(9.5)))
                    };
                    if ui.add(btn).clicked() {
                        circuit.set_component_input_count(comp_id, count);
                        mutated = true;
                    }
                }
            });

            ui.add_space(10.0);
        }

        // 5. Number of Outputs
        ui.label(
            RichText::new("NUMBER OF OUTPUTS")
                .font(Theme::font_bold(10.0))
                .color(Theme::ACCENT_PURPLE),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(format!("{} Output(s)", comp.output_signals.len()))
                .font(Theme::font_regular(11.5))
                .color(Theme::TEXT_PRIMARY),
        );

        ui.add_space(10.0);

        // 6. Data Bits / Bit Width
        ui.label(
            RichText::new("DATA BITS / BIT WIDTH")
                .font(Theme::font_bold(10.0))
                .color(Theme::ACCENT_PURPLE),
        );
        ui.add_space(3.0);
        let cur_bits = comp.bit_width;
        ui.horizontal(|ui| {
            for b in [1, 2, 4, 8, 16] {
                let is_active = cur_bits == b;
                let text = format!("{}-Bit", b);
                let btn = if is_active {
                    egui::Button::new(
                        RichText::new(text)
                            .font(Theme::font_bold(10.0))
                            .color(Theme::ACCENT_PINK),
                    )
                    .stroke(Stroke::new(1.0, Theme::ACCENT_PINK))
                } else {
                    egui::Button::new(RichText::new(text).font(Theme::font_regular(10.0)))
                };
                if ui.add(btn).clicked() {
                    circuit.set_component_bit_width(comp_id, b);
                    mutated = true;
                }
            }
        });

        ui.add_space(10.0);

        // 7. Component Specific Controls
        match comp.kind {
            GateKind::ToggleSwitch | GateKind::BitSwitch => {
                ui.label(
                    RichText::new("SWITCH STATE")
                        .font(Theme::font_bold(10.0))
                        .color(Theme::ACCENT_PURPLE),
                );
                ui.add_space(3.0);
                let is_on = comp.state_flag;
                let lbl = if is_on {
                    "ON / HIGH (1)"
                } else {
                    "OFF / LOW (0)"
                };
                let col = if is_on {
                    Theme::ACCENT_PINK
                } else {
                    Theme::SIGNAL_LOW
                };
                if ui
                    .button(RichText::new(lbl).font(Theme::font_bold(11.5)).color(col))
                    .clicked()
                {
                    if let Some(c) = circuit.components.get_mut(comp_id) {
                        c.state_flag = !c.state_flag;
                    }
                    mutated = true;
                }
                ui.add_space(10.0);
            }
            GateKind::SingleBitDisplay => {
                ui.label(
                    RichText::new("CURRENT BIT VALUE")
                        .font(Theme::font_bold(10.0))
                        .color(Theme::ACCENT_PURPLE),
                );
                ui.add_space(2.0);
                let sig = comp.input_signals.first().copied().unwrap_or(logic_core::Signal::Zero);
                let bit_str = match sig {
                    logic_core::Signal::Zero => "0 (LOW)",
                    logic_core::Signal::One => "1 (HIGH)",
                    logic_core::Signal::X => "X (UNDEFINED)",
                    logic_core::Signal::Z => "Z (HIGH-Z)",
                };
                let col = if sig.is_high() {
                    Theme::ACCENT_PINK
                } else {
                    Theme::TEXT_MUTED
                };
                ui.label(
                    RichText::new(bit_str)
                        .font(Theme::font_bold(12.0))
                        .color(col),
                );
                ui.add_space(10.0);
            }
            GateKind::Clock => {
                ui.label(
                    RichText::new("CLOCK FREQUENCY (HZ)")
                        .font(Theme::font_bold(10.0))
                        .color(Theme::ACCENT_PURPLE),
                );
                ui.add_space(3.0);
                ui.horizontal(|ui| {
                    for hz in [0.5, 1.0, 2.0, 4.0, 8.0] {
                        let is_active = (comp.clock_hz - hz).abs() < 0.1;
                        let text = format!("{} Hz", hz);
                        let btn = if is_active {
                            egui::Button::new(
                                RichText::new(text)
                                    .font(Theme::font_bold(10.0))
                                    .color(Theme::ACCENT_PINK),
                            )
                            .stroke(Stroke::new(1.0, Theme::ACCENT_PINK))
                        } else {
                            egui::Button::new(RichText::new(text).font(Theme::font_regular(10.0)))
                        };
                        if ui.add(btn).clicked() {
                            circuit.set_component_clock_hz(comp_id, hz);
                            mutated = true;
                        }
                    }
                });
                ui.add_space(10.0);
            }
            GateKind::HexDisplay | GateKind::SevenSegment | GateKind::BinaryDisplay4 => {
                ui.label(
                    RichText::new("CURRENT SIGNAL VALUE")
                        .font(Theme::font_bold(10.0))
                        .color(Theme::ACCENT_PURPLE),
                );
                ui.add_space(2.0);
                let mut val = 0u8;
                for (i, sig) in comp.input_signals.iter().enumerate().take(8) {
                    if sig.is_high() {
                        val |= 1 << i;
                    }
                }
                ui.label(
                    RichText::new(format!(
                        "Hex: 0x{:X} | Dec: {} | Bin: {:04b}",
                        val, val, val
                    ))
                    .font(Theme::font_bold(11.5))
                    .color(Theme::ACCENT_PINK),
                );
                ui.add_space(10.0);
            }
            GateKind::Rom16x4 | GateKind::Ram16x4 => {
                ui.label(
                    RichText::new("MEMORY WORDS (HEX)")
                        .font(Theme::font_bold(10.0))
                        .color(Theme::ACCENT_PURPLE),
                );
                ui.add_space(4.0);
                egui::Grid::new("mem_grid")
                    .num_columns(4)
                    .spacing([6.0, 4.0])
                    .show(ui, |ui| {
                        for i in 0..16 {
                            let word = comp.memory.get(i).copied().unwrap_or(0);
                            ui.label(
                                RichText::new(format!("{:X}:", i))
                                    .font(Theme::font_regular(10.0))
                                    .color(Theme::TEXT_MUTED),
                            );
                            let mut hex_str = format!("{:X}", word);
                            if ui
                                .add(egui::TextEdit::singleline(&mut hex_str).desired_width(28.0))
                                .changed()
                                && let Ok(v) = u8::from_str_radix(hex_str.trim(), 16)
                                && let Some(c) = circuit.components.get_mut(comp_id)
                            {
                                if c.memory.len() <= i {
                                    c.memory.resize(16, 0);
                                }
                                c.memory[i] = v & 0xF;
                                mutated = true;
                            }
                            if (i + 1) % 4 == 0 {
                                ui.end_row();
                            }
                        }
                    });
                ui.add_space(10.0);
            }
            _ => {}
        }

        ui.separator();
        ui.add_space(8.0);

        // 8. Quick Actions (Rotate 90 deg, Disconnect Wires, Delete)
        ui.label(
            RichText::new("ACTIONS")
                .font(Theme::font_bold(10.0))
                .color(Theme::ACCENT_PURPLE),
        );
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("Rotate 90 deg (R)").font(Theme::font_medium(11.0)))
                .clicked()
            {
                circuit.set_component_rotation(comp_id, comp.rotation.next());
                mutated = true;
            }
            if ui
                .button(
                    RichText::new("Delete (Del)")
                        .font(Theme::font_medium(11.0))
                        .color(Theme::ACCENT_RED),
                )
                .clicked()
            {
                circuit.remove_component(comp_id);
                mutated = true;
            }
        });

        ui.add_space(12.0);
        mutated
    }
}

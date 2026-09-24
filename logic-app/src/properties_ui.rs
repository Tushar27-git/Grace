use crate::theme::Theme;
use eframe::egui::{self, CornerRadius, RichText, Sense, Stroke, Ui, Vec2};
use logic_core::{Circuit, ComponentId, GateKind, Rotation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftPanelTab {
    Components,
    Properties,
}

pub struct PropertiesUi;

impl PropertiesUi {
    /// Renders the Properties tab inside a panel (e.g. left sidebar tab or right sidebar)
    pub fn show_panel(
        ui: &mut Ui,
        circuit: &mut Circuit,
        selected_id: Option<ComponentId>,
    ) -> bool {
        let mut mutated = false;

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("PROPERTIES")
                    .font(Theme::font_bold(13.5))
                    .color(Theme::TEXT_PRIMARY),
            );
        });
        ui.add_space(8.0);

        let Some(comp_id) = selected_id else {
            Self::show_empty_placeholder(ui, circuit);
            return false;
        };

        if !circuit.components.contains_key(comp_id) {
            Self::show_empty_placeholder(ui, circuit);
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
    ) -> bool {
        if !*is_open {
            return false;
        }

        let Some(comp_id) = selected_id else {
            return false;
        };

        if !circuit.components.contains_key(comp_id) {
            return false;
        }

        let mut mutated = false;
        let modal_frame = Theme::glass_modal();

        egui::Window::new(
            RichText::new("COMPONENT PROPERTIES")
                .font(Theme::font_bold(12.5))
                .color(Theme::TEXT_PRIMARY),
        )
        .open(is_open)
        .frame(modal_frame)
        .resizable(true)
        .default_size(Vec2::new(280.0, 380.0))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-20.0, 60.0))
        .show(ctx, |ui| {
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
                    .font(Theme::font_bold(11.0))
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
                        .font(Theme::font_bold(12.5))
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new(format!("Pos: ({:.0}, {:.0})", comp.pos.0, comp.pos.1))
                        .font(Theme::font_regular(10.0))
                        .color(Theme::TEXT_MUTED),
                );
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        // 2. Custom Label
        ui.label(
            RichText::new("LABEL / DESIGNATOR")
                .font(Theme::font_bold(10.0))
                .color(Theme::ACCENT_PURPLE),
        );
        ui.add_space(2.0);
        let mut label = comp.label.clone();
        if ui
            .add(egui::TextEdit::singleline(&mut label).hint_text("e.g. U1, ENABLE, G1"))
            .changed()
        {
            circuit.set_component_label(comp_id, label);
            mutated = true;
        }

        ui.add_space(10.0);

        // 3. Direction / Orientation (East, South, West, North)
        ui.label(
            RichText::new("DIRECTION / ORIENTATION")
                .font(Theme::font_bold(10.0))
                .color(Theme::ACCENT_PURPLE),
        );
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            let dirs = [
                (Rotation::R0, "East (0°)", "▶"),
                (Rotation::R90, "South (90°)", "▼"),
                (Rotation::R180, "West (180°)", "◀"),
                (Rotation::R270, "North (270°)", "▲"),
            ];

            for (rot, name, icon) in dirs {
                let is_current = comp.rotation == rot;
                let btn_text = format!("{} {}", icon, name);
                let btn = if is_current {
                    egui::Button::new(
                        RichText::new(btn_text)
                            .font(Theme::font_bold(10.5))
                            .color(Theme::ACCENT_PINK),
                    )
                    .stroke(Stroke::new(1.0, Theme::ACCENT_PINK))
                } else {
                    egui::Button::new(RichText::new(btn_text).font(Theme::font_regular(10.5)))
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
                        egui::Button::new(RichText::new(" − ").font(Theme::font_bold(12.0))),
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
            GateKind::ToggleSwitch => {
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

        // 8. Quick Actions (Rotate 90°, Disconnect Wires, Delete)
        ui.label(
            RichText::new("ACTIONS")
                .font(Theme::font_bold(10.0))
                .color(Theme::ACCENT_PURPLE),
        );
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("↻ Rotate 90°").font(Theme::font_medium(11.0)))
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

use crate::theme::Theme;
use eframe::egui::{self, RichText, Sense, Ui, Vec2, Window};
use logic_core::{
    Circuit, CircuitIO, ComponentId, GateKind, Simulator, TruthTable, TruthTableDiff,
    TruthTableGenerator,
};
use std::collections::HashSet;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniversalChallenge {
    NotFromNand,
    AndFromNand,
    OrFromNand,
    XorFromNand,
    NotFromNor,
    OrFromNor,
    AndFromNor,
    XorFromNor,
}

impl UniversalChallenge {
    pub const ALL: &'static [UniversalChallenge] = &[
        UniversalChallenge::NotFromNand,
        UniversalChallenge::AndFromNand,
        UniversalChallenge::OrFromNand,
        UniversalChallenge::XorFromNand,
        UniversalChallenge::NotFromNor,
        UniversalChallenge::OrFromNor,
        UniversalChallenge::AndFromNor,
        UniversalChallenge::XorFromNor,
    ];

    pub fn title(self) -> &'static str {
        match self {
            UniversalChallenge::NotFromNand => "NOT from NAND",
            UniversalChallenge::AndFromNand => "AND from NAND",
            UniversalChallenge::OrFromNand => "OR from NAND",
            UniversalChallenge::XorFromNand => "XOR from NAND",
            UniversalChallenge::NotFromNor => "NOT from NOR",
            UniversalChallenge::OrFromNor => "OR from NOR",
            UniversalChallenge::AndFromNor => "AND from NOR",
            UniversalChallenge::XorFromNor => "XOR from NOR",
        }
    }

    pub fn target_kind(self) -> GateKind {
        match self {
            UniversalChallenge::NotFromNand | UniversalChallenge::NotFromNor => GateKind::Not,
            UniversalChallenge::AndFromNand | UniversalChallenge::AndFromNor => GateKind::And,
            UniversalChallenge::OrFromNand | UniversalChallenge::OrFromNor => GateKind::Or,
            UniversalChallenge::XorFromNand | UniversalChallenge::XorFromNor => GateKind::Xor,
        }
    }

    pub fn allowed_gate(self) -> GateKind {
        match self {
            UniversalChallenge::NotFromNand
            | UniversalChallenge::AndFromNand
            | UniversalChallenge::OrFromNand
            | UniversalChallenge::XorFromNand => GateKind::Nand,
            UniversalChallenge::NotFromNor
            | UniversalChallenge::OrFromNor
            | UniversalChallenge::AndFromNor
            | UniversalChallenge::XorFromNor => GateKind::Nor,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            UniversalChallenge::NotFromNand => "Construct a NOT inverter using only 1 NAND gate.",
            UniversalChallenge::AndFromNand => "Construct an AND gate using 2 NAND gates.",
            UniversalChallenge::OrFromNand => {
                "Construct an OR gate using 3 NAND gates (De Morgan's law)."
            }
            UniversalChallenge::XorFromNand => "Construct an XOR gate using 4 NAND gates.",
            UniversalChallenge::NotFromNor => "Construct a NOT inverter using only 1 NOR gate.",
            UniversalChallenge::OrFromNor => "Construct an OR gate using 2 NOR gates.",
            UniversalChallenge::AndFromNor => {
                "Construct an AND gate using 3 NOR gates (De Morgan's law)."
            }
            UniversalChallenge::XorFromNor => "Construct an XOR gate using 4 or 5 NOR gates.",
        }
    }
}

pub struct VerificationUiState {
    pub is_open: bool,
    pub current_table: Option<TruthTable>,
    pub current_io: Option<CircuitIO>,
    pub error_message: Option<String>,
    pub verification_mode: bool,
    pub expected_target: GateKind,
    pub diff_result: Option<TruthTableDiff>,
    pub active_challenge: Option<UniversalChallenge>,
    pub challenge_feedback: Option<(bool, String)>,
}

impl Default for VerificationUiState {
    fn default() -> Self {
        Self {
            is_open: false,
            current_table: None,
            current_io: None,
            error_message: None,
            verification_mode: false,
            expected_target: GateKind::And,
            diff_result: None,
            active_challenge: None,
            challenge_feedback: None,
        }
    }
}

impl VerificationUiState {
    pub fn generate_from_circuit(
        &mut self,
        circuit: &Circuit,
        selected_components: &HashSet<ComponentId>,
    ) {
        let filter = if selected_components.is_empty() {
            None
        } else {
            Some(selected_components)
        };

        match TruthTableGenerator::generate(circuit, filter) {
            Ok((table, io)) => {
                self.current_table = Some(table);
                self.current_io = Some(io);
                self.error_message = None;
                self.update_diff();
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
                self.current_table = None;
                self.current_io = None;
                self.diff_result = None;
            }
        }
    }

    pub fn update_diff(&mut self) {
        if self.verification_mode {
            if let Some(table) = &self.current_table {
                let ref_table = TruthTableGenerator::reference_table(self.expected_target.clone());
                self.diff_result = Some(table.diff(&ref_table));
            }
        } else {
            self.diff_result = None;
        }
    }

    pub fn load_challenge(&mut self, challenge: UniversalChallenge, circuit: &mut Circuit) {
        circuit.components.clear();
        circuit.nets.clear();
        self.active_challenge = Some(challenge);
        self.challenge_feedback = None;
        self.verification_mode = true;
        self.expected_target = challenge.target_kind();

        // Scaffold inputs and output
        let is_single_input = challenge.target_kind() == GateKind::Not;

        let sw_a = circuit.add_component(GateKind::ToggleSwitch, (100.0, 160.0));
        let sw_b = if !is_single_input {
            Some(circuit.add_component(GateKind::ToggleSwitch, (100.0, 260.0)))
        } else {
            None
        };

        let led = circuit.add_component(GateKind::Led, (450.0, 200.0));

        // Place ready-to-wire universal gates
        let gate_kind = challenge.allowed_gate();
        match challenge {
            UniversalChallenge::NotFromNand | UniversalChallenge::NotFromNor => {
                circuit.add_component(gate_kind, (260.0, 160.0));
            }
            UniversalChallenge::AndFromNand | UniversalChallenge::OrFromNor => {
                circuit.add_component(gate_kind.clone(), (240.0, 200.0));
                circuit.add_component(gate_kind, (340.0, 200.0));
            }
            UniversalChallenge::OrFromNand | UniversalChallenge::AndFromNor => {
                circuit.add_component(gate_kind.clone(), (220.0, 150.0));
                circuit.add_component(gate_kind.clone(), (220.0, 250.0));
                circuit.add_component(gate_kind, (330.0, 200.0));
            }
            UniversalChallenge::XorFromNand | UniversalChallenge::XorFromNor => {
                circuit.add_component(gate_kind.clone(), (200.0, 200.0));
                circuit.add_component(gate_kind.clone(), (290.0, 140.0));
                circuit.add_component(gate_kind.clone(), (290.0, 260.0));
                circuit.add_component(gate_kind, (380.0, 200.0));
            }
        }

        Simulator::settle(circuit);
        _ = (sw_a, sw_b, led);
    }

    pub fn verify_current_challenge(&mut self, circuit: &Circuit) {
        let challenge = match self.active_challenge {
            Some(c) => c,
            None => return,
        };

        match TruthTableGenerator::generate(circuit, None) {
            Ok((table, io)) => {
                let ref_table = TruthTableGenerator::reference_table(challenge.target_kind());
                let diff = table.diff(&ref_table);

                if diff.is_match {
                    self.challenge_feedback = Some((
                        true,
                        format!(
                            "CHALLENGE PASSED! Successfully constructed {} using {} gates.",
                            challenge.target_kind().display_name(),
                            challenge.allowed_gate().short_code()
                        ),
                    ));
                } else {
                    let mismatches = diff.mismatches.len();
                    self.challenge_feedback = Some((
                        false,
                        format!(
                            "Verification Failed: {} / {} output combinations mismatch target {}.",
                            mismatches,
                            diff.total_rows,
                            challenge.target_kind().display_name()
                        ),
                    ));
                }

                self.current_table = Some(table);
                self.current_io = Some(io);
                self.diff_result = Some(diff);
                self.is_open = true;
            }
            Err(e) => {
                self.challenge_feedback = Some((false, format!("Cannot verify: {}", e)));
            }
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        circuit: &mut Circuit,
        selected_components: &HashSet<ComponentId>,
    ) {
        if !self.is_open {
            return;
        }

        let mut is_open_local = self.is_open;

        Window::new(RichText::new("Truth Table & Verification").color(Theme::TEXT_PRIMARY))
            .open(&mut is_open_local)
            .resizable(true)
            .default_width(520.0)
            .default_height(440.0)
            .frame(Theme::glass_modal())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Regenerate Table").clicked() {
                        self.generate_from_circuit(circuit, selected_components);
                    }

                    if let Some(table) = &self.current_table {
                        if ui.button("Copy Table").clicked() {
                            let text = table.to_plain_text();
                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                let _ = cb.set_text(text);
                            }
                        }

                        if ui.button("Export CSV").clicked()
                            && let Some(path) = rfd::FileDialog::new()
                                .add_filter("CSV", &["csv"])
                                .set_file_name("truth_table.csv")
                                .save_file()
                        {
                            let _ = fs::write(&path, table.to_csv());
                        }

                        if ui.button("Export Text").clicked()
                            && let Some(path) = rfd::FileDialog::new()
                                .add_filter("Text", &["txt"])
                                .set_file_name("truth_table.txt")
                                .save_file()
                        {
                            let _ = fs::write(&path, table.to_plain_text());
                        }
                    }
                });

                ui.separator();

                // Verification Mode Controls
                ui.horizontal(|ui| {
                    if ui
                        .checkbox(&mut self.verification_mode, "Verification Mode")
                        .changed()
                    {
                        self.update_diff();
                    }

                    if self.verification_mode {
                        ui.label("Target Function:");
                        let prev_target = self.expected_target.clone();
                        egui::ComboBox::from_id_salt("verify_target_combo")
                            .selected_text(self.expected_target.display_name())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.expected_target,
                                    GateKind::Not,
                                    "NOT Gate",
                                );
                                ui.selectable_value(
                                    &mut self.expected_target,
                                    GateKind::And,
                                    "AND Gate",
                                );
                                ui.selectable_value(
                                    &mut self.expected_target,
                                    GateKind::Or,
                                    "OR Gate",
                                );
                                ui.selectable_value(
                                    &mut self.expected_target,
                                    GateKind::Xor,
                                    "XOR Gate",
                                );
                                ui.selectable_value(
                                    &mut self.expected_target,
                                    GateKind::Nand,
                                    "NAND Gate",
                                );
                                ui.selectable_value(
                                    &mut self.expected_target,
                                    GateKind::Nor,
                                    "NOR Gate",
                                );
                                ui.selectable_value(
                                    &mut self.expected_target,
                                    GateKind::Xnor,
                                    "XNOR Gate",
                                );
                            });

                        if prev_target != self.expected_target {
                            self.update_diff();
                        }
                    }
                });

                if let Some(diff) = &self.diff_result {
                    ui.add_space(4.0);
                    if diff.is_match {
                        ui.label(
                            RichText::new(
                                "VERIFICATION PASSED: All outputs match expected function.",
                            )
                            .color(Theme::ACCENT_PINK)
                            .strong(),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!(
                                "VERIFICATION FAILED: {}/{} mismatches detected.",
                                diff.mismatches.len(),
                                diff.total_rows
                            ))
                            .color(Theme::ACCENT_RED)
                            .strong(),
                        );
                    }
                }

                ui.separator();

                if let Some(err) = &self.error_message {
                    ui.add_space(10.0);
                    ui.label(RichText::new(err).color(Theme::ACCENT_RED));
                } else if let Some(table) = &self.current_table {
                    ui.label(
                        RichText::new("Click a row to drive live canvas to that input state:")
                            .size(11.0)
                            .color(Theme::ACCENT_PURPLE),
                    );
                    ui.add_space(6.0);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        Self::render_truth_table_grid(
                            ui,
                            table,
                            &self.current_io,
                            self.diff_result.as_ref(),
                            circuit,
                        );
                    });
                } else {
                    ui.label("Click 'Regenerate Table' to analyze the current circuit.");
                }
            });

        self.is_open = is_open_local;
    }

    fn render_truth_table_grid(
        ui: &mut Ui,
        table: &TruthTable,
        io: &Option<CircuitIO>,
        diff: Option<&TruthTableDiff>,
        circuit: &mut Circuit,
    ) {
        egui::Grid::new("truth_table_grid")
            .striped(true)
            .spacing(Vec2::new(14.0, 6.0))
            .show(ui, |ui| {
                // Header row
                for name in &table.input_names {
                    ui.label(RichText::new(name).strong().color(Theme::ACCENT_PURPLE));
                }
                for name in &table.output_names {
                    let label = if diff.is_some() {
                        format!("{} (Act)", name)
                    } else {
                        name.clone()
                    };
                    ui.label(RichText::new(label).strong().color(Theme::ACCENT_PINK));
                }
                if diff.is_some() {
                    ui.label(
                        RichText::new("Expected")
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                    ui.label(RichText::new("Status").strong().color(Theme::TEXT_PRIMARY));
                }
                ui.end_row();

                // Rows
                for (row_idx, row) in table.rows.iter().enumerate() {
                    let mismatch_entry =
                        diff.and_then(|d| d.mismatches.iter().find(|m| m.row_index == row_idx));
                    let is_mismatch = mismatch_entry.is_some();

                    let row_color = if is_mismatch {
                        Theme::ACCENT_RED
                    } else {
                        Theme::TEXT_PRIMARY
                    };

                    // Input cells
                    for sig in &row.inputs {
                        let text = if sig.is_high() { "1" } else { "0" };
                        let r = ui.add(
                            egui::Label::new(
                                RichText::new(text)
                                    .font(egui::FontId::monospace(12.0))
                                    .color(row_color),
                            )
                            .sense(Sense::click()),
                        );
                        if r.clicked() {
                            Self::drive_inputs_from_row(io, row, circuit);
                        }
                    }

                    // Actual output cells
                    for sig in &row.outputs {
                        let text = if sig.is_high() { "1" } else { "0" };
                        let r = ui.add(
                            egui::Label::new(
                                RichText::new(text)
                                    .font(egui::FontId::monospace(12.0))
                                    .color(if is_mismatch {
                                        Theme::ACCENT_RED
                                    } else {
                                        Theme::ACCENT_PINK
                                    }),
                            )
                            .sense(Sense::click()),
                        );
                        if r.clicked() {
                            Self::drive_inputs_from_row(io, row, circuit);
                        }
                    }

                    // Expected & Status cells
                    if let Some(_d) = diff {
                        if let Some(m) = mismatch_entry {
                            let exp_str = m
                                .expected
                                .iter()
                                .map(|s| if s.is_high() { "1" } else { "0" })
                                .collect::<Vec<_>>()
                                .join(" ");
                            ui.label(
                                RichText::new(exp_str)
                                    .font(egui::FontId::monospace(12.0))
                                    .color(Theme::TEXT_PRIMARY),
                            );
                            ui.label(RichText::new("MISMATCH").color(Theme::ACCENT_RED).strong());
                        } else {
                            let exp_str = row
                                .outputs
                                .iter()
                                .map(|s| if s.is_high() { "1" } else { "0" })
                                .collect::<Vec<_>>()
                                .join(" ");
                            ui.label(
                                RichText::new(exp_str)
                                    .font(egui::FontId::monospace(12.0))
                                    .color(Theme::TEXT_PRIMARY),
                            );
                            ui.label(RichText::new("MATCH").color(Theme::ACCENT_PINK));
                        }
                    }

                    ui.end_row();
                }
            });
    }

    fn drive_inputs_from_row(
        io: &Option<CircuitIO>,
        row: &logic_core::TruthTableRow,
        circuit: &mut Circuit,
    ) {
        if let Some(circuit_io) = io {
            for (bit_idx, (comp_id, _)) in circuit_io.input_switches.iter().enumerate() {
                if let Some(sig) = row.inputs.get(bit_idx)
                    && let Some(comp) = circuit.components.get_mut(*comp_id)
                {
                    comp.state_flag = sig.is_high();
                }
            }
            Simulator::settle(circuit);
        }
    }
}

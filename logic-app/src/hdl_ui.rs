use crate::theme::Theme;
use eframe::egui::{self, RichText};
use logic_core::{Circuit, HdlExporter};
use std::fs;

#[derive(Debug, PartialEq, Eq)]
pub enum HdlTab {
    Verilog,
    Vhdl,
}

pub struct HdlUiState {
    pub is_open: bool,
    pub active_tab: HdlTab,
    pub module_name: String,
    pub generated_verilog: String,
    pub generated_vhdl: String,
    pub feedback_msg: Option<String>,
}

impl Default for HdlUiState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: HdlTab::Verilog,
            module_name: "circuit_top".to_string(),
            generated_verilog: String::new(),
            generated_vhdl: String::new(),
            feedback_msg: None,
        }
    }
}

impl HdlUiState {
    pub fn open(&mut self, circuit: &Circuit) {
        self.is_open = true;
        self.feedback_msg = None;
        self.regenerate(circuit);
    }

    pub fn regenerate(&mut self, circuit: &Circuit) {
        self.generated_verilog = HdlExporter::to_verilog(circuit, &self.module_name);
        self.generated_vhdl = HdlExporter::to_vhdl(circuit, &self.module_name);
    }

    pub fn show(&mut self, ctx: &egui::Context, circuit: &Circuit) {
        if !self.is_open {
            return;
        }

        let mut is_open_local = self.is_open;
        let modal_frame = Theme::glass_modal();

        egui::Window::new("Export Hardware Description (HDL)")
            .open(&mut is_open_local)
            .frame(modal_frame)
            .resizable(true)
            .default_size(egui::Vec2::new(560.0, 480.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Module / Entity Name:");
                    if ui.text_edit_singleline(&mut self.module_name).changed() {
                        self.regenerate(circuit);
                    }
                    if ui.button("Regenerate").clicked() {
                        self.regenerate(circuit);
                    }
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.active_tab == HdlTab::Verilog, "Verilog (.v)")
                        .clicked()
                    {
                        self.active_tab = HdlTab::Verilog;
                    }
                    if ui
                        .selectable_label(self.active_tab == HdlTab::Vhdl, "VHDL (.vhd)")
                        .clicked()
                    {
                        self.active_tab = HdlTab::Vhdl;
                    }
                });

                ui.separator();

                let current_code = match self.active_tab {
                    HdlTab::Verilog => &self.generated_verilog,
                    HdlTab::Vhdl => &self.generated_vhdl,
                };

                // Code preview area
                egui::ScrollArea::vertical()
                    .max_height(340.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut current_code.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Copy to Clipboard").clicked()
                        && let Ok(mut cb) = arboard::Clipboard::new()
                    {
                        let _ = cb.set_text(current_code.clone());
                        self.feedback_msg = Some("Copied code to clipboard!".to_string());
                    }

                    if ui.button("Export to File...").clicked() {
                        let (ext, filter_name) = match self.active_tab {
                            HdlTab::Verilog => ("v", "Verilog File (*.v)"),
                            HdlTab::Vhdl => ("vhd", "VHDL File (*.vhd)"),
                        };

                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name(format!("{}.{}", self.module_name, ext))
                            .add_filter(filter_name, &[ext])
                            .save_file()
                            && fs::write(&path, current_code).is_ok()
                        {
                            self.feedback_msg = Some(format!("Saved: {}", path.display()));
                        }
                    }

                    if let Some(ref msg) = self.feedback_msg {
                        ui.label(RichText::new(msg).color(Theme::ACCENT_PINK));
                    }
                });
            });

        self.is_open = is_open_local;
    }
}

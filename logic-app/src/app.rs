use crate::canvas::CanvasState;
use crate::hdl_ui::HdlUiState;
use crate::palette::Palette;
use crate::theme::{Theme, ThemeMode, apply_theme};
use crate::verification_ui::{UniversalChallenge, VerificationUiState};
use crate::waveform::WaveformState;
use eframe::egui::{self, CentralPanel, Frame, Key, Panel, RichText, Ui};
use logic_core::{Circuit, ClockEdge, ComponentId, GateKind, PortEndpoint, Rotation, Simulator};
use ron::ser::PrettyConfig;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

pub struct LogicLabApp {
    pub circuit: Circuit,
    pub canvas: CanvasState,
    pub selected_for_placement: Option<GateKind>,
    pub undo_stack: Vec<Circuit>,
    pub redo_stack: Vec<Circuit>,
    pub clock_running: bool,
    pub last_clock_tick: Instant,
    pub status_message: String,
    pub verification_ui: VerificationUiState,
    pub subcircuit_modal_open: bool,
    pub new_subcircuit_name: String,
    pub waveform: WaveformState,
    pub hdl_ui: HdlUiState,
    pub theme_mode: ThemeMode,
}

impl LogicLabApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_theme(&cc.egui_ctx);
        let mut circuit = Circuit::new();
        Self::create_demo_circuit(&mut circuit);

        Self {
            circuit,
            canvas: CanvasState::default(),
            selected_for_placement: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clock_running: false,
            last_clock_tick: Instant::now(),
            status_message: "Ready. Select components to build logic circuits.".to_string(),
            verification_ui: VerificationUiState::default(),
            subcircuit_modal_open: false,
            new_subcircuit_name: String::new(),
            waveform: WaveformState::default(),
            hdl_ui: HdlUiState::default(),
            theme_mode: ThemeMode::Light,
        }
    }

    fn create_demo_circuit(circuit: &mut Circuit) {
        // Build a welcome starter circuit: ToggleSwitch -> NOT Gate -> LED
        let sw = circuit.add_component(GateKind::ToggleSwitch, (100.0, 100.0));
        let not_gate = circuit.add_component(GateKind::Not, (220.0, 100.0));
        let led = circuit.add_component(GateKind::Led, (340.0, 100.0));

        let sw_out = PortEndpoint {
            component_id: sw,
            is_output: true,
            port_index: 0,
        };
        let not_in = PortEndpoint {
            component_id: not_gate,
            is_output: false,
            port_index: 0,
        };
        circuit.connect_ports(sw_out, not_in);

        let not_out = PortEndpoint {
            component_id: not_gate,
            is_output: true,
            port_index: 0,
        };
        let led_in = PortEndpoint {
            component_id: led,
            is_output: false,
            port_index: 0,
        };
        circuit.connect_ports(not_out, led_in);

        Simulator::settle(circuit);
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.circuit.clone());
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.circuit.clone());
            self.circuit = prev;
            Simulator::settle(&mut self.circuit);
            self.status_message = "Undo".to_string();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.circuit.clone());
            self.circuit = next;
            Simulator::settle(&mut self.circuit);
            self.status_message = "Redo".to_string();
        }
    }

    fn delete_selected(&mut self) {
        if self.canvas.selection.is_empty() {
            return;
        }
        self.push_undo();
        for id in self.canvas.selection.selected_components.drain() {
            self.circuit.remove_component(id);
        }
        Simulator::settle(&mut self.circuit);
        self.status_message = "Deleted selected items".to_string();
    }

    fn rotate_selected(&mut self) {
        if self.canvas.selection.selected_components.is_empty() {
            return;
        }
        self.push_undo();
        for &id in &self.canvas.selection.selected_components {
            if let Some(comp) = self.circuit.components.get_mut(id) {
                comp.rotation = comp.rotation.next();
            }
        }
        Simulator::settle(&mut self.circuit);
        self.status_message = "Rotated 90°".to_string();
    }

    fn duplicate_selected(&mut self) {
        if self.canvas.selection.selected_components.is_empty() {
            return;
        }
        self.push_undo();

        let mut old_to_new = HashMap::new();
        let offset = (40.0, 40.0);

        type ComponentDuplicateData = (ComponentId, GateKind, (f32, f32), Rotation, bool);
        let to_dup: Vec<ComponentDuplicateData> = self
            .canvas
            .selection
            .selected_components
            .iter()
            .filter_map(|&id| {
                self.circuit
                    .components
                    .get(id)
                    .map(|c| (id, c.kind.clone(), c.pos, c.rotation, c.state_flag))
            })
            .collect();

        for (old_id, kind, pos, rotation, state_flag) in to_dup {
            let new_id = self
                .circuit
                .add_component(kind, (pos.0 + offset.0, pos.1 + offset.1));
            if let Some(new_comp) = self.circuit.components.get_mut(new_id) {
                new_comp.rotation = rotation;
                new_comp.state_flag = state_flag;
            }
            old_to_new.insert(old_id, new_id);
        }

        self.canvas.selection.clear();
        for &new_id in old_to_new.values() {
            self.canvas.selection.selected_components.insert(new_id);
        }

        Simulator::settle(&mut self.circuit);
        self.status_message = format!("Duplicated {} components", old_to_new.len());
    }

    fn copy_to_clipboard(&mut self) {
        if self.canvas.selection.selected_components.is_empty() {
            return;
        }

        let selected_set = &self.canvas.selection.selected_components;
        let mut sub_circuit = Circuit::new();
        let mut map = HashMap::new();

        for &id in selected_set {
            if let Some(comp) = self.circuit.components.get(id) {
                let nid = sub_circuit.add_component(comp.kind.clone(), comp.pos);
                if let Some(nc) = sub_circuit.components.get_mut(nid) {
                    nc.rotation = comp.rotation;
                    nc.state_flag = comp.state_flag;
                }
                map.insert(id, nid);
            }
        }

        // Copy internal nets
        for (_, net) in &self.circuit.nets {
            if let Some(&new_src_id) = map.get(&net.source.component_id) {
                let new_src = PortEndpoint {
                    component_id: new_src_id,
                    is_output: true,
                    port_index: net.source.port_index,
                };
                for sink in &net.sinks {
                    if let Some(&new_sink_id) = map.get(&sink.component_id) {
                        let new_sink = PortEndpoint {
                            component_id: new_sink_id,
                            is_output: false,
                            port_index: sink.port_index,
                        };
                        sub_circuit.connect_ports(new_src, new_sink);
                    }
                }
            }
        }

        if let Ok(ron_str) = ron::ser::to_string_pretty(&sub_circuit, PrettyConfig::default())
            && let Ok(mut clipboard) = arboard::Clipboard::new()
        {
            let _ = clipboard.set_text(ron_str);
            self.status_message = format!("Copied {} components", selected_set.len());
        }
    }

    fn package_selection_into_subcircuit(&mut self, name: String) {
        if name.trim().is_empty() {
            return;
        }
        let selected_set: Vec<ComponentId> = self
            .canvas
            .selection
            .selected_components
            .iter()
            .copied()
            .collect();
        if selected_set.is_empty() {
            return;
        }

        self.push_undo();
        let mut sub_circuit = Circuit::new();
        let mut map = HashMap::new();

        // Calculate bounding box minimum to normalize component positions
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        for &id in &selected_set {
            if let Some(comp) = self.circuit.components.get(id) {
                min_x = min_x.min(comp.pos.0);
                min_y = min_y.min(comp.pos.1);
            }
        }
        if min_x == f32::MAX {
            min_x = 0.0;
            min_y = 0.0;
        }

        for &id in &selected_set {
            if let Some(comp) = self.circuit.components.get(id) {
                let norm_pos = (comp.pos.0 - min_x + 80.0, comp.pos.1 - min_y + 80.0);
                let nid = sub_circuit.add_component(comp.kind.clone(), norm_pos);
                if let Some(nc) = sub_circuit.components.get_mut(nid) {
                    nc.rotation = comp.rotation;
                    nc.state_flag = comp.state_flag;
                }
                map.insert(id, nid);
            }
        }

        // Copy internal nets
        for (_, net) in &self.circuit.nets {
            if let Some(&new_src_id) = map.get(&net.source.component_id) {
                let new_src = PortEndpoint {
                    component_id: new_src_id,
                    is_output: net.source.is_output,
                    port_index: net.source.port_index,
                };
                for sink in &net.sinks {
                    if let Some(&new_sink_id) = map.get(&sink.component_id) {
                        let new_sink = PortEndpoint {
                            component_id: new_sink_id,
                            is_output: sink.is_output,
                            port_index: sink.port_index,
                        };
                        sub_circuit.connect_ports(new_src, new_sink);
                    }
                }
            }
        }

        let input_count = sub_circuit
            .components
            .iter()
            .filter(|(_, c)| matches!(c.kind, GateKind::ToggleSwitch | GateKind::Clock))
            .count()
            .max(1);
        let output_count = sub_circuit
            .components
            .iter()
            .filter(|(_, c)| c.kind == GateKind::Led)
            .count()
            .max(1);

        let input_names = (0..input_count).map(|i| format!("IN_{}", i)).collect();
        let output_names = (0..output_count).map(|i| format!("OUT_{}", i)).collect();

        let sub_def = logic_core::SubcircuitDef {
            name: name.clone(),
            input_names,
            output_names,
            circuit: sub_circuit,
        };

        self.circuit.subcircuits.insert(name.clone(), sub_def);
        self.status_message = format!("Created subcircuit '{}' in palette!", name);
    }

    fn paste_from_clipboard(&mut self) {
        let text = match arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
            Ok(t) => t,
            Err(_) => return,
        };

        let sub_circuit: Circuit = match ron::from_str(&text) {
            Ok(c) => c,
            Err(_) => return,
        };

        if sub_circuit.components.is_empty() {
            return;
        }

        self.push_undo();
        let mut map = HashMap::new();
        let offset = (40.0, 40.0);

        for (old_id, comp) in &sub_circuit.components {
            let new_id = self.circuit.add_component(
                comp.kind.clone(),
                (comp.pos.0 + offset.0, comp.pos.1 + offset.1),
            );
            if let Some(new_comp) = self.circuit.components.get_mut(new_id) {
                new_comp.rotation = comp.rotation;
                new_comp.state_flag = comp.state_flag;
            }
            map.insert(old_id, new_id);
        }

        for (_, net) in &sub_circuit.nets {
            if let Some(&new_src_id) = map.get(&net.source.component_id) {
                let new_src = PortEndpoint {
                    component_id: new_src_id,
                    is_output: true,
                    port_index: net.source.port_index,
                };
                for sink in &net.sinks {
                    if let Some(&new_sink_id) = map.get(&sink.component_id) {
                        let new_sink = PortEndpoint {
                            component_id: new_sink_id,
                            is_output: false,
                            port_index: sink.port_index,
                        };
                        self.circuit.connect_ports(new_src, new_sink);
                    }
                }
            }
        }

        self.canvas.selection.clear();
        for &nid in map.values() {
            self.canvas.selection.selected_components.insert(nid);
        }

        Simulator::settle(&mut self.circuit);
        self.status_message = format!("Pasted {} components", map.len());
    }

    fn save_to_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Logic Lab Circuit", &["ron"])
            .set_file_name("circuit.ron")
            .save_file()
            && let Ok(ron_str) = ron::ser::to_string_pretty(&self.circuit, PrettyConfig::default())
        {
            if fs::write(&path, ron_str).is_ok() {
                self.status_message = format!("Saved circuit to {}", path.display());
            } else {
                self.status_message = "Failed to write file".to_string();
            }
        }
    }

    fn load_from_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Logic Lab Circuit", &["ron"])
            .pick_file()
            && let Ok(content) = fs::read_to_string(&path)
        {
            if let Ok(loaded) = ron::from_str::<Circuit>(&content) {
                self.push_undo();
                self.circuit = loaded;
                Simulator::settle(&mut self.circuit);
                self.canvas.selection.clear();
                self.status_message = format!("Loaded circuit from {}", path.display());
            } else {
                self.status_message = "Failed to parse .ron circuit file".to_string();
            }
        }
    }

    fn clear_circuit(&mut self) {
        self.push_undo();
        self.circuit = Circuit::new();
        self.canvas.selection.clear();
        self.status_message = "Canvas cleared".to_string();
    }
}

impl eframe::App for LogicLabApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // Automatic periodic tick for Clock sources
        if self.clock_running && self.last_clock_tick.elapsed().as_millis() >= 500 {
            self.last_clock_tick = Instant::now();
            Simulator::tick(&mut self.circuit, ClockEdge::Rising);
            self.waveform.sample_circuit(&self.circuit);
            ui.ctx().request_repaint();
        }

        // Global Keyboard Shortcuts
        ui.input(|i| {
            let ctrl = i.modifiers.command || i.modifiers.ctrl;
            let shift = i.modifiers.shift;

            if ctrl && i.key_pressed(Key::Z) {
                if shift {
                    self.redo();
                } else {
                    self.undo();
                }
            } else if ctrl && i.key_pressed(Key::Y) {
                self.redo();
            } else if ctrl && i.key_pressed(Key::C) {
                self.copy_to_clipboard();
            } else if ctrl && i.key_pressed(Key::V) {
                self.paste_from_clipboard();
            } else if ctrl && i.key_pressed(Key::D) {
                self.duplicate_selected();
            } else if ctrl && i.key_pressed(Key::S) {
                self.save_to_file();
            } else if ctrl && i.key_pressed(Key::O) {
                self.load_from_file();
            } else if i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace) {
                self.delete_selected();
            } else if i.key_pressed(Key::R) {
                self.rotate_selected();
            } else if i.key_pressed(Key::F8) || (ctrl && i.key_pressed(Key::T)) {
                self.theme_mode.toggle();
            }
        });

        // 1. Top Menu & Navigation Bar
        let top_frame = Theme::glass_panel();
        Panel::top("top_toolbar").frame(top_frame).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Logic Lab")
                        .size(15.0)
                        .strong()
                        .color(Theme::ACCENT_PINK),
                );

                ui.separator();

                // File Menu
                ui.menu_button("File", |ui| {
                    if ui.button("Save Circuit... (Ctrl+S)").clicked() {
                        ui.close();
                        self.save_to_file();
                    }
                    if ui.button("Load Circuit... (Ctrl+O)").clicked() {
                        ui.close();
                        self.load_from_file();
                    }
                    ui.separator();
                    if ui.button("Export HDL (Verilog / VHDL)...").clicked() {
                        ui.close();
                        self.hdl_ui.open(&self.circuit);
                    }
                    ui.separator();
                    if ui
                        .button(RichText::new("Clear Canvas").color(Theme::ACCENT_RED))
                        .clicked()
                    {
                        ui.close();
                        self.clear_circuit();
                    }
                });

                // Edit Menu
                ui.menu_button("Edit", |ui| {
                    let can_undo = !self.undo_stack.is_empty();
                    if ui
                        .add_enabled(can_undo, egui::Button::new("Undo (Ctrl+Z)"))
                        .clicked()
                    {
                        ui.close();
                        self.undo();
                    }
                    let can_redo = !self.redo_stack.is_empty();
                    if ui
                        .add_enabled(can_redo, egui::Button::new("Redo (Ctrl+Y)"))
                        .clicked()
                    {
                        ui.close();
                        self.redo();
                    }
                    ui.separator();
                    let has_sel = !self.canvas.selection.is_empty();
                    if ui
                        .add_enabled(has_sel, egui::Button::new("Rotate Selection (R)"))
                        .clicked()
                    {
                        ui.close();
                        self.rotate_selected();
                    }
                    if ui
                        .add_enabled(has_sel, egui::Button::new("Duplicate Selection (Ctrl+D)"))
                        .clicked()
                    {
                        ui.close();
                        self.duplicate_selected();
                    }
                    if ui
                        .add_enabled(
                            has_sel,
                            egui::Button::new(
                                RichText::new("Delete Selection (Del)").color(Theme::ACCENT_RED),
                            ),
                        )
                        .clicked()
                    {
                        ui.close();
                        self.delete_selected();
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            has_sel,
                            egui::Button::new(
                                RichText::new("Package Subcircuit (Ctrl+G)")
                                    .color(Theme::ACCENT_PINK),
                            ),
                        )
                        .clicked()
                    {
                        ui.close();
                        self.new_subcircuit_name =
                            format!("IC_{}", self.circuit.subcircuits.len() + 1);
                        self.subcircuit_modal_open = true;
                    }
                });

                // View Menu
                ui.menu_button("View", |ui| {
                    let theme_lbl = if self.theme_mode.is_dark() {
                        "Switch to Light Mode (Ctrl+T / F8)"
                    } else {
                        "Switch to Dark Mode (Ctrl+T / F8)"
                    };
                    if ui.button(theme_lbl).clicked() {
                        self.theme_mode.toggle();
                    }
                    ui.separator();
                    let grid_lbl = if self.canvas.show_grid {
                        "Grid: Hide"
                    } else {
                        "Grid: Show"
                    };
                    if ui.button(grid_lbl).clicked() {
                        self.canvas.show_grid = !self.canvas.show_grid;
                    }
                    let snap_lbl = if self.canvas.snap_to_grid {
                        "Snap: Disable"
                    } else {
                        "Snap: Enable"
                    };
                    if ui.button(snap_lbl).clicked() {
                        self.canvas.snap_to_grid = !self.canvas.snap_to_grid;
                    }
                    ui.separator();
                    if ui.button("Reset Zoom (100%)").clicked() {
                        self.canvas.zoom = 1.0;
                    }
                    if ui.button("Center Viewport").clicked() {
                        self.canvas.pan = egui::Vec2::new(300.0, 200.0);
                    }
                });

                // Simulation Menu
                ui.menu_button("Simulation", |ui| {
                    let clock_btn_text = if self.clock_running {
                        "Pause Clock"
                    } else {
                        "Run Clock (2Hz)"
                    };
                    if ui.button(clock_btn_text).clicked() {
                        self.clock_running = !self.clock_running;
                    }
                    if ui.button("Step Tick Clock").clicked() {
                        Simulator::tick(&mut self.circuit, ClockEdge::Rising);
                        self.waveform.sample_circuit(&self.circuit);
                        self.status_message = "Clock ticked".to_string();
                    }
                    ui.separator();
                    if ui.button("Settle Circuit Signals").clicked() {
                        Simulator::settle(&mut self.circuit);
                        self.status_message = "Circuit settled".to_string();
                    }
                });

                // Tools Menu
                ui.menu_button("Tools", |ui| {
                    if ui
                        .button(
                            RichText::new("Truth Table & Verification").color(Theme::ACCENT_PINK),
                        )
                        .clicked()
                    {
                        ui.close();
                        self.verification_ui.is_open = true;
                        self.verification_ui.generate_from_circuit(
                            &self.circuit,
                            &self.canvas.selection.selected_components,
                        );
                    }
                    let wave_lbl = if self.waveform.is_open {
                        "Hide Timing Waveform"
                    } else {
                        "Show Timing Waveform"
                    };
                    if ui.button(wave_lbl).clicked() {
                        self.waveform.is_open = !self.waveform.is_open;
                    }
                    ui.separator();
                    ui.menu_button("Universal Gates Lab", |ui| {
                        for &ch in UniversalChallenge::ALL {
                            let is_active = self.verification_ui.active_challenge == Some(ch);
                            if ui.selectable_label(is_active, ch.title()).clicked() {
                                ui.close();
                                self.push_undo();
                                self.verification_ui.load_challenge(ch, &mut self.circuit);
                                self.status_message = format!("Loaded Lab: {}", ch.title());
                            }
                        }
                    });
                });

                // Right-aligned quick indicators
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Theme toggle button
                    let theme_txt = if self.theme_mode.is_dark() {
                        "Theme: Dark"
                    } else {
                        "Theme: Light"
                    };
                    if ui
                        .button(RichText::new(theme_txt).color(Theme::ACCENT_PINK))
                        .on_hover_text("Toggle Theme (Ctrl+T / F8)")
                        .clicked()
                    {
                        self.theme_mode.toggle();
                    }

                    ui.separator();

                    // Clock indicator badge
                    let (clk_text, clk_color) = if self.clock_running {
                        ("CLK: 2Hz [RUNNING]", Theme::ACCENT_PINK)
                    } else {
                        ("CLK: [PAUSED]", Theme::TEXT_MUTED)
                    };
                    if ui
                        .button(RichText::new(clk_text).color(clk_color).size(11.0))
                        .on_hover_text("Click to toggle clock run/pause")
                        .clicked()
                    {
                        self.clock_running = !self.clock_running;
                    }

                    ui.separator();

                    // Zoom indicator badge
                    let zoom_text = format!("{:.0}%", self.canvas.zoom * 100.0);
                    if ui
                        .button(
                            RichText::new(zoom_text)
                                .color(Theme::ACCENT_PURPLE)
                                .size(11.0),
                        )
                        .on_hover_text("Click to reset zoom to 100%")
                        .clicked()
                    {
                        self.canvas.zoom = 1.0;
                    }

                    // Selection badge (if items selected)
                    let sel_count = self.canvas.selection.selected_components.len();
                    if sel_count > 0 {
                        ui.separator();
                        let sel_lbl = format!("{} selected", sel_count);
                        if ui
                            .button(
                                RichText::new(sel_lbl)
                                    .color(Theme::ACCENT_PINK)
                                    .strong()
                                    .size(11.0),
                            )
                            .on_hover_text("Click to package selection into subcircuit (Ctrl+G)")
                            .clicked()
                        {
                            self.new_subcircuit_name =
                                format!("IC_{}", self.circuit.subcircuits.len() + 1);
                            self.subcircuit_modal_open = true;
                        }
                    }
                });
            });
        });

        // Universal Lab Banner (if active)
        if let Some(ch) = self.verification_ui.active_challenge {
            let banner_frame = Frame::side_top_panel(ui.style()).fill(Theme::BG_PANEL_RAISED);
            Panel::top("lab_active_banner")
                .frame(banner_frame)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("CHALLENGE: {}", ch.title()))
                                .strong()
                                .color(Theme::ACCENT_PINK),
                        );
                        ui.label(
                            RichText::new(ch.description())
                                .color(Theme::TEXT_PRIMARY)
                                .size(12.0),
                        );

                        if ui
                            .button(
                                RichText::new("Verify Solution")
                                    .color(Theme::ACCENT_PINK)
                                    .strong(),
                            )
                            .clicked()
                        {
                            self.verification_ui.verify_current_challenge(&self.circuit);
                        }

                        if let Some((passed, ref msg)) = self.verification_ui.challenge_feedback {
                            let color = if passed {
                                Theme::ACCENT_PINK
                            } else {
                                Theme::ACCENT_RED
                            };
                            ui.label(RichText::new(msg).color(color).strong());
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Exit Lab").clicked() {
                                self.verification_ui.active_challenge = None;
                                self.verification_ui.challenge_feedback = None;
                            }
                        });
                    });
                });
        }

        // 2. Bottom Status Bar Panel
        let bottom_frame = Theme::glass_panel();
        Panel::bottom("bottom_status_bar")
            .frame(bottom_frame)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&self.status_message)
                            .color(Theme::TEXT_PRIMARY)
                            .size(12.0),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "Components: {} | Nets: {} | Zoom: {:.0}% | {}",
                                self.circuit.components.len(),
                                self.circuit.nets.len(),
                                self.canvas.zoom * 100.0,
                                self.theme_mode.label()
                            ))
                            .color(Theme::ACCENT_PURPLE)
                            .size(11.0),
                        );
                    });
                });
            });

        // 3. Bottom Waveform Panel (if active)
        if self.waveform.is_open {
            let wave_frame = Theme::glass_panel();
            Panel::bottom("waveform_bottom_panel")
                .frame(wave_frame)
                .resizable(true)
                .default_size(200.0)
                .show(ui, |ui| {
                    self.waveform.show(ui, &self.circuit);
                });
        }

        // 4. Left Palette Panel
        let left_frame = Theme::glass_panel();
        Panel::left("palette_panel")
            .frame(left_frame)
            .resizable(true)
            .default_size(220.0)
            .min_size(190.0)
            .max_size(320.0)
            .show(ui, |ui| {
                Palette::show(ui, &mut self.selected_for_placement, &self.circuit);
            });

        // 5. Central Canvas Panel
        let central_frame = Frame::central_panel(ui.style()).fill(self.theme_mode.bg_canvas());
        CentralPanel::default().frame(central_frame).show(ui, |ui| {
            let canvas_res = self.canvas.show(
                ui,
                &mut self.circuit,
                &mut self.selected_for_placement,
                self.theme_mode,
            );
            if canvas_res.placed_component || canvas_res.circuit_mutated {
                self.waveform.sample_circuit(&self.circuit);
                self.push_undo();
            }
        });

        // 5. Verification & Truth Table Modal
        self.verification_ui.show(
            ui.ctx(),
            &mut self.circuit,
            &self.canvas.selection.selected_components,
        );

        // 6. Subcircuit Packaging Modal
        if self.subcircuit_modal_open {
            let mut is_open = true;
            let modal_frame = Theme::glass_modal();

            egui::Window::new("Package Selection as Subcircuit")
                .open(&mut is_open)
                .frame(modal_frame)
                .resizable(false)
                .collapsible(false)
                .default_size(egui::Vec2::new(320.0, 180.0))
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.label(
                        RichText::new("Package selected components into a reusable IC block.")
                            .size(12.0)
                            .color(Theme::TEXT_PRIMARY.gamma_multiply(0.7)),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.new_subcircuit_name);
                    });
                    ui.add_space(8.0);
                    let sel_count = self.canvas.selection.selected_components.len();
                    ui.label(
                        RichText::new(format!("Includes {} selected components", sel_count))
                            .size(11.0)
                            .color(Theme::ACCENT_PURPLE),
                    );
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                RichText::new("Create Subcircuit")
                                    .color(Theme::ACCENT_PINK)
                                    .strong(),
                            )
                            .clicked()
                        {
                            let name = self.new_subcircuit_name.trim().to_string();
                            if !name.is_empty() {
                                self.package_selection_into_subcircuit(name);
                                self.subcircuit_modal_open = false;
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.subcircuit_modal_open = false;
                        }
                    });
                });
            if !is_open {
                self.subcircuit_modal_open = false;
            }
        }

        // 7. HDL Export Modal
        self.hdl_ui.show(ui.ctx(), &self.circuit);
    }
}

use crate::theme::Theme;
use eframe::egui::{self, Color32, FontId, Pos2, RichText, Sense, Stroke, Ui, Vec2};
use logic_core::{Circuit, ComponentId, GateKind, Signal};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct SignalHistory {
    pub label: String,
    pub samples: VecDeque<Signal>,
}

#[derive(Debug, Clone)]
pub struct WaveformState {
    pub is_open: bool,
    pub is_recording: bool,
    pub step_counter: usize,
    pub max_samples: usize,
    pub track_data: HashMap<ComponentId, SignalHistory>,
    pub zoom_x: f32,
}

impl Default for WaveformState {
    fn default() -> Self {
        Self {
            is_open: false,
            is_recording: true,
            step_counter: 0,
            max_samples: 80,
            track_data: HashMap::new(),
            zoom_x: 20.0,
        }
    }
}

impl WaveformState {
    /// Records a snapshot sample of all traced circuit nodes
    pub fn sample_circuit(&mut self, circuit: &Circuit) {
        if !self.is_recording {
            return;
        }

        self.step_counter += 1;

        // Auto-discover traceable components: Clocks, Switches, LEDs, Flip-Flops
        for (id, comp) in &circuit.components {
            let should_trace = matches!(
                comp.kind,
                GateKind::Clock
                    | GateKind::ToggleSwitch
                    | GateKind::Led
                    | GateKind::DFlipFlop
                    | GateKind::JkFlipFlop
                    | GateKind::TFlipFlop
                    | GateKind::Counter4
            );

            if should_trace {
                let sig =
                    comp.output_signals.first().copied().unwrap_or_else(|| {
                        comp.input_signals.first().copied().unwrap_or(Signal::Zero)
                    });

                let entry = self.track_data.entry(id).or_insert_with(|| {
                    let label = format!("{}_{:?}", comp.kind.display_name(), id);
                    SignalHistory {
                        label,
                        samples: VecDeque::new(),
                    }
                });

                entry.samples.push_back(sig);
                if entry.samples.len() > self.max_samples {
                    entry.samples.pop_front();
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.step_counter = 0;
        self.track_data.clear();
    }

    /// Renders the logic analyzer timing waveform UI in a bottom drawer or panel
    pub fn show(&mut self, ui: &mut Ui, circuit: &Circuit) {
        if !self.is_open {
            return;
        }

        ui.horizontal(|ui| {
            ui.visuals_mut().override_text_color = Some(Theme::TEXT_PRIMARY);
            ui.label(
                RichText::new("Timing Waveform (Logic Analyzer)")
                    .strong()
                    .color(Theme::ACCENT_PINK),
            );

            ui.separator();
            let rec_text = if self.is_recording {
                "Pause Recording"
            } else {
                "Resume Recording"
            };
            if ui.button(rec_text).clicked() {
                self.is_recording = !self.is_recording;
            }

            if ui.button("Clear History").clicked() {
                self.clear();
            }

            ui.separator();
            ui.label(format!("Samples: {}", self.step_counter));

            ui.add(egui::Slider::new(&mut self.zoom_x, 10.0..=40.0).text("Time Scale"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    self.is_open = false;
                }
            });
        });

        ui.separator();

        if self.track_data.is_empty() {
            ui.label(RichText::new("No activity recorded yet. Run the clock or flip switches to see waveform transitions.").color(Theme::TEXT_PRIMARY.gamma_multiply(0.7)));
            return;
        }

        let total_tracks = self.track_data.len();
        let track_height = 36.0;
        let header_height = 24.0;
        let total_height = header_height + (total_tracks as f32 * track_height) + 16.0;

        let available_size = Vec2::new(ui.available_width(), total_height.min(220.0));
        let (rect, response) = ui.allocate_exact_size(available_size, Sense::hover());
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 4.0, Theme::BG_PANEL_RAISED);
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Theme::ACCENT_PURPLE),
            egui::StrokeKind::Inside,
        );

        let label_width = 140.0;
        let waveform_start_x = rect.min.x + label_width;

        // Draw track labels on left and waveform lanes on right
        let mut sorted_keys: Vec<_> = self.track_data.keys().copied().collect();
        sorted_keys.sort_by_key(|id| {
            circuit
                .components
                .get(*id)
                .map(|c| c.kind.display_name())
                .unwrap_or_default()
        });

        // 1. Time header tick marks
        let time_axis_y = rect.min.y + 16.0;
        let mut sample_idx = 0;
        let mut cur_x = waveform_start_x;
        while cur_x < rect.max.x {
            painter.line_segment(
                [
                    Pos2::new(cur_x, rect.min.y + 6.0),
                    Pos2::new(cur_x, rect.max.y - 6.0),
                ],
                Stroke::new(1.0, Theme::GRID_LINE.gamma_multiply(0.2)),
            );

            if sample_idx % 5 == 0 {
                painter.text(
                    Pos2::new(cur_x, time_axis_y),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", sample_idx),
                    FontId::monospace(9.0),
                    Theme::TEXT_PRIMARY.gamma_multiply(0.6),
                );
            }
            cur_x += self.zoom_x;
            sample_idx += 1;
        }

        // 2. Render each signal track
        for (i, comp_id) in sorted_keys.iter().enumerate() {
            let track = match self.track_data.get(comp_id) {
                Some(t) => t,
                None => continue,
            };

            let track_top_y = rect.min.y + header_height + (i as f32 * track_height);
            let y_high = track_top_y + 8.0;
            let y_low = track_top_y + track_height - 8.0;

            // Track label on the left
            let label_pos = Pos2::new(rect.min.x + 8.0, track_top_y + (track_height * 0.5));
            let short_label = if track.label.len() > 16 {
                format!("{}...", &track.label[..14])
            } else {
                track.label.clone()
            };
            painter.text(
                label_pos,
                egui::Align2::LEFT_CENTER,
                short_label,
                FontId::monospace(11.0),
                Theme::TEXT_PRIMARY,
            );

            // Subtle lane separator
            painter.line_segment(
                [
                    Pos2::new(rect.min.x + 4.0, track_top_y + track_height),
                    Pos2::new(rect.max.x - 4.0, track_top_y + track_height),
                ],
                Stroke::new(1.0, Theme::ACCENT_PURPLE.gamma_multiply(0.2)),
            );

            // Draw digital square-wave transitions
            let mut prev_sig = None;
            let mut prev_pos = None;

            for (s_idx, &sig) in track.samples.iter().enumerate() {
                let x_start = waveform_start_x + (s_idx as f32 * self.zoom_x);
                let x_end = x_start + self.zoom_x;

                if x_start > rect.max.x {
                    break;
                }

                let y_val = match sig {
                    Signal::One => y_high,
                    Signal::Zero => y_low,
                    _ => (y_high + y_low) * 0.5,
                };

                let wire_color = match sig {
                    Signal::One => Theme::SIGNAL_HIGH,
                    Signal::Zero => Theme::SIGNAL_LOW,
                    Signal::X => Theme::SIGNAL_X,
                    Signal::Z => Color32::from_gray(100),
                };

                let current_start_pos = Pos2::new(x_start, y_val);
                let current_end_pos = Pos2::new(x_end, y_val);

                // Vertical transition edge if state changed
                if let Some(prev_p) = prev_pos
                    && prev_sig != Some(sig)
                {
                    painter.line_segment([prev_p, current_start_pos], Stroke::new(1.5, wire_color));
                }

                // Horizontal duration line
                painter.line_segment(
                    [current_start_pos, current_end_pos],
                    Stroke::new(1.8, wire_color),
                );

                prev_sig = Some(sig);
                prev_pos = Some(current_end_pos);
            }
        }

        // Hover cursor line tracking time
        if let Some(hover_pos) = response.hover_pos()
            && hover_pos.x >= waveform_start_x
            && hover_pos.x <= rect.max.x
        {
            painter.line_segment(
                [
                    Pos2::new(hover_pos.x, rect.min.y + 4.0),
                    Pos2::new(hover_pos.x, rect.max.y - 4.0),
                ],
                Stroke::new(1.0, Theme::ACCENT_PINK.gamma_multiply(0.6)),
            );

            let sample_at_cursor =
                ((hover_pos.x - waveform_start_x) / self.zoom_x).floor() as usize;
            painter.text(
                Pos2::new(hover_pos.x, rect.min.y + 8.0),
                egui::Align2::CENTER_TOP,
                format!("T={}", sample_at_cursor),
                FontId::monospace(10.0),
                Theme::ACCENT_PINK,
            );
        }
    }
}

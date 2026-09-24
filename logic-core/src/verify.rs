use crate::component::GateKind;
use crate::graph::{Circuit, ComponentId, PortEndpoint};
use crate::signal::Signal;
use crate::sim::Simulator;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VerifyError {
    #[error("No inputs found in the circuit to generate truth table")]
    NoInputs,
    #[error("No outputs found in the circuit to evaluate")]
    NoOutputs,
    #[error("Too many inputs ({0}); maximum supported is 10")]
    TooManyInputs(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TruthTableRow {
    pub inputs: Vec<Signal>,
    pub outputs: Vec<Signal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TruthTable {
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
    pub rows: Vec<TruthTableRow>,
}

impl TruthTable {
    pub fn to_csv(&self) -> String {
        let mut s = String::new();
        let headers: Vec<&str> = self
            .input_names
            .iter()
            .chain(self.output_names.iter())
            .map(|x| x.as_str())
            .collect();
        s.push_str(&headers.join(","));
        s.push('\n');

        for row in &self.rows {
            let mut line = Vec::new();
            for sig in &row.inputs {
                line.push(if sig.is_high() { "1" } else { "0" });
            }
            for sig in &row.outputs {
                line.push(if sig.is_high() { "1" } else { "0" });
            }
            s.push_str(&line.join(","));
            s.push('\n');
        }
        s
    }

    pub fn to_plain_text(&self) -> String {
        let mut s = String::new();
        let in_headers = self.input_names.join(" | ");
        let out_headers = self.output_names.join(" | ");
        let header_line = format!(" {} || {} ", in_headers, out_headers);
        let divider = "-".repeat(header_line.len());

        s.push_str(&divider);
        s.push('\n');
        s.push_str(&header_line);
        s.push('\n');
        s.push_str(&divider);
        s.push('\n');

        for row in &self.rows {
            let ins: Vec<String> = row
                .inputs
                .iter()
                .enumerate()
                .map(|(i, sig)| {
                    let name_len = self.input_names.get(i).map(|n| n.len()).unwrap_or(1);
                    format!(
                        "{:^width$}",
                        if sig.is_high() { "1" } else { "0" },
                        width = name_len
                    )
                })
                .collect();
            let outs: Vec<String> = row
                .outputs
                .iter()
                .enumerate()
                .map(|(i, sig)| {
                    let name_len = self.output_names.get(i).map(|n| n.len()).unwrap_or(1);
                    format!(
                        "{:^width$}",
                        if sig.is_high() { "1" } else { "0" },
                        width = name_len
                    )
                })
                .collect();

            s.push_str(&format!(" {} || {} \n", ins.join(" | "), outs.join(" | ")));
        }
        s.push_str(&divider);
        s.push('\n');
        s
    }

    pub fn diff(&self, expected: &TruthTable) -> TruthTableDiff {
        let mut mismatches = Vec::new();

        for (idx, actual_row) in self.rows.iter().enumerate() {
            if let Some(exp_row) = expected.rows.get(idx) {
                if actual_row.outputs != exp_row.outputs {
                    mismatches.push(MismatchRow {
                        row_index: idx,
                        inputs: actual_row.inputs.clone(),
                        expected: exp_row.outputs.clone(),
                        actual: actual_row.outputs.clone(),
                    });
                }
            } else {
                mismatches.push(MismatchRow {
                    row_index: idx,
                    inputs: actual_row.inputs.clone(),
                    expected: vec![Signal::X; actual_row.outputs.len()],
                    actual: actual_row.outputs.clone(),
                });
            }
        }

        TruthTableDiff {
            total_rows: self.rows.len(),
            is_match: mismatches.is_empty(),
            mismatches,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MismatchRow {
    pub row_index: usize,
    pub inputs: Vec<Signal>,
    pub expected: Vec<Signal>,
    pub actual: Vec<Signal>,
}

#[derive(Debug, Clone)]
pub struct TruthTableDiff {
    pub total_rows: usize,
    pub is_match: bool,
    pub mismatches: Vec<MismatchRow>,
}

pub struct CircuitIO {
    pub input_switches: Vec<(ComponentId, String)>,
    pub output_leds: Vec<(ComponentId, String)>,
}

impl CircuitIO {
    pub fn detect(
        circuit: &Circuit,
        filter_components: Option<&HashSet<ComponentId>>,
    ) -> Result<Self, VerifyError> {
        let mut input_switches = Vec::new();
        let mut output_leds = Vec::new();

        for (id, comp) in &circuit.components {
            if let Some(filter) = filter_components
                && !filter.contains(&id)
            {
                continue;
            }

            match comp.kind {
                GateKind::ToggleSwitch => {
                    let label = format!("IN_{}", input_switches.len());
                    input_switches.push((id, label));
                }
                GateKind::Led => {
                    let label = format!("OUT_{}", output_leds.len());
                    output_leds.push((id, label));
                }
                _ => {}
            }
        }

        // If no switches were explicitly in the selection/circuit, detect unconnected input ports
        if input_switches.is_empty() {
            for (id, comp) in &circuit.components {
                if let Some(filter) = filter_components
                    && !filter.contains(&id)
                {
                    continue;
                }
                for in_idx in 0..comp.kind.input_count() {
                    let ep = PortEndpoint {
                        component_id: id,
                        is_output: false,
                        port_index: in_idx,
                    };
                    let is_connected = circuit.find_net_driving_sink(ep).is_some();
                    if !is_connected {
                        let label = format!("{}_{}", comp.kind.short_code(), in_idx);
                        input_switches.push((id, label));
                    }
                }
            }
        }

        // If no LEDs found, detect outputs with no downstream sink
        if output_leds.is_empty() {
            for (id, comp) in &circuit.components {
                if let Some(filter) = filter_components
                    && !filter.contains(&id)
                {
                    continue;
                }
                for out_idx in 0..comp.kind.output_count() {
                    let label = format!("{}_Q{}", comp.kind.short_code(), out_idx);
                    output_leds.push((id, label));
                }
            }
        }

        if input_switches.is_empty() {
            return Err(VerifyError::NoInputs);
        }
        if output_leds.is_empty() {
            return Err(VerifyError::NoOutputs);
        }
        if input_switches.len() > 10 {
            return Err(VerifyError::TooManyInputs(input_switches.len()));
        }

        // Rename single inputs/outputs to clean A, B, Q
        if input_switches.len() == 1 {
            input_switches[0].1 = "A".to_string();
        } else if input_switches.len() == 2 {
            input_switches[0].1 = "A".to_string();
            input_switches[1].1 = "B".to_string();
        } else if input_switches.len() == 3 {
            input_switches[0].1 = "A".to_string();
            input_switches[1].1 = "B".to_string();
            input_switches[2].1 = "C".to_string();
        } else if input_switches.len() == 4 {
            input_switches[0].1 = "A".to_string();
            input_switches[1].1 = "B".to_string();
            input_switches[2].1 = "C".to_string();
            input_switches[3].1 = "D".to_string();
        }

        if output_leds.len() == 1 {
            output_leds[0].1 = "Q".to_string();
        }

        Ok(Self {
            input_switches,
            output_leds,
        })
    }
}

pub struct TruthTableGenerator;

impl TruthTableGenerator {
    pub fn generate(
        circuit: &Circuit,
        filter_components: Option<&HashSet<ComponentId>>,
    ) -> Result<(TruthTable, CircuitIO), VerifyError> {
        let io = CircuitIO::detect(circuit, filter_components)?;
        let n_inputs = io.input_switches.len();
        let total_combinations = 1 << n_inputs;

        let input_names: Vec<String> = io.input_switches.iter().map(|(_, n)| n.clone()).collect();
        let output_names: Vec<String> = io.output_leds.iter().map(|(_, n)| n.clone()).collect();

        let mut rows = Vec::with_capacity(total_combinations);

        for combo in 0..total_combinations {
            let mut sim_circuit = circuit.clone();

            // Drive inputs
            let mut row_inputs = Vec::with_capacity(n_inputs);
            for (bit_idx, (sw_id, _)) in io.input_switches.iter().enumerate() {
                // MSB first: (n_inputs - 1 - bit_idx)
                let bit_val = ((combo >> (n_inputs - 1 - bit_idx)) & 1) == 1;
                let sig = if bit_val { Signal::One } else { Signal::Zero };
                row_inputs.push(sig);

                if let Some(comp) = sim_circuit.components.get_mut(*sw_id) {
                    if comp.kind == GateKind::ToggleSwitch {
                        comp.state_flag = bit_val;
                    } else if comp.input_signals.len() > bit_idx {
                        comp.input_signals[bit_idx] = sig;
                    }
                }
            }

            Simulator::settle(&mut sim_circuit);

            // Read outputs
            let mut row_outputs = Vec::with_capacity(io.output_leds.len());
            for (led_id, _) in &io.output_leds {
                if let Some(comp) = sim_circuit.components.get(*led_id) {
                    let out_sig = if comp.kind == GateKind::Led {
                        comp.input_signals.first().copied().unwrap_or(Signal::Zero)
                    } else {
                        comp.output_signals.first().copied().unwrap_or(Signal::Zero)
                    };
                    row_outputs.push(out_sig);
                } else {
                    row_outputs.push(Signal::Zero);
                }
            }

            rows.push(TruthTableRow {
                inputs: row_inputs,
                outputs: row_outputs,
            });
        }

        Ok((
            TruthTable {
                input_names,
                output_names,
                rows,
            },
            io,
        ))
    }

    /// Reference truth tables for standard target logic functions
    pub fn reference_table(kind: GateKind) -> TruthTable {
        match kind {
            GateKind::Not => TruthTable {
                input_names: vec!["A".to_string()],
                output_names: vec!["Q".to_string()],
                rows: vec![
                    TruthTableRow {
                        inputs: vec![Signal::Zero],
                        outputs: vec![Signal::One],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One],
                        outputs: vec![Signal::Zero],
                    },
                ],
            },
            GateKind::And => TruthTable {
                input_names: vec!["A".to_string(), "B".to_string()],
                output_names: vec!["Q".to_string()],
                rows: vec![
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::Zero],
                        outputs: vec![Signal::Zero],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::One],
                        outputs: vec![Signal::Zero],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::Zero],
                        outputs: vec![Signal::Zero],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::One],
                        outputs: vec![Signal::One],
                    },
                ],
            },
            GateKind::Or => TruthTable {
                input_names: vec!["A".to_string(), "B".to_string()],
                output_names: vec!["Q".to_string()],
                rows: vec![
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::Zero],
                        outputs: vec![Signal::Zero],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::One],
                        outputs: vec![Signal::One],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::Zero],
                        outputs: vec![Signal::One],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::One],
                        outputs: vec![Signal::One],
                    },
                ],
            },
            GateKind::Xor => TruthTable {
                input_names: vec!["A".to_string(), "B".to_string()],
                output_names: vec!["Q".to_string()],
                rows: vec![
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::Zero],
                        outputs: vec![Signal::Zero],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::One],
                        outputs: vec![Signal::One],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::Zero],
                        outputs: vec![Signal::One],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::One],
                        outputs: vec![Signal::Zero],
                    },
                ],
            },
            GateKind::Nand => TruthTable {
                input_names: vec!["A".to_string(), "B".to_string()],
                output_names: vec!["Q".to_string()],
                rows: vec![
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::Zero],
                        outputs: vec![Signal::One],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::One],
                        outputs: vec![Signal::One],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::Zero],
                        outputs: vec![Signal::One],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::One],
                        outputs: vec![Signal::Zero],
                    },
                ],
            },
            GateKind::Nor => TruthTable {
                input_names: vec!["A".to_string(), "B".to_string()],
                output_names: vec!["Q".to_string()],
                rows: vec![
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::Zero],
                        outputs: vec![Signal::One],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::One],
                        outputs: vec![Signal::Zero],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::Zero],
                        outputs: vec![Signal::Zero],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::One],
                        outputs: vec![Signal::Zero],
                    },
                ],
            },
            GateKind::Xnor => TruthTable {
                input_names: vec!["A".to_string(), "B".to_string()],
                output_names: vec!["Q".to_string()],
                rows: vec![
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::Zero],
                        outputs: vec![Signal::One],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::Zero, Signal::One],
                        outputs: vec![Signal::Zero],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::Zero],
                        outputs: vec![Signal::Zero],
                    },
                    TruthTableRow {
                        inputs: vec![Signal::One, Signal::One],
                        outputs: vec![Signal::One],
                    },
                ],
            },
            _ => TruthTable {
                input_names: vec![],
                output_names: vec![],
                rows: vec![],
            },
        }
    }
}

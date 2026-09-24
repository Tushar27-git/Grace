use crate::component::GateKind;
use crate::graph::{Circuit, ComponentId, PortEndpoint};
use std::collections::{HashMap, HashSet};

type CircuitIoPorts = (Vec<(ComponentId, String)>, Vec<(ComponentId, String)>);

pub struct HdlExporter;

impl HdlExporter {
    /// Generates standard, synthesizable Verilog code for the circuit.
    pub fn to_verilog(circuit: &Circuit, module_name: &str) -> String {
        let clean_mod_name = sanitize_identifier(module_name);
        let mut code = String::new();

        code.push_str(
            "// =============================================================================\n",
        );
        code.push_str(&format!("// Logic Lab HDL Export: {}\n", clean_mod_name));
        code.push_str("// Generated automatically by Logic Lab\n");
        code.push_str(
            "// =============================================================================\n\n",
        );

        // First, if there are subcircuits, emit subcircuit modules recursively
        for (sub_name, sub_def) in &circuit.subcircuits {
            code.push_str(&Self::to_verilog(&sub_def.circuit, sub_name));
            code.push('\n');
        }

        // Identify inputs (ToggleSwitch, Clock) and outputs (Led, etc.)
        let (inputs, outputs) = Self::detect_circuit_io(circuit);

        code.push_str(&format!("module {} (\n", clean_mod_name));
        let mut port_decls = Vec::new();
        for (id, name) in &inputs {
            let safe_name = format!("{}_{}", sanitize_identifier(name), id_suffix(*id));
            port_decls.push(format!("    input  wire {}", safe_name));
        }
        for (id, name) in &outputs {
            let safe_name = format!("{}_{}", sanitize_identifier(name), id_suffix(*id));
            port_decls.push(format!("    output wire {}", safe_name));
        }

        if port_decls.is_empty() {
            port_decls.push("    // No external ports connected".to_string());
        }
        code.push_str(&port_decls.join(",\n"));
        code.push_str("\n);\n\n");

        // Net wire map: PortEndpoint -> wire name
        let mut wire_map: HashMap<PortEndpoint, String> = HashMap::new();
        let mut internal_wires: HashSet<String> = HashSet::new();

        // Map inputs to port names
        for (id, name) in &inputs {
            let safe_name = format!("{}_{}", sanitize_identifier(name), id_suffix(*id));
            wire_map.insert(
                PortEndpoint {
                    component_id: *id,
                    is_output: true,
                    port_index: 0,
                },
                safe_name,
            );
        }

        // Map outputs to port names
        for (id, name) in &outputs {
            let safe_name = format!("{}_{}", sanitize_identifier(name), id_suffix(*id));
            wire_map.insert(
                PortEndpoint {
                    component_id: *id,
                    is_output: false,
                    port_index: 0,
                },
                safe_name,
            );
        }

        // Internal wires for connected nets
        for (nid, net) in &circuit.nets {
            let net_name = if let Some(ref lbl) = net.label {
                sanitize_identifier(lbl)
            } else {
                format!("wire_{:?}", nid)
            };

            let src_name = wire_map.get(&net.source).cloned().unwrap_or_else(|| {
                internal_wires.insert(net_name.clone());
                net_name.clone()
            });

            wire_map.insert(net.source, src_name.clone());

            for sink in &net.sinks {
                if let Some(out_port_name) = wire_map.get(sink).cloned() {
                    // Driven output port: connect to net source
                    code.push_str(&format!("    assign {} = {};\n", out_port_name, src_name));
                } else {
                    wire_map.insert(*sink, src_name.clone());
                }
            }
        }

        // Declare internal wires
        if !internal_wires.is_empty() {
            code.push_str("    // Internal signals\n");
            let mut sorted_wires: Vec<_> = internal_wires.into_iter().collect();
            sorted_wires.sort();
            for w in sorted_wires {
                code.push_str(&format!("    wire {};\n", w));
            }
            code.push('\n');
        }

        // Instantiate logic gates and blocks
        code.push_str("    // Component Instantiations\n");
        for (cid, comp) in &circuit.components {
            let sid = id_suffix(cid);
            let in_sig = |p: usize| {
                wire_map
                    .get(&PortEndpoint {
                        component_id: cid,
                        is_output: false,
                        port_index: p,
                    })
                    .cloned()
                    .unwrap_or_else(|| "1'b0".to_string())
            };
            let out_sig = |p: usize| {
                wire_map
                    .get(&PortEndpoint {
                        component_id: cid,
                        is_output: true,
                        port_index: p,
                    })
                    .cloned()
                    .unwrap_or_else(|| format!("open_out_{}_{}", sid, p))
            };

            match &comp.kind {
                GateKind::And => {
                    code.push_str(&format!(
                        "    and  gate_and_{} ({}, {}, {});\n",
                        sid,
                        out_sig(0),
                        in_sig(0),
                        in_sig(1)
                    ));
                }
                GateKind::Or => {
                    code.push_str(&format!(
                        "    or   gate_or_{} ({}, {}, {});\n",
                        sid,
                        out_sig(0),
                        in_sig(0),
                        in_sig(1)
                    ));
                }
                GateKind::Not => {
                    code.push_str(&format!(
                        "    not  gate_not_{} ({}, {});\n",
                        sid,
                        out_sig(0),
                        in_sig(0)
                    ));
                }
                GateKind::Nand => {
                    code.push_str(&format!(
                        "    nand gate_nand_{} ({}, {}, {});\n",
                        sid,
                        out_sig(0),
                        in_sig(0),
                        in_sig(1)
                    ));
                }
                GateKind::Nor => {
                    code.push_str(&format!(
                        "    nor  gate_nor_{} ({}, {}, {});\n",
                        sid,
                        out_sig(0),
                        in_sig(0),
                        in_sig(1)
                    ));
                }
                GateKind::Xor => {
                    code.push_str(&format!(
                        "    xor  gate_xor_{} ({}, {}, {});\n",
                        sid,
                        out_sig(0),
                        in_sig(0),
                        in_sig(1)
                    ));
                }
                GateKind::Xnor => {
                    code.push_str(&format!(
                        "    xnor gate_xnor_{} ({}, {}, {});\n",
                        sid,
                        out_sig(0),
                        in_sig(0),
                        in_sig(1)
                    ));
                }
                GateKind::HalfAdder => {
                    code.push_str(&format!(
                        "    assign {{{}, {}}} = {} + {};\n",
                        out_sig(1),
                        out_sig(0),
                        in_sig(0),
                        in_sig(1)
                    ));
                }
                GateKind::FullAdder => {
                    code.push_str(&format!(
                        "    assign {{{}, {}}} = {} + {} + {};\n",
                        out_sig(1),
                        out_sig(0),
                        in_sig(0),
                        in_sig(1),
                        in_sig(2)
                    ));
                }
                GateKind::HalfSubtractor => {
                    code.push_str(&format!(
                        "    assign {{{}, {}}} = {} - {};\n",
                        out_sig(1),
                        out_sig(0),
                        in_sig(0),
                        in_sig(1)
                    ));
                }
                GateKind::FullSubtractor => {
                    code.push_str(&format!(
                        "    assign {{{}, {}}} = {} - {} - {};\n",
                        out_sig(1),
                        out_sig(0),
                        in_sig(0),
                        in_sig(1),
                        in_sig(2)
                    ));
                }
                GateKind::RippleCarryAdder4 | GateKind::CarryLookaheadAdder4 => {
                    code.push_str(&format!(
                        "    assign {{{}, {}, {}, {}, {}}} = {{{}, {}, {}, {}}} + {{{}, {}, {}, {}}} + {};\n",
                        out_sig(4), out_sig(3), out_sig(2), out_sig(1), out_sig(0),
                        in_sig(3), in_sig(2), in_sig(1), in_sig(0),
                        in_sig(7), in_sig(6), in_sig(5), in_sig(4),
                        in_sig(8)
                    ));
                }
                GateKind::Mux2 => {
                    code.push_str(&format!(
                        "    assign {} = {} ? {} : {};\n",
                        out_sig(0),
                        in_sig(2),
                        in_sig(1),
                        in_sig(0)
                    ));
                }
                GateKind::DFlipFlop => {
                    code.push_str(&format!(
                        "    reg reg_{sid} = 0;\n    always @(posedge {}) reg_{sid} <= {};\n    assign {} = reg_{sid};\n    assign {} = ~reg_{sid};\n",
                        in_sig(1), in_sig(0), out_sig(0), out_sig(1)
                    ));
                }
                GateKind::SubcircuitInstance(sub_name) => {
                    let clean_sub = sanitize_identifier(sub_name);
                    code.push_str(&format!("    {} inst_{}_{} (\n", clean_sub, clean_sub, sid));
                    let mut conns = Vec::new();
                    for i in 0..comp.input_signals.len() {
                        conns.push(format!("        .in_{}({})", i, in_sig(i)));
                    }
                    for i in 0..comp.output_signals.len() {
                        conns.push(format!("        .out_{}({})", i, out_sig(i)));
                    }
                    code.push_str(&conns.join(",\n"));
                    code.push_str("\n    );\n");
                }
                _ => {}
            }
        }

        code.push_str("\nendmodule\n");
        code
    }

    /// Generates standard IEEE 1164 VHDL code for the circuit.
    pub fn to_vhdl(circuit: &Circuit, entity_name: &str) -> String {
        let clean_ent_name = sanitize_identifier(entity_name);
        let mut code = String::new();

        code.push_str(
            "-- =============================================================================\n",
        );
        code.push_str(&format!("-- Logic Lab VHDL Export: {}\n", clean_ent_name));
        code.push_str("-- Generated automatically by Logic Lab\n");
        code.push_str(
            "-- =============================================================================\n\n",
        );
        code.push_str("library IEEE;\n");
        code.push_str("use IEEE.STD_LOGIC_1164.ALL;\n");
        code.push_str("use IEEE.NUMERIC_STD.ALL;\n\n");

        let (inputs, outputs) = Self::detect_circuit_io(circuit);

        code.push_str(&format!("entity {} is\n", clean_ent_name));
        code.push_str("    Port (\n");
        let mut port_decls = Vec::new();
        for (id, name) in &inputs {
            let safe_name = format!("{}_{}", sanitize_identifier(name), id_suffix(*id));
            port_decls.push(format!("        {} : in  STD_LOGIC", safe_name));
        }
        for (id, name) in &outputs {
            let safe_name = format!("{}_{}", sanitize_identifier(name), id_suffix(*id));
            port_decls.push(format!("        {} : out STD_LOGIC", safe_name));
        }
        if port_decls.is_empty() {
            port_decls.push("        dummy_port : in STD_LOGIC".to_string());
        }
        code.push_str(&port_decls.join(";\n"));
        code.push_str("\n    );\n");
        code.push_str(&format!("end {};\n\n", clean_ent_name));

        code.push_str(&format!(
            "architecture Structural of {} is\n",
            clean_ent_name
        ));

        let mut wire_map: HashMap<PortEndpoint, String> = HashMap::new();
        let mut internal_wires: HashSet<String> = HashSet::new();

        for (id, name) in &inputs {
            let safe_name = format!("{}_{}", sanitize_identifier(name), id_suffix(*id));
            wire_map.insert(
                PortEndpoint {
                    component_id: *id,
                    is_output: true,
                    port_index: 0,
                },
                safe_name,
            );
        }
        for (id, name) in &outputs {
            let safe_name = format!("{}_{}", sanitize_identifier(name), id_suffix(*id));
            wire_map.insert(
                PortEndpoint {
                    component_id: *id,
                    is_output: false,
                    port_index: 0,
                },
                safe_name,
            );
        }

        for (nid, net) in &circuit.nets {
            let net_name = if let Some(ref lbl) = net.label {
                sanitize_identifier(lbl)
            } else {
                format!("sig_{:?}", nid)
            };

            let src_name = wire_map.get(&net.source).cloned().unwrap_or_else(|| {
                internal_wires.insert(net_name.clone());
                net_name.clone()
            });

            wire_map.insert(net.source, src_name.clone());

            for sink in &net.sinks {
                if let Some(out_port_name) = wire_map.get(sink).cloned() {
                    wire_map.insert(*sink, out_port_name);
                } else {
                    wire_map.insert(*sink, src_name.clone());
                }
            }
        }

        if !internal_wires.is_empty() {
            let mut sorted_wires: Vec<_> = internal_wires.into_iter().collect();
            sorted_wires.sort();
            for w in sorted_wires {
                code.push_str(&format!("    signal {} : STD_LOGIC := '0';\n", w));
            }
        }

        code.push_str("begin\n");

        for (cid, comp) in &circuit.components {
            let sid = id_suffix(cid);
            let in_sig = |p: usize| {
                wire_map
                    .get(&PortEndpoint {
                        component_id: cid,
                        is_output: false,
                        port_index: p,
                    })
                    .cloned()
                    .unwrap_or_else(|| "'0'".to_string())
            };
            let out_sig = |p: usize| {
                wire_map
                    .get(&PortEndpoint {
                        component_id: cid,
                        is_output: true,
                        port_index: p,
                    })
                    .cloned()
                    .unwrap_or_else(|| format!("open_{}_{}", sid, p))
            };

            match &comp.kind {
                GateKind::And => code.push_str(&format!(
                    "    {} <= {} and {};\n",
                    out_sig(0),
                    in_sig(0),
                    in_sig(1)
                )),
                GateKind::Or => code.push_str(&format!(
                    "    {} <= {} or {};\n",
                    out_sig(0),
                    in_sig(0),
                    in_sig(1)
                )),
                GateKind::Not => {
                    code.push_str(&format!("    {} <= not {};\n", out_sig(0), in_sig(0)))
                }
                GateKind::Nand => code.push_str(&format!(
                    "    {} <= not ({} and {});\n",
                    out_sig(0),
                    in_sig(0),
                    in_sig(1)
                )),
                GateKind::Nor => code.push_str(&format!(
                    "    {} <= not ({} or {});\n",
                    out_sig(0),
                    in_sig(0),
                    in_sig(1)
                )),
                GateKind::Xor => code.push_str(&format!(
                    "    {} <= {} xor {};\n",
                    out_sig(0),
                    in_sig(0),
                    in_sig(1)
                )),
                GateKind::Xnor => code.push_str(&format!(
                    "    {} <= not ({} xor {});\n",
                    out_sig(0),
                    in_sig(0),
                    in_sig(1)
                )),
                GateKind::Mux2 => code.push_str(&format!(
                    "    {} <= {} when {} = '1' else {};\n",
                    out_sig(0),
                    in_sig(1),
                    in_sig(2),
                    in_sig(0)
                )),
                _ => {}
            }
        }

        code.push_str("end Structural;\n");
        code
    }

    fn detect_circuit_io(circuit: &Circuit) -> CircuitIoPorts {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        for (id, comp) in &circuit.components {
            match comp.kind {
                GateKind::ToggleSwitch => inputs.push((id, "sw".to_string())),
                GateKind::Clock => inputs.push((id, "clk".to_string())),
                GateKind::Led => outputs.push((id, "led".to_string())),
                _ => {}
            }
        }

        inputs.sort_by_key(|(id, _)| format!("{:?}", id));
        outputs.sort_by_key(|(id, _)| format!("{:?}", id));

        (inputs, outputs)
    }
}

fn sanitize_identifier(s: &str) -> String {
    let clean: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    if clean.is_empty()
        || clean
            .chars()
            .next()
            .map(|c| c.is_numeric())
            .unwrap_or(false)
    {
        format!("net_{}", clean)
    } else {
        clean
    }
}

fn id_suffix(id: ComponentId) -> String {
    let raw = format!("{:?}", id);
    raw.chars().filter(|c| c.is_alphanumeric()).collect()
}

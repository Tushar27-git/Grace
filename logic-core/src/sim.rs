use crate::component::{ClockEdge, GateKind};
use crate::graph::{Circuit, ComponentId, ComponentNode, PortEndpoint, SubcircuitDef};
use crate::signal::Signal;
use std::collections::{HashMap, VecDeque};

pub struct Simulator;

impl Simulator {
    /// Full settle: propagates all signals through the circuit graph until stable.
    pub fn settle(circuit: &mut Circuit) {
        let mut queue: VecDeque<ComponentId> = VecDeque::new();
        let subcircuits = circuit.subcircuits.clone();

        // Step 1: Initial evaluation of components
        for (comp_id, comp) in &mut circuit.components {
            let new_outputs = Self::eval_node_with_subcircuits(comp, &subcircuits);
            if new_outputs != comp.output_signals {
                comp.output_signals = new_outputs;
            }
            queue.push_back(comp_id);
        }

        // Synchronize all nets with their source component outputs
        for (_, net) in &mut circuit.nets {
            if let Some(src_comp) = circuit.components.get(net.source.component_id)
                && let Some(&src_sig) = src_comp.output_signals.get(net.source.port_index)
            {
                net.current_signal = src_sig;
                for sink in &net.sinks {
                    if let Some(sink_comp) = circuit.components.get_mut(sink.component_id)
                        && sink.port_index < sink_comp.input_signals.len()
                        && sink_comp.input_signals[sink.port_index] != src_sig
                    {
                        sink_comp.input_signals[sink.port_index] = src_sig;
                        if !queue.contains(&sink.component_id) {
                            queue.push_back(sink.component_id);
                        }
                    }
                }
            }
        }

        let max_iterations = circuit.components.len() * 40 + 150;
        let mut iteration_count = 0;

        while let Some(comp_id) = queue.pop_front() {
            iteration_count += 1;
            if iteration_count > max_iterations {
                for (_, comp) in &mut circuit.components {
                    for sig in &mut comp.output_signals {
                        *sig = Signal::X;
                    }
                }
                for (_, net) in &mut circuit.nets {
                    net.current_signal = Signal::X;
                }
                break;
            }

            let mut changed_outputs = Vec::new();
            if let Some(comp) = circuit.components.get_mut(comp_id) {
                let new_outputs = Self::eval_node_with_subcircuits(comp, &subcircuits);
                for (out_idx, &new_sig) in new_outputs.iter().enumerate() {
                    let old_sig = comp.output_signals.get(out_idx).copied();
                    if old_sig != Some(new_sig) {
                        if out_idx < comp.output_signals.len() {
                            comp.output_signals[out_idx] = new_sig;
                        } else {
                            comp.output_signals.push(new_sig);
                        }
                        changed_outputs.push((out_idx, new_sig));
                    }
                }
            }

            // Propagate changed outputs to connected nets and sink components
            for (out_idx, new_sig) in changed_outputs {
                let source_endpoint = PortEndpoint {
                    component_id: comp_id,
                    is_output: true,
                    port_index: out_idx,
                };

                let net_info = circuit
                    .nets
                    .iter_mut()
                    .find(|(_, net)| net.source == source_endpoint)
                    .map(|(nid, net)| {
                        net.current_signal = new_sig;
                        (nid, net.sinks.clone())
                    });

                if let Some((_, sinks)) = net_info {
                    for sink in sinks {
                        if let Some(sink_comp) = circuit.components.get_mut(sink.component_id)
                            && sink.port_index < sink_comp.input_signals.len()
                            && sink_comp.input_signals[sink.port_index] != new_sig
                        {
                            sink_comp.input_signals[sink.port_index] = new_sig;
                            if !queue.contains(&sink.component_id) {
                                queue.push_back(sink.component_id);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Advances clocks and propagates sequential state
    pub fn tick(circuit: &mut Circuit, edge: ClockEdge) {
        if edge == ClockEdge::Rising {
            for (_, comp) in &mut circuit.components {
                if comp.kind == GateKind::Clock {
                    comp.state_flag = !comp.state_flag;
                }
            }
        }
        Self::settle(circuit);
    }

    /// Evaluates any component node according to its kind and inputs (using empty subcircuits)
    pub fn eval_node(comp: &mut ComponentNode) -> Vec<Signal> {
        let empty = HashMap::new();
        Self::eval_node_with_subcircuits(comp, &empty)
    }

    /// Evaluates any component node with access to registered subcircuit definitions
    pub fn eval_node_with_subcircuits(
        comp: &mut ComponentNode,
        subcircuits: &HashMap<String, SubcircuitDef>,
    ) -> Vec<Signal> {
        let inputs = &comp.input_signals;

        match &comp.kind {
            // 1. Primitive Logic Gates
            GateKind::And => {
                let mut res = Signal::One;
                for &inp in inputs {
                    res = res & inp;
                }
                vec![res]
            }
            GateKind::Or => {
                let mut res = Signal::Zero;
                for &inp in inputs {
                    res = res | inp;
                }
                vec![res]
            }
            GateKind::Not => {
                let a = inputs.first().copied().unwrap_or(Signal::Zero);
                vec![!a]
            }
            GateKind::Nand => {
                let mut res = Signal::One;
                for &inp in inputs {
                    res = res & inp;
                }
                vec![!res]
            }
            GateKind::Nor => {
                let mut res = Signal::Zero;
                for &inp in inputs {
                    res = res | inp;
                }
                vec![!res]
            }
            GateKind::Xor => {
                let mut count = 0;
                for &inp in inputs {
                    if inp.is_high() {
                        count += 1;
                    }
                }
                vec![if count % 2 == 1 {
                    Signal::One
                } else {
                    Signal::Zero
                }]
            }
            GateKind::Xnor => {
                let mut count = 0;
                for &inp in inputs {
                    if inp.is_high() {
                        count += 1;
                    }
                }
                vec![if count % 2 == 0 {
                    Signal::One
                } else {
                    Signal::Zero
                }]
            }

            // 2. Fundamental I/O & Displays
            GateKind::ToggleSwitch | GateKind::Clock => {
                vec![if comp.state_flag {
                    Signal::One
                } else {
                    Signal::Zero
                }]
            }
            GateKind::Led
            | GateKind::BinaryDisplay4
            | GateKind::HexDisplay
            | GateKind::SevenSegment => {
                vec![]
            }

            // 3. Arithmetic: Half Adder (A, B -> Sum, Cout)
            GateKind::HalfAdder => {
                let a = inputs.first().copied().unwrap_or(Signal::Zero);
                let b = inputs.get(1).copied().unwrap_or(Signal::Zero);
                vec![a ^ b, a & b]
            }

            // Full Adder (A, B, Cin -> Sum, Cout)
            GateKind::FullAdder => {
                let a = inputs.first().copied().unwrap_or(Signal::Zero);
                let b = inputs.get(1).copied().unwrap_or(Signal::Zero);
                let cin = inputs.get(2).copied().unwrap_or(Signal::Zero);
                let sum = a ^ b ^ cin;
                let cout = (a & b) | (cin & (a ^ b));
                vec![sum, cout]
            }

            // Half Subtractor (A, B -> Diff, Bout)
            GateKind::HalfSubtractor => {
                let a = inputs.first().copied().unwrap_or(Signal::Zero);
                let b = inputs.get(1).copied().unwrap_or(Signal::Zero);
                vec![a ^ b, (!a) & b]
            }

            // Full Subtractor (A, B, Bin -> Diff, Bout)
            GateKind::FullSubtractor => {
                let a = inputs.first().copied().unwrap_or(Signal::Zero);
                let b = inputs.get(1).copied().unwrap_or(Signal::Zero);
                let bin = inputs.get(2).copied().unwrap_or(Signal::Zero);
                let diff = a ^ b ^ bin;
                let bout = ((!a) & b) | (bin & !(a ^ b));
                vec![diff, bout]
            }

            // 4-Bit Ripple Carry Adder: A[0..3], B[0..3], Cin -> Sum[0..3], Cout
            GateKind::RippleCarryAdder4 => {
                let mut c = inputs.get(8).copied().unwrap_or(Signal::Zero);
                let mut sums = Vec::with_capacity(5);
                for i in 0..4 {
                    let a = inputs.get(i).copied().unwrap_or(Signal::Zero);
                    let b = inputs.get(4 + i).copied().unwrap_or(Signal::Zero);
                    let s = a ^ b ^ c;
                    c = (a & b) | (c & (a ^ b));
                    sums.push(s);
                }
                sums.push(c);
                sums
            }

            // 4-Bit Carry-Lookahead Adder: A[0..3], B[0..3], Cin -> Sum[0..3], Cout
            GateKind::CarryLookaheadAdder4 => {
                let cin = inputs.get(8).copied().unwrap_or(Signal::Zero);
                let mut p = [Signal::Zero; 4];
                let mut g = [Signal::Zero; 4];
                for i in 0..4 {
                    let a = inputs.get(i).copied().unwrap_or(Signal::Zero);
                    let b = inputs.get(4 + i).copied().unwrap_or(Signal::Zero);
                    p[i] = a ^ b;
                    g[i] = a & b;
                }
                let c0 = cin;
                let c1 = g[0] | (p[0] & c0);
                let c2 = g[1] | (p[1] & g[0]) | (p[1] & p[0] & c0);
                let c3 = g[2] | (p[2] & g[1]) | (p[2] & p[1] & g[0]) | (p[2] & p[1] & p[0] & c0);
                let c4 = g[3]
                    | (p[3] & g[2])
                    | (p[3] & p[2] & g[1])
                    | (p[3] & p[2] & p[1] & g[0])
                    | (p[3] & p[2] & p[1] & p[0] & c0);

                let s0 = p[0] ^ c0;
                let s1 = p[1] ^ c1;
                let s2 = p[2] ^ c2;
                let s3 = p[3] ^ c3;

                vec![s0, s1, s2, s3, c4]
            }

            // 4. Sequential: SR Latch (S, R -> Q, !Q)
            GateKind::SrLatch => {
                let s = inputs.first().map(|x| x.is_high()).unwrap_or(false);
                let r = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                if s && !r {
                    comp.state_flag = true;
                } else if !s && r {
                    comp.state_flag = false;
                }
                let q = if comp.state_flag {
                    Signal::One
                } else {
                    Signal::Zero
                };
                vec![q, !q]
            }

            // D Latch (D, EN -> Q, !Q)
            GateKind::DLatch => {
                let d = inputs.first().map(|x| x.is_high()).unwrap_or(false);
                let en = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                if en {
                    comp.state_flag = d;
                }
                let q = if comp.state_flag {
                    Signal::One
                } else {
                    Signal::Zero
                };
                vec![q, !q]
            }

            // D Flip-Flop (D, CLK -> Q, !Q) - Edge Triggered
            GateKind::DFlipFlop => {
                let d = inputs.first().map(|x| x.is_high()).unwrap_or(false);
                let clk = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                if clk && !comp.clock_prev {
                    comp.state_flag = d;
                }
                comp.clock_prev = clk;
                let q = if comp.state_flag {
                    Signal::One
                } else {
                    Signal::Zero
                };
                vec![q, !q]
            }

            // JK Flip-Flop (J, K, CLK -> Q, !Q)
            GateKind::JkFlipFlop => {
                let j = inputs.first().map(|x| x.is_high()).unwrap_or(false);
                let k = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                let clk = inputs.get(2).map(|x| x.is_high()).unwrap_or(false);
                if clk && !comp.clock_prev {
                    match (j, k) {
                        (true, false) => comp.state_flag = true,
                        (false, true) => comp.state_flag = false,
                        (true, true) => comp.state_flag = !comp.state_flag,
                        (false, false) => {}
                    }
                }
                comp.clock_prev = clk;
                let q = if comp.state_flag {
                    Signal::One
                } else {
                    Signal::Zero
                };
                vec![q, !q]
            }

            // T Flip-Flop (T, CLK -> Q, !Q)
            GateKind::TFlipFlop => {
                let t = inputs.first().map(|x| x.is_high()).unwrap_or(false);
                let clk = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                if clk && !comp.clock_prev && t {
                    comp.state_flag = !comp.state_flag;
                }
                comp.clock_prev = clk;
                let q = if comp.state_flag {
                    Signal::One
                } else {
                    Signal::Zero
                };
                vec![q, !q]
            }

            // Clock Divider (CLK, RST -> /2, /4)
            GateKind::ClockDivider => {
                let clk = inputs.first().map(|x| x.is_high()).unwrap_or(false);
                let rst = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                if comp.memory.len() < 2 {
                    comp.memory = vec![0, 0];
                }
                if rst {
                    comp.memory[0] = 0;
                    comp.memory[1] = 0;
                } else if clk && !comp.clock_prev {
                    let prev_div2 = comp.memory[0];
                    comp.memory[0] = 1 - comp.memory[0];
                    if prev_div2 == 1 && comp.memory[0] == 0 {
                        comp.memory[1] = 1 - comp.memory[1];
                    }
                }
                comp.clock_prev = clk;
                vec![
                    if comp.memory[0] == 1 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if comp.memory[1] == 1 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                ]
            }

            // 4-Bit Register (D0..D3, CLK -> Q0..Q3)
            GateKind::Register4 => {
                let clk = inputs.get(4).map(|x| x.is_high()).unwrap_or(false);
                if comp.memory.len() < 4 {
                    comp.memory = vec![0; 4];
                }
                if clk && !comp.clock_prev {
                    for i in 0..4 {
                        comp.memory[i] = if inputs.get(i).map(|s| s.is_high()).unwrap_or(false) {
                            1
                        } else {
                            0
                        };
                    }
                }
                comp.clock_prev = clk;
                comp.memory
                    .iter()
                    .map(|&b| if b == 1 { Signal::One } else { Signal::Zero })
                    .collect()
            }

            // 4-Bit Shift Register (D, CLK, EN, RST -> Q0..Q3)
            GateKind::ShiftRegister4 => {
                let d = inputs.first().map(|x| x.is_high()).unwrap_or(false);
                let clk = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                if comp.memory.len() < 4 {
                    comp.memory = vec![0; 4];
                }
                if clk && !comp.clock_prev {
                    comp.memory[3] = comp.memory[2];
                    comp.memory[2] = comp.memory[1];
                    comp.memory[1] = comp.memory[0];
                    comp.memory[0] = if d { 1 } else { 0 };
                }
                comp.clock_prev = clk;
                comp.memory
                    .iter()
                    .map(|&b| if b == 1 { Signal::One } else { Signal::Zero })
                    .collect()
            }

            // 4-Bit Counter (CLK, RST -> Q0..Q3)
            GateKind::Counter4 => {
                let clk = inputs.first().map(|x| x.is_high()).unwrap_or(false);
                let rst = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                if comp.memory.is_empty() {
                    comp.memory = vec![0];
                }
                if rst {
                    comp.memory[0] = 0;
                } else if clk && !comp.clock_prev {
                    comp.memory[0] = (comp.memory[0] + 1) % 16;
                }
                comp.clock_prev = clk;
                let val = comp.memory[0];
                vec![
                    if (val & 1) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if (val & 2) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if (val & 4) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if (val & 8) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                ]
            }

            // 5. Routing: 2:1 Multiplexer (I0, I1, Sel -> Y)
            GateKind::Mux2 => {
                let i0 = inputs.first().copied().unwrap_or(Signal::Zero);
                let i1 = inputs.get(1).copied().unwrap_or(Signal::Zero);
                let sel = inputs.get(2).map(|x| x.is_high()).unwrap_or(false);
                vec![if sel { i1 } else { i0 }]
            }

            // 4:1 Multiplexer (I0..I3, S0, S1 -> Y)
            GateKind::Mux4 => {
                let s0 = inputs.get(4).map(|x| x.is_high()).unwrap_or(false);
                let s1 = inputs.get(5).map(|x| x.is_high()).unwrap_or(false);
                let idx = (if s0 { 1 } else { 0 }) + (if s1 { 2 } else { 0 });
                let selected = inputs.get(idx).copied().unwrap_or(Signal::Zero);
                vec![selected]
            }

            // 1:2 Demultiplexer (IN, Sel -> Y0, Y1)
            GateKind::Demux2 => {
                let val = inputs.first().copied().unwrap_or(Signal::Zero);
                let sel = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                if sel {
                    vec![Signal::Zero, val]
                } else {
                    vec![val, Signal::Zero]
                }
            }

            // 1:4 Demultiplexer (IN, EN, S0, S1 -> Y0..Y3)
            GateKind::Demux4 => {
                let val = inputs.first().copied().unwrap_or(Signal::Zero);
                let en = inputs.get(1).map(|x| x.is_high()).unwrap_or(true);
                let s0 = inputs.get(2).map(|x| x.is_high()).unwrap_or(false);
                let s1 = inputs.get(3).map(|x| x.is_high()).unwrap_or(false);
                let idx = (if s0 { 1 } else { 0 }) + (if s1 { 2 } else { 0 });
                let mut outs = vec![Signal::Zero; 4];
                if en {
                    outs[idx] = val;
                }
                outs
            }

            // 4:2 Priority Encoder (D0..D3 -> A0, A1)
            GateKind::Encoder4to2 => {
                let d3 = inputs.get(3).map(|x| x.is_high()).unwrap_or(false);
                let d2 = inputs.get(2).map(|x| x.is_high()).unwrap_or(false);
                let d1 = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                let (a1, a0) = if d3 {
                    (Signal::One, Signal::One)
                } else if d2 {
                    (Signal::One, Signal::Zero)
                } else if d1 {
                    (Signal::Zero, Signal::One)
                } else {
                    (Signal::Zero, Signal::Zero)
                };
                vec![a0, a1]
            }

            // 2:4 Decoder (A0, A1 -> Y0..Y3)
            GateKind::Decoder2to4 => {
                let a0 = inputs.first().map(|x| x.is_high()).unwrap_or(false);
                let a1 = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                let idx = (if a0 { 1 } else { 0 }) + (if a1 { 2 } else { 0 });
                let mut outs = vec![Signal::Zero; 4];
                outs[idx] = Signal::One;
                outs
            }

            // 2-Bit Comparator (A0, A1, B0, B1 -> A>B, A=B, A<B)
            GateKind::Comparator2 => {
                let a0 = inputs.first().map(|x| x.is_high()).unwrap_or(false);
                let a1 = inputs.get(1).map(|x| x.is_high()).unwrap_or(false);
                let b0 = inputs.get(2).map(|x| x.is_high()).unwrap_or(false);
                let b1 = inputs.get(3).map(|x| x.is_high()).unwrap_or(false);

                let val_a = (if a0 { 1 } else { 0 }) + (if a1 { 2 } else { 0 });
                let val_b = (if b0 { 1 } else { 0 }) + (if b1 { 2 } else { 0 });

                vec![
                    if val_a > val_b {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if val_a == val_b {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if val_a < val_b {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                ]
            }

            // 6. Computer Blocks: 4-Bit ALU (A0..A3, B0..B3, Op0..Op2 -> R0..R3, Zero, Carry)
            GateKind::Alu4 => {
                let mut a = 0u8;
                let mut b = 0u8;
                for i in 0..4 {
                    if inputs.get(i).map(|x| x.is_high()).unwrap_or(false) {
                        a |= 1 << i;
                    }
                    if inputs.get(4 + i).map(|x| x.is_high()).unwrap_or(false) {
                        b |= 1 << i;
                    }
                }
                let mut op = 0u8;
                for i in 0..3 {
                    if inputs.get(8 + i).map(|x| x.is_high()).unwrap_or(false) {
                        op |= 1 << i;
                    }
                }

                let (res, carry) = match op {
                    0 => {
                        // ADD
                        let r = (a as u16) + (b as u16);
                        ((r & 0xF) as u8, r > 15)
                    }
                    1 => {
                        // SUB
                        let r = (a as i16) - (b as i16);
                        ((r & 0xF) as u8, r < 0)
                    }
                    2 => (a & b, false),                 // AND
                    3 => (a | b, false),                 // OR
                    4 => (a ^ b, false),                 // XOR
                    5 => ((!a) & 0xF, false),            // NOT
                    6 => ((a << 1) & 0xF, (a & 8) != 0), // SHL
                    _ => (a >> 1, (a & 1) != 0),         // SHR
                };

                let zero = res == 0;
                let mut outs = Vec::with_capacity(6);
                for i in 0..4 {
                    outs.push(if (res & (1 << i)) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    });
                }
                outs.push(if zero { Signal::One } else { Signal::Zero });
                outs.push(if carry { Signal::One } else { Signal::Zero });
                outs
            }

            // 16x4 ROM (A0..A3 -> D0..D3)
            GateKind::Rom16x4 => {
                let mut addr = 0usize;
                for i in 0..4 {
                    if inputs.get(i).map(|x| x.is_high()).unwrap_or(false) {
                        addr |= 1 << i;
                    }
                }
                let val = comp.memory.get(addr % 16).copied().unwrap_or(0);
                vec![
                    if (val & 1) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if (val & 2) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if (val & 4) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if (val & 8) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                ]
            }

            // 16x4 RAM (A0..A3, D_in0..D_in3, WE, CLK -> D_out0..D_out3)
            GateKind::Ram16x4 => {
                let mut addr = 0usize;
                for i in 0..4 {
                    if inputs.get(i).map(|x| x.is_high()).unwrap_or(false) {
                        addr |= 1 << i;
                    }
                }
                let mut data_in = 0u8;
                for i in 0..4 {
                    if inputs.get(4 + i).map(|x| x.is_high()).unwrap_or(false) {
                        data_in |= 1 << i;
                    }
                }
                let we = inputs.get(8).map(|x| x.is_high()).unwrap_or(false);
                let clk = inputs.get(9).map(|x| x.is_high()).unwrap_or(false);

                if comp.memory.len() < 16 {
                    comp.memory = vec![0; 16];
                }

                if clk && !comp.clock_prev && we {
                    comp.memory[addr % 16] = data_in;
                }
                comp.clock_prev = clk;

                let val = comp.memory[addr % 16];
                vec![
                    if (val & 1) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if (val & 2) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if (val & 4) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                    if (val & 8) != 0 {
                        Signal::One
                    } else {
                        Signal::Zero
                    },
                ]
            }

            // Custom Subcircuit instance evaluation
            GateKind::SubcircuitInstance(name) => {
                if let Some(sub_def) = subcircuits.get(name) {
                    Self::evaluate_subcircuit(sub_def, inputs)
                } else {
                    vec![inputs.first().copied().unwrap_or(Signal::Zero)]
                }
            }
        }
    }

    /// Hierarchical simulation for subcircuit instances
    pub fn evaluate_subcircuit(sub_def: &SubcircuitDef, inputs: &[Signal]) -> Vec<Signal> {
        let mut sub = sub_def.circuit.clone();

        // 1. Gather all input components (ToggleSwitch or Clock) sorted by Y position
        let mut input_ids: Vec<ComponentId> = sub
            .components
            .iter()
            .filter(|(_, c)| matches!(c.kind, GateKind::ToggleSwitch | GateKind::Clock))
            .map(|(id, _)| id)
            .collect();
        input_ids.sort_by(|&a, &b| {
            let ya = sub.components.get(a).map(|c| c.pos.1).unwrap_or(0.0);
            let yb = sub.components.get(b).map(|c| c.pos.1).unwrap_or(0.0);
            ya.partial_cmp(&yb).unwrap_or(std::cmp::Ordering::Equal)
        });

        for (idx, &in_id) in input_ids.iter().enumerate() {
            if let Some(sig) = inputs.get(idx)
                && let Some(c) = sub.components.get_mut(in_id)
            {
                c.state_flag = sig.is_high();
                c.output_signals = vec![*sig];
            }
        }

        // 2. Settle the internal circuit
        Self::settle(&mut sub);

        // 3. Gather all output components (Led) sorted by Y position
        let mut output_ids: Vec<ComponentId> = sub
            .components
            .iter()
            .filter(|(_, c)| c.kind == GateKind::Led)
            .map(|(id, _)| id)
            .collect();
        output_ids.sort_by(|&a, &b| {
            let ya = sub.components.get(a).map(|c| c.pos.1).unwrap_or(0.0);
            let yb = sub.components.get(b).map(|c| c.pos.1).unwrap_or(0.0);
            ya.partial_cmp(&yb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut res = Vec::new();
        for &out_id in &output_ids {
            if let Some(c) = sub.components.get(out_id) {
                res.push(c.input_signals.first().copied().unwrap_or(Signal::Zero));
            }
        }

        if res.is_empty() {
            res.push(Signal::Zero);
        }
        res
    }
}

use logic_core::{
    verify::TruthTableGenerator, Circuit, ClockEdge, GateKind, HdlExporter, PortEndpoint, Signal,
    Simulator,
};
use std::time::Instant;

fn main() {
    println!("================================================================================");
    println!("                       LOGIC LAB SIMULATOR BENCHMARK SUITE                      ");
    println!("                       Engine: logic-core v1.0.0 (Rust 2024 / release)          ");
    println!("================================================================================\n");

    bench_raw_gate_evaluations();
    bench_inverter_chains();
    bench_arithmetic_blocks();
    bench_sequential_clock_ticks();
    bench_truth_table_generation();
    bench_hdl_generation();

    println!("================================================================================");
    println!("                           BENCHMARK SUITE COMPLETED                            ");
    println!("================================================================================");
}

fn bench_raw_gate_evaluations() {
    println!("[1] RAW GATE EVALUATION THROUGHPUT (1,000,000 iterations per gate)");
    println!("{:<24} {:<18} {:<18} {:<16}", "Gate Type", "Elapsed (ms)", "Throughput (ops/s)", "Latency/eval");
    println!("{:-<76}", "");

    let gates = [
        (GateKind::And, vec![Signal::One, Signal::One]),
        (GateKind::Or, vec![Signal::Zero, Signal::One]),
        (GateKind::Not, vec![Signal::One]),
        (GateKind::Nand, vec![Signal::One, Signal::One]),
        (GateKind::Nor, vec![Signal::Zero, Signal::Zero]),
        (GateKind::Xor, vec![Signal::One, Signal::Zero]),
        (GateKind::Xnor, vec![Signal::One, Signal::One]),
        (GateKind::HalfAdder, vec![Signal::One, Signal::One]),
        (GateKind::FullAdder, vec![Signal::One, Signal::One, Signal::One]),
        (GateKind::Alu4, vec![
            Signal::One, Signal::Zero, Signal::One, Signal::Zero, // A = 5
            Signal::Zero, Signal::Zero, Signal::One, Signal::One, // B = 3
            Signal::Zero, Signal::Zero, Signal::Zero,             // Op = ADD
        ]),
    ];

    let iters = 1_000_000usize;

    for (gate, inputs) in gates {
        let start = Instant::now();
        let mut black_box = 0usize;
        for _ in 0..iters {
            let out = gate.eval(&inputs, false);
            black_box = black_box.wrapping_add(out.len());
        }
        let elapsed = start.elapsed();
        let millis = elapsed.as_secs_f64() * 1000.0;
        let ops_sec = (iters as f64) / elapsed.as_secs_f64();
        let ns_op = (elapsed.as_nanos() as f64) / (iters as f64);

        println!(
            "{:<24} {:<18.2} {:<18.2e} {:<16.2} ns",
            format!("{:?}", gate),
            millis,
            ops_sec,
            ns_op
        );
        std::hint::black_box(black_box);
    }
    println!();
}

fn bench_inverter_chains() {
    println!("[2] INVERTER CASCADE CONVERGENCE (Event-driven settle propagation)");
    println!("{:<18} {:<16} {:<18} {:<18}", "Chain Stages", "Build Time (ms)", "Settle Time (ms)", "Passes / Settle");
    println!("{:-<72}", "");

    let stages_list = [10, 50, 100, 250, 500, 1000];

    for &stages in &stages_list {
        let build_start = Instant::now();
        let mut circuit = Circuit::new();

        let sw = circuit.add_component(GateKind::ToggleSwitch, (0.0, 0.0));
        let mut last_id = sw;
        let mut last_port = 0usize;

        for i in 0..stages {
            let inv = circuit.add_component(GateKind::Not, ((i + 1) as f32 * 50.0, 0.0));
            circuit.connect_ports(
                PortEndpoint { component_id: last_id, is_output: true, port_index: last_port },
                PortEndpoint { component_id: inv, is_output: false, port_index: 0 },
            );
            last_id = inv;
            last_port = 0;
        }

        let led = circuit.add_component(GateKind::Led, ((stages + 1) as f32 * 50.0, 0.0));
        circuit.connect_ports(
            PortEndpoint { component_id: last_id, is_output: true, port_index: 0 },
            PortEndpoint { component_id: led, is_output: false, port_index: 0 },
        );
        let build_elapsed = build_start.elapsed().as_secs_f64() * 1000.0;

        // Warmup settle
        Simulator::settle(&mut circuit);

        // Toggle switch and measure settle convergence
        if let Some(comp) = circuit.components.get_mut(sw) {
            comp.state_flag = true;
        }

        let settle_start = Instant::now();
        Simulator::settle(&mut circuit);
        let settle_elapsed = settle_start.elapsed().as_secs_f64() * 1000.0;

        println!(
            "{:<18} {:<16.3} {:<18.3} {:<18}",
            format!("{} inverters", stages),
            build_elapsed,
            settle_elapsed,
            "Converged"
        );
    }
    println!();
}

fn bench_arithmetic_blocks() {
    println!("[3] ARITHMETIC CONVERGENCE & SETTLE SPEED (10,000 iterations)");
    println!("{:<24} {:<18} {:<18} {:<16}", "Circuit Block", "Total Time (ms)", "Settle ops/s", "Avg Latency");
    println!("{:-<76}", "");

    let blocks = [
        ("RippleCarryAdder-4", GateKind::RippleCarryAdder4),
        ("CarryLookaheadAdder-4", GateKind::CarryLookaheadAdder4),
        ("ALU-4 (Arithmetic)", GateKind::Alu4),
        ("FullSubtractor", GateKind::FullSubtractor),
    ];

    let iters = 10_000usize;

    for (name, kind) in blocks {
        let mut circuit = Circuit::new();
        let comp_id = circuit.add_component(kind.clone(), (100.0, 100.0));
        let inputs_count = kind.input_count();

        let mut switches = Vec::new();
        for i in 0..inputs_count {
            let sw = circuit.add_component(GateKind::ToggleSwitch, (0.0, i as f32 * 30.0));
            circuit.connect_ports(
                PortEndpoint { component_id: sw, is_output: true, port_index: 0 },
                PortEndpoint { component_id: comp_id, is_output: false, port_index: i },
            );
            switches.push(sw);
        }

        let start = Instant::now();
        for i in 0..iters {
            let active_idx = i % switches.len();
            if let Some(sw) = circuit.components.get_mut(switches[active_idx]) {
                sw.state_flag = (i / switches.len()) % 2 == 1;
            }
            Simulator::settle(&mut circuit);
        }
        let elapsed = start.elapsed();
        let millis = elapsed.as_secs_f64() * 1000.0;
        let settles_per_sec = (iters as f64) / elapsed.as_secs_f64();
        let avg_latency_us = (elapsed.as_micros() as f64) / (iters as f64);

        println!(
            "{:<24} {:<18.2} {:<18.2e} {:<16.2} µs",
            name,
            millis,
            settles_per_sec,
            avg_latency_us
        );
    }
    println!();
}

fn bench_sequential_clock_ticks() {
    println!("[4] SEQUENTIAL CLOCK TICKING THROUGHPUT (100,000 clock edge cycles)");
    println!("{:<24} {:<18} {:<18} {:<16}", "Sequential Block", "Total Time (ms)", "Clock Freq (Hz)", "Tick Latency");
    println!("{:-<76}", "");

    let blocks = [
        ("D Flip-Flop", GateKind::DFlipFlop),
        ("JK Flip-Flop", GateKind::JkFlipFlop),
        ("T Flip-Flop", GateKind::TFlipFlop),
        ("Register-4", GateKind::Register4),
        ("ShiftRegister-4", GateKind::ShiftRegister4),
        ("Counter-4", GateKind::Counter4),
    ];

    let iters = 100_000usize;

    for (name, kind) in blocks {
        let mut circuit = Circuit::new();
        let clk = circuit.add_component(GateKind::Clock, (0.0, 0.0));
        let seq = circuit.add_component(kind.clone(), (100.0, 100.0));

        let clk_port = if kind == GateKind::Counter4 || kind == GateKind::Register4 { 4 } else { 1 };
        circuit.connect_ports(
            PortEndpoint { component_id: clk, is_output: true, port_index: 0 },
            PortEndpoint { component_id: seq, is_output: false, port_index: clk_port },
        );

        let start = Instant::now();
        for _ in 0..iters {
            Simulator::tick(&mut circuit, ClockEdge::Rising);
        }
        let elapsed = start.elapsed();
        let millis = elapsed.as_secs_f64() * 1000.0;
        let hz = (iters as f64) / elapsed.as_secs_f64();
        let latency_ns = (elapsed.as_nanos() as f64) / (iters as f64);

        println!(
            "{:<24} {:<18.2} {:<18.2e} {:<16.2} ns",
            name,
            millis,
            hz,
            latency_ns
        );
    }
    println!();
}

fn bench_truth_table_generation() {
    println!("[5] TRUTH TABLE VERIFICATION & EXHAUSTIVE SWEEP");
    println!("{:<24} {:<16} {:<18} {:<18}", "Circuit Type", "Input Bits", "Total States", "Generation Time");
    println!("{:-<76}", "");

    // 2-input XOR
    let mut c2 = Circuit::new();
    let sw1 = c2.add_component(GateKind::ToggleSwitch, (0.0, 0.0));
    let sw2 = c2.add_component(GateKind::ToggleSwitch, (0.0, 30.0));
    let xor = c2.add_component(GateKind::Xor, (100.0, 15.0));
    let led = c2.add_component(GateKind::Led, (200.0, 15.0));
    c2.connect_ports(PortEndpoint { component_id: sw1, is_output: true, port_index: 0 }, PortEndpoint { component_id: xor, is_output: false, port_index: 0 });
    c2.connect_ports(PortEndpoint { component_id: sw2, is_output: true, port_index: 0 }, PortEndpoint { component_id: xor, is_output: false, port_index: 1 });
    c2.connect_ports(PortEndpoint { component_id: xor, is_output: true, port_index: 0 }, PortEndpoint { component_id: led, is_output: false, port_index: 0 });

    let t0 = Instant::now();
    let _tt2 = TruthTableGenerator::generate(&c2, None);
    let d2 = t0.elapsed().as_secs_f64() * 1000.0;
    println!("{:<24} {:<16} {:<18} {:<18.3} ms", "2-Input XOR", 2, 4, d2);

    // 3-input Full Adder
    let mut c3 = Circuit::new();
    let sw_a = c3.add_component(GateKind::ToggleSwitch, (0.0, 0.0));
    let sw_b = c3.add_component(GateKind::ToggleSwitch, (0.0, 30.0));
    let sw_cin = c3.add_component(GateKind::ToggleSwitch, (0.0, 60.0));
    let fa = c3.add_component(GateKind::FullAdder, (100.0, 30.0));
    let led_s = c3.add_component(GateKind::Led, (200.0, 15.0));
    let led_c = c3.add_component(GateKind::Led, (200.0, 45.0));
    c3.connect_ports(PortEndpoint { component_id: sw_a, is_output: true, port_index: 0 }, PortEndpoint { component_id: fa, is_output: false, port_index: 0 });
    c3.connect_ports(PortEndpoint { component_id: sw_b, is_output: true, port_index: 0 }, PortEndpoint { component_id: fa, is_output: false, port_index: 1 });
    c3.connect_ports(PortEndpoint { component_id: sw_cin, is_output: true, port_index: 0 }, PortEndpoint { component_id: fa, is_output: false, port_index: 2 });
    c3.connect_ports(PortEndpoint { component_id: fa, is_output: true, port_index: 0 }, PortEndpoint { component_id: led_s, is_output: false, port_index: 0 });
    c3.connect_ports(PortEndpoint { component_id: fa, is_output: true, port_index: 1 }, PortEndpoint { component_id: led_c, is_output: false, port_index: 0 });

    let t1 = Instant::now();
    let _tt3 = TruthTableGenerator::generate(&c3, None);
    let d3 = t1.elapsed().as_secs_f64() * 1000.0;
    println!("{:<24} {:<16} {:<18} {:<18.3} ms", "Full Adder", 3, 8, d3);

    // 4-input MUX4
    let mut c4 = Circuit::new();
    let sw0 = c4.add_component(GateKind::ToggleSwitch, (0.0, 0.0));
    let sw1 = c4.add_component(GateKind::ToggleSwitch, (0.0, 20.0));
    let sw2 = c4.add_component(GateKind::ToggleSwitch, (0.0, 40.0));
    let sw3 = c4.add_component(GateKind::ToggleSwitch, (0.0, 60.0));
    let mux = c4.add_component(GateKind::Mux4, (100.0, 30.0));
    let led_m = c4.add_component(GateKind::Led, (200.0, 30.0));
    c4.connect_ports(PortEndpoint { component_id: sw0, is_output: true, port_index: 0 }, PortEndpoint { component_id: mux, is_output: false, port_index: 0 });
    c4.connect_ports(PortEndpoint { component_id: sw1, is_output: true, port_index: 0 }, PortEndpoint { component_id: mux, is_output: false, port_index: 1 });
    c4.connect_ports(PortEndpoint { component_id: sw2, is_output: true, port_index: 0 }, PortEndpoint { component_id: mux, is_output: false, port_index: 2 });
    c4.connect_ports(PortEndpoint { component_id: sw3, is_output: true, port_index: 0 }, PortEndpoint { component_id: mux, is_output: false, port_index: 3 });
    c4.connect_ports(PortEndpoint { component_id: mux, is_output: true, port_index: 0 }, PortEndpoint { component_id: led_m, is_output: false, port_index: 0 });

    let t4 = Instant::now();
    let _tt4 = TruthTableGenerator::generate(&c4, None);
    let d4 = t4.elapsed().as_secs_f64() * 1000.0;
    println!("{:<24} {:<16} {:<18} {:<18.3} ms", "4-Input Mux", 4, 16, d4);
    println!();
}

fn bench_hdl_generation() {
    println!("[6] HDL CODE SYNTHESIS & CODE GENERATION");
    println!("{:<24} {:<16} {:<18} {:<16}", "Target Language", "Component Count", "Generation Time", "Throughput");
    println!("{:-<76}", "");

    let mut c = Circuit::new();
    for i in 0..100 {
        let kind = match i % 5 {
            0 => GateKind::And,
            1 => GateKind::Or,
            2 => GateKind::Xor,
            3 => GateKind::Nand,
            _ => GateKind::Not,
        };
        c.add_component(kind, (i as f32 * 10.0, i as f32 * 10.0));
    }

    let t_v = Instant::now();
    for _ in 0..100 {
        let _ = HdlExporter::to_verilog(&c, "bench_circuit");
    }
    let d_v = t_v.elapsed().as_secs_f64() * 1000.0 / 100.0;
    println!("{:<24} {:<16} {:<18.3} ms {:<16.2} circuits/s", "Verilog 2001", 100, d_v, 1000.0 / d_v);

    let t_vhdl = Instant::now();
    for _ in 0..100 {
        let _ = HdlExporter::to_vhdl(&c, "bench_circuit");
    }
    let d_vhdl = t_vhdl.elapsed().as_secs_f64() * 1000.0 / 100.0;
    println!("{:<24} {:<16} {:<18.3} ms {:<16.2} circuits/s", "VHDL 93 / 2008", 100, d_vhdl, 1000.0 / d_vhdl);
    println!();
}

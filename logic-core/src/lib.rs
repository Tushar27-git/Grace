pub mod component;
pub mod graph;
pub mod hdl;
pub mod signal;
pub mod sim;
pub mod verify;

pub use component::*;
pub use graph::*;
pub use hdl::*;
pub use signal::*;
pub use sim::*;
pub use verify::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_evaluations() {
        assert_eq!(
            GateKind::And.eval(&[Signal::Zero, Signal::Zero], false),
            vec![Signal::Zero]
        );
        assert_eq!(
            GateKind::And.eval(&[Signal::One, Signal::One], false),
            vec![Signal::One]
        );
        assert_eq!(
            GateKind::Or.eval(&[Signal::One, Signal::Zero], false),
            vec![Signal::One]
        );
        assert_eq!(
            GateKind::Not.eval(&[Signal::One], false),
            vec![Signal::Zero]
        );
        assert_eq!(
            GateKind::Nand.eval(&[Signal::One, Signal::One], false),
            vec![Signal::Zero]
        );
        assert_eq!(
            GateKind::Nor.eval(&[Signal::Zero, Signal::Zero], false),
            vec![Signal::One]
        );
        assert_eq!(
            GateKind::Xor.eval(&[Signal::One, Signal::Zero], false),
            vec![Signal::One]
        );
        assert_eq!(
            GateKind::Xor.eval(&[Signal::One, Signal::One], false),
            vec![Signal::Zero]
        );
        assert_eq!(
            GateKind::Xnor.eval(&[Signal::One, Signal::One], false),
            vec![Signal::One]
        );
    }

    #[test]
    fn test_switch_not_led_circuit_propagation() {
        let mut circuit = Circuit::new();
        let sw = circuit.add_component(GateKind::ToggleSwitch, (0.0, 0.0));
        let not_gate = circuit.add_component(GateKind::Not, (50.0, 0.0));
        let led = circuit.add_component(GateKind::Led, (100.0, 0.0));

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

        Simulator::settle(&mut circuit);
        assert_eq!(circuit.components[sw].output_signals[0], Signal::Zero);
        assert_eq!(circuit.components[not_gate].output_signals[0], Signal::One);
        assert_eq!(circuit.components[led].input_signals[0], Signal::One);

        circuit.components[sw].state_flag = true;
        Simulator::settle(&mut circuit);
        assert_eq!(circuit.components[sw].output_signals[0], Signal::One);
        assert_eq!(circuit.components[not_gate].output_signals[0], Signal::Zero);
        assert_eq!(circuit.components[led].input_signals[0], Signal::Zero);
    }

    #[test]
    fn test_fanout_and_delete() {
        let mut circuit = Circuit::new();
        let sw = circuit.add_component(GateKind::ToggleSwitch, (0.0, 0.0));
        let led1 = circuit.add_component(GateKind::Led, (100.0, 0.0));
        let led2 = circuit.add_component(GateKind::Led, (100.0, 50.0));

        let sw_out = PortEndpoint {
            component_id: sw,
            is_output: true,
            port_index: 0,
        };
        let led1_in = PortEndpoint {
            component_id: led1,
            is_output: false,
            port_index: 0,
        };
        let led2_in = PortEndpoint {
            component_id: led2,
            is_output: false,
            port_index: 0,
        };

        circuit.connect_ports(sw_out, led1_in);
        circuit.connect_ports(sw_out, led2_in);

        circuit.components[sw].state_flag = true;
        Simulator::settle(&mut circuit);
        assert_eq!(circuit.components[led1].input_signals[0], Signal::One);
        assert_eq!(circuit.components[led2].input_signals[0], Signal::One);

        // Remove led1: led2 should remain connected and driven
        circuit.remove_component(led1);
        Simulator::settle(&mut circuit);
        assert_eq!(circuit.components[led2].input_signals[0], Signal::One);

        // Remove sw: all connected nets should be cleaned up
        circuit.remove_component(sw);
        assert_eq!(circuit.nets.len(), 0);
    }

    #[test]
    fn test_clock_tick() {
        let mut circuit = Circuit::new();
        let clk = circuit.add_component(GateKind::Clock, (0.0, 0.0));
        let led = circuit.add_component(GateKind::Led, (100.0, 0.0));

        let clk_out = PortEndpoint {
            component_id: clk,
            is_output: true,
            port_index: 0,
        };
        let led_in = PortEndpoint {
            component_id: led,
            is_output: false,
            port_index: 0,
        };
        circuit.connect_ports(clk_out, led_in);

        Simulator::settle(&mut circuit);
        assert_eq!(circuit.components[led].input_signals[0], Signal::Zero);

        Simulator::tick(&mut circuit, ClockEdge::Rising);
        assert_eq!(circuit.components[led].input_signals[0], Signal::One);

        Simulator::tick(&mut circuit, ClockEdge::Rising);
        assert_eq!(circuit.components[led].input_signals[0], Signal::Zero);
    }

    #[test]
    fn test_truth_table_generation_and_diff() {
        let mut circuit = Circuit::new();
        let sw_a = circuit.add_component(GateKind::ToggleSwitch, (0.0, 0.0));
        let sw_b = circuit.add_component(GateKind::ToggleSwitch, (0.0, 40.0));
        let and_gate = circuit.add_component(GateKind::And, (60.0, 20.0));
        let led = circuit.add_component(GateKind::Led, (120.0, 20.0));

        let sw_a_out = PortEndpoint {
            component_id: sw_a,
            is_output: true,
            port_index: 0,
        };
        let and_in_0 = PortEndpoint {
            component_id: and_gate,
            is_output: false,
            port_index: 0,
        };
        circuit.connect_ports(sw_a_out, and_in_0);

        let sw_b_out = PortEndpoint {
            component_id: sw_b,
            is_output: true,
            port_index: 0,
        };
        let and_in_1 = PortEndpoint {
            component_id: and_gate,
            is_output: false,
            port_index: 1,
        };
        circuit.connect_ports(sw_b_out, and_in_1);

        let and_out = PortEndpoint {
            component_id: and_gate,
            is_output: true,
            port_index: 0,
        };
        let led_in = PortEndpoint {
            component_id: led,
            is_output: false,
            port_index: 0,
        };
        circuit.connect_ports(and_out, led_in);

        // Generate truth table
        let (tt, _) =
            TruthTableGenerator::generate(&circuit, None).expect("Should generate truth table");
        assert_eq!(tt.rows.len(), 4);
        assert_eq!(tt.rows[0].outputs[0], Signal::Zero); // 0,0 -> 0
        assert_eq!(tt.rows[1].outputs[0], Signal::Zero); // 0,1 -> 0
        assert_eq!(tt.rows[2].outputs[0], Signal::Zero); // 1,0 -> 0
        assert_eq!(tt.rows[3].outputs[0], Signal::One); // 1,1 -> 1

        // Diff against AND reference table
        let and_ref = TruthTableGenerator::reference_table(GateKind::And);
        let diff = tt.diff(&and_ref);
        assert!(diff.is_match);
        assert_eq!(diff.mismatches.len(), 0);

        // Diff against OR reference table (should fail)
        let or_ref = TruthTableGenerator::reference_table(GateKind::Or);
        let diff_mismatch = tt.diff(&or_ref);
        assert!(!diff_mismatch.is_match);
        assert!(!diff_mismatch.mismatches.is_empty());

        // CSV and plain text formatting
        let csv = tt.to_csv();
        assert!(csv.contains("A,B,Q"));
        let txt = tt.to_plain_text();
        assert!(txt.contains("||"));
    }

    #[test]
    fn test_phase3_arithmetic_adders() {
        // Half Adder: 1 + 1 = 0, Carry 1
        let ha_res = GateKind::HalfAdder.eval(&[Signal::One, Signal::One], false);
        assert_eq!(ha_res, vec![Signal::Zero, Signal::One]); // Sum=0, Cout=1

        // Full Adder: 1 + 1 + 1 = 1, Carry 1
        let fa_res = GateKind::FullAdder.eval(&[Signal::One, Signal::One, Signal::One], false);
        assert_eq!(fa_res, vec![Signal::One, Signal::One]); // Sum=1, Cout=1

        // 4-Bit Ripple Carry Adder: 5 + 3 = 8 (0101 + 0011 = 1000)
        // A = 5 (1, 0, 1, 0), B = 3 (1, 1, 0, 0), Cin = 0
        let rca_inputs = vec![
            Signal::One,
            Signal::Zero,
            Signal::One,
            Signal::Zero,
            Signal::One,
            Signal::One,
            Signal::Zero,
            Signal::Zero,
            Signal::Zero,
        ];
        let rca_res = GateKind::RippleCarryAdder4.eval(&rca_inputs, false);
        // Result should be 8: (0, 0, 0, 1), Cout=0
        assert_eq!(
            rca_res,
            vec![
                Signal::Zero,
                Signal::Zero,
                Signal::Zero,
                Signal::One,
                Signal::Zero
            ]
        );

        // 4-Bit Carry-Lookahead Adder: 5 + 3 = 8
        let cla_res = GateKind::CarryLookaheadAdder4.eval(&rca_inputs, false);
        assert_eq!(cla_res, rca_res);

        // Test carry propagation: 15 + 1 = 16 (Sum=0, Cout=1)
        // A = 15 (1,1,1,1), B = 0 (0,0,0,0), Cin = 1
        let carry_inputs = vec![
            Signal::One,
            Signal::One,
            Signal::One,
            Signal::One,
            Signal::Zero,
            Signal::Zero,
            Signal::Zero,
            Signal::Zero,
            Signal::One,
        ];
        let rca_carry = GateKind::RippleCarryAdder4.eval(&carry_inputs, false);
        let cla_carry = GateKind::CarryLookaheadAdder4.eval(&carry_inputs, false);
        assert_eq!(
            rca_carry,
            vec![
                Signal::Zero,
                Signal::Zero,
                Signal::Zero,
                Signal::Zero,
                Signal::One
            ]
        );
        assert_eq!(cla_carry, rca_carry);
    }

    #[test]
    fn test_phase3_sequential_and_routing() {
        // 4:1 Mux: select input 2 (index 2)
        // I0=0, I1=0, I2=1, I3=0, S0=0, S1=1 (binary 2)
        let mux_inputs = vec![
            Signal::Zero,
            Signal::Zero,
            Signal::One,
            Signal::Zero,
            Signal::Zero,
            Signal::One,
        ];
        let mux_res = GateKind::Mux4.eval(&mux_inputs, false);
        assert_eq!(mux_res, vec![Signal::One]);

        // D Flip-Flop: Rising clock edge latches D
        let mut dff_node = ComponentNode::new(GateKind::DFlipFlop, (0.0, 0.0));
        dff_node.input_signals = vec![Signal::One, Signal::Zero]; // D=1, CLK=0
        let _ = Simulator::eval_node(&mut dff_node);
        assert!(!dff_node.state_flag);

        dff_node.input_signals = vec![Signal::One, Signal::One]; // CLK rises!
        let out = Simulator::eval_node(&mut dff_node);
        assert!(dff_node.state_flag);
        assert_eq!(out, vec![Signal::One, Signal::Zero]); // Q=1, !Q=0
    }

    #[test]
    fn test_phase3_alu_and_ram() {
        // 4-Bit ALU: 7 + 2 = 9 (Op=0: ADD)
        // A = 7 (1, 1, 1, 0), B = 2 (0, 1, 0, 0), Op = 0 (0, 0, 0)
        let alu_inputs = vec![
            Signal::One,
            Signal::One,
            Signal::One,
            Signal::Zero,
            Signal::Zero,
            Signal::One,
            Signal::Zero,
            Signal::Zero,
            Signal::Zero,
            Signal::Zero,
            Signal::Zero,
        ];
        let alu_res = GateKind::Alu4.eval(&alu_inputs, false);
        // 9 = 1, 0, 0, 1; Zero=0, Carry=0
        assert_eq!(
            alu_res,
            vec![
                Signal::One,
                Signal::Zero,
                Signal::Zero,
                Signal::One,
                Signal::Zero,
                Signal::Zero
            ]
        );

        // RAM 16x4: Write 5 to address 3, then read it back
        let mut ram_node = ComponentNode::new(GateKind::Ram16x4, (0.0, 0.0));
        // Addr = 3 (1, 1, 0, 0), DataIn = 5 (1, 0, 1, 0), WE = 1, CLK = 1
        ram_node.input_signals = vec![
            Signal::One,
            Signal::One,
            Signal::Zero,
            Signal::Zero, // Addr 3
            Signal::One,
            Signal::Zero,
            Signal::One,
            Signal::Zero, // Data 5
            Signal::One,  // WE
            Signal::One,  // CLK
        ];
        let ram_out = Simulator::eval_node(&mut ram_node);
        assert_eq!(
            ram_out,
            vec![Signal::One, Signal::Zero, Signal::One, Signal::Zero]
        );
    }

    #[test]
    fn test_subcircuit_creation_and_hierarchical_sim() {
        use crate::graph::SubcircuitDef;

        // 1. Build an internal circuit: XOR gate with 2 switches and 1 LED
        let mut sub = Circuit::new();
        let sw_a = sub.add_component(GateKind::ToggleSwitch, (0.0, 0.0));
        let sw_b = sub.add_component(GateKind::ToggleSwitch, (0.0, 50.0));
        let xor_gate = sub.add_component(GateKind::Xor, (100.0, 25.0));
        let led = sub.add_component(GateKind::Led, (200.0, 25.0));

        sub.connect_ports(
            PortEndpoint {
                component_id: sw_a,
                is_output: true,
                port_index: 0,
            },
            PortEndpoint {
                component_id: xor_gate,
                is_output: false,
                port_index: 0,
            },
        );
        sub.connect_ports(
            PortEndpoint {
                component_id: sw_b,
                is_output: true,
                port_index: 0,
            },
            PortEndpoint {
                component_id: xor_gate,
                is_output: false,
                port_index: 1,
            },
        );
        sub.connect_ports(
            PortEndpoint {
                component_id: xor_gate,
                is_output: true,
                port_index: 0,
            },
            PortEndpoint {
                component_id: led,
                is_output: false,
                port_index: 0,
            },
        );

        let sub_def = SubcircuitDef {
            name: "XorBlock".into(),
            input_names: vec!["A".into(), "B".into()],
            output_names: vec!["Y".into()],
            circuit: sub,
        };

        // 2. Instantiate this subcircuit in a parent circuit
        let mut parent = Circuit::new();
        parent.subcircuits.insert("XorBlock".into(), sub_def);

        let parent_sw_a = parent.add_component(GateKind::ToggleSwitch, (0.0, 0.0));
        let parent_sw_b = parent.add_component(GateKind::ToggleSwitch, (0.0, 40.0));
        let sub_inst = parent.add_component(
            GateKind::SubcircuitInstance("XorBlock".into()),
            (100.0, 20.0),
        );
        let parent_led = parent.add_component(GateKind::Led, (200.0, 20.0));

        parent.connect_ports(
            PortEndpoint {
                component_id: parent_sw_a,
                is_output: true,
                port_index: 0,
            },
            PortEndpoint {
                component_id: sub_inst,
                is_output: false,
                port_index: 0,
            },
        );
        parent.connect_ports(
            PortEndpoint {
                component_id: parent_sw_b,
                is_output: true,
                port_index: 0,
            },
            PortEndpoint {
                component_id: sub_inst,
                is_output: false,
                port_index: 1,
            },
        );
        parent.connect_ports(
            PortEndpoint {
                component_id: sub_inst,
                is_output: true,
                port_index: 0,
            },
            PortEndpoint {
                component_id: parent_led,
                is_output: false,
                port_index: 0,
            },
        );

        // Test: A=1, B=0 -> Output=1
        if let Some(c) = parent.components.get_mut(parent_sw_a) {
            c.state_flag = true;
        }
        Simulator::settle(&mut parent);
        assert_eq!(
            parent.components.get(parent_led).unwrap().input_signals[0],
            Signal::One
        );

        // Test: A=1, B=1 -> Output=0
        if let Some(c) = parent.components.get_mut(parent_sw_b) {
            c.state_flag = true;
        }
        Simulator::settle(&mut parent);
        assert_eq!(
            parent.components.get(parent_led).unwrap().input_signals[0],
            Signal::Zero
        );
    }

    #[test]
    fn test_hdl_export_verilog_and_vhdl() {
        let mut circuit = Circuit::new();
        let sw_a = circuit.add_component(GateKind::ToggleSwitch, (0.0, 0.0));
        let sw_b = circuit.add_component(GateKind::ToggleSwitch, (0.0, 40.0));
        let and_gate = circuit.add_component(GateKind::And, (100.0, 20.0));
        let led = circuit.add_component(GateKind::Led, (200.0, 20.0));

        circuit.connect_ports(
            PortEndpoint {
                component_id: sw_a,
                is_output: true,
                port_index: 0,
            },
            PortEndpoint {
                component_id: and_gate,
                is_output: false,
                port_index: 0,
            },
        );
        circuit.connect_ports(
            PortEndpoint {
                component_id: sw_b,
                is_output: true,
                port_index: 0,
            },
            PortEndpoint {
                component_id: and_gate,
                is_output: false,
                port_index: 1,
            },
        );
        circuit.connect_ports(
            PortEndpoint {
                component_id: and_gate,
                is_output: true,
                port_index: 0,
            },
            PortEndpoint {
                component_id: led,
                is_output: false,
                port_index: 0,
            },
        );

        let verilog = HdlExporter::to_verilog(&circuit, "my_and_circuit");
        assert!(verilog.contains("module my_and_circuit"));
        assert!(verilog.contains("endmodule"));
        assert!(verilog.contains("and  gate_and_"));

        let vhdl = HdlExporter::to_vhdl(&circuit, "my_and_circuit");
        assert!(vhdl.contains("entity my_and_circuit is"));
        assert!(vhdl.contains("architecture Structural of my_and_circuit is"));
        assert!(vhdl.contains("end Structural;"));
    }

    #[test]
    fn test_multi_input_gates_and_component_properties() {
        // 1. Multi-input gate boolean evaluation
        // 4-input AND: all 1s -> 1, one 0 -> 0
        assert_eq!(
            GateKind::And.eval(&[Signal::One, Signal::One, Signal::One, Signal::One], false),
            vec![Signal::One]
        );
        assert_eq!(
            GateKind::And.eval(
                &[Signal::One, Signal::One, Signal::Zero, Signal::One],
                false
            ),
            vec![Signal::Zero]
        );

        // 3-input OR: one 1 -> 1, all 0s -> 0
        assert_eq!(
            GateKind::Or.eval(&[Signal::Zero, Signal::Zero, Signal::One], false),
            vec![Signal::One]
        );
        assert_eq!(
            GateKind::Or.eval(&[Signal::Zero, Signal::Zero, Signal::Zero], false),
            vec![Signal::Zero]
        );

        // 3-input NAND: inverse of AND
        assert_eq!(
            GateKind::Nand.eval(&[Signal::One, Signal::One, Signal::One], false),
            vec![Signal::Zero]
        );
        assert_eq!(
            GateKind::Nand.eval(&[Signal::One, Signal::Zero, Signal::One], false),
            vec![Signal::One]
        );

        // 3-input NOR: inverse of OR
        assert_eq!(
            GateKind::Nor.eval(&[Signal::Zero, Signal::Zero, Signal::Zero], false),
            vec![Signal::One]
        );
        assert_eq!(
            GateKind::Nor.eval(&[Signal::Zero, Signal::One, Signal::Zero], false),
            vec![Signal::Zero]
        );

        // 3-input XOR (odd parity): 1,1,1 -> 1; 1,1,0 -> 0
        assert_eq!(
            GateKind::Xor.eval(&[Signal::One, Signal::One, Signal::One], false),
            vec![Signal::One]
        );
        assert_eq!(
            GateKind::Xor.eval(&[Signal::One, Signal::One, Signal::Zero], false),
            vec![Signal::Zero]
        );

        // 2. Component properties and dynamic reconfiguration
        let mut circuit = Circuit::new();
        let and_id = circuit.add_component(GateKind::And, (100.0, 100.0));

        // Default 2 inputs
        assert_eq!(circuit.components[and_id].input_signals.len(), 2);

        // Customize label, bit_width, rotation
        circuit.set_component_label(and_id, "U1_AND4".to_string());
        circuit.set_component_bit_width(and_id, 8);
        circuit.set_component_rotation(and_id, Rotation::R90);
        assert_eq!(circuit.components[and_id].label, "U1_AND4");
        assert_eq!(circuit.components[and_id].bit_width, 8);
        assert_eq!(circuit.components[and_id].rotation, Rotation::R90);

        // Expand to 4 inputs
        circuit.set_component_input_count(and_id, 4);
        assert_eq!(circuit.components[and_id].input_signals.len(), 4);

        // Verify dynamic dimensions scale
        let (hw, hh) = circuit.components[and_id].half_dimensions();
        assert_eq!(hw, 25.0);
        assert!(hh > 20.0); // 20.0 + (4 - 2) * 6.5 = 33.0

        // Wire inputs and test pruning upon reducing inputs
        let sw1 = circuit.add_component(GateKind::ToggleSwitch, (0.0, 0.0));
        let sw4 = circuit.add_component(GateKind::ToggleSwitch, (0.0, 150.0));
        circuit.connect_ports(
            PortEndpoint {
                component_id: sw1,
                is_output: true,
                port_index: 0,
            },
            PortEndpoint {
                component_id: and_id,
                is_output: false,
                port_index: 0,
            },
        );
        circuit.connect_ports(
            PortEndpoint {
                component_id: sw4,
                is_output: true,
                port_index: 0,
            },
            PortEndpoint {
                component_id: and_id,
                is_output: false,
                port_index: 3,
            },
        );
        assert_eq!(circuit.nets.len(), 2);

        // Reduce to 2 inputs -> port 3 wire should be pruned automatically!
        circuit.set_component_input_count(and_id, 2);
        assert_eq!(circuit.components[and_id].input_signals.len(), 2);
        assert_eq!(circuit.nets.len(), 1); // Only port 0 wire remains
    }
}

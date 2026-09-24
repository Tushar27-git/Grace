use crate::signal::Signal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortSpec {
    pub name: String,
    pub width: u8,
}

impl PortSpec {
    pub fn new(name: impl Into<String>, width: u8) -> Self {
        Self {
            name: name.into(),
            width,
        }
    }

    pub fn single_bit(name: impl Into<String>) -> Self {
        Self::new(name, 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockEdge {
    Rising,
    Falling,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateKind {
    // 1. Primitive Logic Gates
    And,
    Or,
    Not,
    Nand,
    Nor,
    Xor,
    Xnor,

    // 2. Fundamental I/O
    ToggleSwitch,
    Led,
    Clock,

    // 3. Displays
    BinaryDisplay4,
    HexDisplay,
    SevenSegment,

    // 4. Arithmetic
    HalfAdder,
    FullAdder,
    HalfSubtractor,
    FullSubtractor,
    RippleCarryAdder4,
    CarryLookaheadAdder4,

    // 5. Sequential
    SrLatch,
    DLatch,
    DFlipFlop,
    JkFlipFlop,
    TFlipFlop,
    Register4,
    ShiftRegister4,
    Counter4,
    ClockDivider,

    // 6. Computer & Routing Blocks
    Mux2,
    Mux4,
    Demux2,
    Demux4,
    Encoder4to2,
    Decoder2to4,
    Comparator2,
    Alu4,
    Rom16x4,
    Ram16x4,

    // 7. Custom Hierarchical Subcircuit
    SubcircuitInstance(String),
}

impl GateKind {
    pub fn display_name(&self) -> String {
        match self {
            GateKind::And => "AND Gate".into(),
            GateKind::Or => "OR Gate".into(),
            GateKind::Not => "NOT Gate".into(),
            GateKind::Nand => "NAND Gate".into(),
            GateKind::Nor => "NOR Gate".into(),
            GateKind::Xor => "XOR Gate".into(),
            GateKind::Xnor => "XNOR Gate".into(),
            GateKind::ToggleSwitch => "Toggle Switch".into(),
            GateKind::Led => "LED".into(),
            GateKind::Clock => "Clock Source".into(),
            GateKind::BinaryDisplay4 => "4-Bit Binary Display".into(),
            GateKind::HexDisplay => "Hex Display".into(),
            GateKind::SevenSegment => "7-Segment Display".into(),
            GateKind::HalfAdder => "Half Adder".into(),
            GateKind::FullAdder => "Full Adder".into(),
            GateKind::HalfSubtractor => "Half Subtractor".into(),
            GateKind::FullSubtractor => "Full Subtractor".into(),
            GateKind::RippleCarryAdder4 => "4-Bit Ripple Carry Adder".into(),
            GateKind::CarryLookaheadAdder4 => "4-Bit Carry-Lookahead Adder".into(),
            GateKind::SrLatch => "SR Latch".into(),
            GateKind::DLatch => "D Latch".into(),
            GateKind::DFlipFlop => "D Flip-Flop".into(),
            GateKind::JkFlipFlop => "JK Flip-Flop".into(),
            GateKind::TFlipFlop => "T Flip-Flop".into(),
            GateKind::Register4 => "4-Bit Register".into(),
            GateKind::ShiftRegister4 => "4-Bit Shift Register".into(),
            GateKind::Counter4 => "4-Bit Counter".into(),
            GateKind::ClockDivider => "Clock Divider (/2, /4)".into(),
            GateKind::Mux2 => "2:1 Multiplexer".into(),
            GateKind::Mux4 => "4:1 Multiplexer".into(),
            GateKind::Demux2 => "1:2 Demultiplexer".into(),
            GateKind::Demux4 => "1:4 Demultiplexer".into(),
            GateKind::Encoder4to2 => "4:2 Priority Encoder".into(),
            GateKind::Decoder2to4 => "2:4 Decoder".into(),
            GateKind::Comparator2 => "2-Bit Comparator".into(),
            GateKind::Alu4 => "4-Bit ALU".into(),
            GateKind::Rom16x4 => "16x4 ROM".into(),
            GateKind::Ram16x4 => "16x4 RAM".into(),
            GateKind::SubcircuitInstance(name) => format!("IC: {}", name),
        }
    }

    pub fn short_code(&self) -> String {
        match self {
            GateKind::And => "AND".into(),
            GateKind::Or => "OR".into(),
            GateKind::Not => "NOT".into(),
            GateKind::Nand => "NAND".into(),
            GateKind::Nor => "NOR".into(),
            GateKind::Xor => "XOR".into(),
            GateKind::Xnor => "XNOR".into(),
            GateKind::ToggleSwitch => "SW".into(),
            GateKind::Led => "LED".into(),
            GateKind::Clock => "CLK".into(),
            GateKind::BinaryDisplay4 => "BIN4".into(),
            GateKind::HexDisplay => "HEX".into(),
            GateKind::SevenSegment => "7SEG".into(),
            GateKind::HalfAdder => "HA".into(),
            GateKind::FullAdder => "FA".into(),
            GateKind::HalfSubtractor => "HS".into(),
            GateKind::FullSubtractor => "FS".into(),
            GateKind::RippleCarryAdder4 => "ADD4".into(),
            GateKind::CarryLookaheadAdder4 => "CLA4".into(),
            GateKind::SrLatch => "SR".into(),
            GateKind::DLatch => "DLAT".into(),
            GateKind::DFlipFlop => "DFF".into(),
            GateKind::JkFlipFlop => "JKFF".into(),
            GateKind::TFlipFlop => "TFF".into(),
            GateKind::Register4 => "REG4".into(),
            GateKind::ShiftRegister4 => "SHF4".into(),
            GateKind::Counter4 => "CNT4".into(),
            GateKind::ClockDivider => "DIV".into(),
            GateKind::Mux2 => "MUX2".into(),
            GateKind::Mux4 => "MUX4".into(),
            GateKind::Demux2 => "DMX2".into(),
            GateKind::Demux4 => "DMX4".into(),
            GateKind::Encoder4to2 => "ENC".into(),
            GateKind::Decoder2to4 => "DEC".into(),
            GateKind::Comparator2 => "CMP".into(),
            GateKind::Alu4 => "ALU4".into(),
            GateKind::Rom16x4 => "ROM".into(),
            GateKind::Ram16x4 => "RAM".into(),
            GateKind::SubcircuitInstance(name) => {
                let code: String = name.chars().take(4).collect();
                code.to_uppercase()
            }
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            GateKind::And
            | GateKind::Or
            | GateKind::Not
            | GateKind::Nand
            | GateKind::Nor
            | GateKind::Xor
            | GateKind::Xnor => "Logic Gates",
            GateKind::ToggleSwitch | GateKind::Led | GateKind::Clock => "Input / Output",
            GateKind::BinaryDisplay4 | GateKind::HexDisplay | GateKind::SevenSegment => "Displays",
            GateKind::HalfAdder
            | GateKind::FullAdder
            | GateKind::HalfSubtractor
            | GateKind::FullSubtractor
            | GateKind::RippleCarryAdder4
            | GateKind::CarryLookaheadAdder4 => "Arithmetic",
            GateKind::SrLatch
            | GateKind::DLatch
            | GateKind::DFlipFlop
            | GateKind::JkFlipFlop
            | GateKind::TFlipFlop
            | GateKind::Register4
            | GateKind::ShiftRegister4
            | GateKind::Counter4
            | GateKind::ClockDivider => "Sequential",
            GateKind::Mux2
            | GateKind::Mux4
            | GateKind::Demux2
            | GateKind::Demux4
            | GateKind::Encoder4to2
            | GateKind::Decoder2to4
            | GateKind::Comparator2 => "Routing & Selection",
            GateKind::Alu4 | GateKind::Rom16x4 | GateKind::Ram16x4 => "Computer Blocks",
            GateKind::SubcircuitInstance(_) => "Subcircuits",
        }
    }

    pub fn input_count(&self) -> usize {
        match self {
            GateKind::ToggleSwitch | GateKind::Clock => 0,
            GateKind::Not | GateKind::Led => 1,
            GateKind::And
            | GateKind::Or
            | GateKind::Nand
            | GateKind::Nor
            | GateKind::Xor
            | GateKind::Xnor
            | GateKind::HalfAdder
            | GateKind::HalfSubtractor
            | GateKind::SrLatch
            | GateKind::DLatch
            | GateKind::DFlipFlop
            | GateKind::TFlipFlop
            | GateKind::Demux2
            | GateKind::Decoder2to4
            | GateKind::ClockDivider => 2,
            GateKind::FullAdder
            | GateKind::FullSubtractor
            | GateKind::JkFlipFlop
            | GateKind::Mux2 => 3,
            GateKind::BinaryDisplay4
            | GateKind::HexDisplay
            | GateKind::Demux4
            | GateKind::Encoder4to2
            | GateKind::Comparator2 => 4,
            GateKind::Register4 | GateKind::ShiftRegister4 | GateKind::Counter4 => 5,
            GateKind::Mux4 => 6,
            GateKind::SevenSegment => 7,
            GateKind::RippleCarryAdder4 | GateKind::CarryLookaheadAdder4 => 9,
            GateKind::Alu4 => 11,                 // 4 A + 4 B + 3 Opcode
            GateKind::Rom16x4 => 4,               // 4-bit Address
            GateKind::Ram16x4 => 10,              // 4 Addr + 4 DataIn + WE + CLK
            GateKind::SubcircuitInstance(_) => 2, // default, customized per subcircuit definition
        }
    }

    pub fn output_count(&self) -> usize {
        match self {
            GateKind::Led
            | GateKind::BinaryDisplay4
            | GateKind::HexDisplay
            | GateKind::SevenSegment => 0,
            GateKind::And
            | GateKind::Or
            | GateKind::Not
            | GateKind::Nand
            | GateKind::Nor
            | GateKind::Xor
            | GateKind::Xnor
            | GateKind::ToggleSwitch
            | GateKind::Clock
            | GateKind::Mux2
            | GateKind::Mux4 => 1,
            GateKind::HalfAdder
            | GateKind::FullAdder
            | GateKind::HalfSubtractor
            | GateKind::FullSubtractor
            | GateKind::SrLatch
            | GateKind::DLatch
            | GateKind::DFlipFlop
            | GateKind::JkFlipFlop
            | GateKind::TFlipFlop
            | GateKind::Demux2
            | GateKind::Encoder4to2
            | GateKind::ClockDivider => 2,
            GateKind::Comparator2 => 3, // A>B, A=B, A<B
            GateKind::Demux4
            | GateKind::Decoder2to4
            | GateKind::Register4
            | GateKind::ShiftRegister4
            | GateKind::Counter4
            | GateKind::Rom16x4
            | GateKind::Ram16x4 => 4,
            GateKind::RippleCarryAdder4 | GateKind::CarryLookaheadAdder4 => 5, // 4 Sum + Cout
            GateKind::Alu4 => 6, // 4 Result + Zero + Carry
            GateKind::SubcircuitInstance(_) => 1,
        }
    }

    pub fn is_sequential(&self) -> bool {
        matches!(
            self,
            GateKind::Clock
                | GateKind::SrLatch
                | GateKind::DLatch
                | GateKind::DFlipFlop
                | GateKind::JkFlipFlop
                | GateKind::TFlipFlop
                | GateKind::Register4
                | GateKind::ShiftRegister4
                | GateKind::Counter4
                | GateKind::ClockDivider
                | GateKind::Ram16x4
        )
    }

    pub fn port_label_in(&self, idx: usize) -> &'static str {
        match self {
            GateKind::And
            | GateKind::Or
            | GateKind::Nand
            | GateKind::Nor
            | GateKind::Xor
            | GateKind::Xnor => match idx {
                0 => "A",
                _ => "B",
            },
            GateKind::Not => "A",
            GateKind::Led => "IN",
            GateKind::HalfAdder | GateKind::HalfSubtractor => match idx {
                0 => "A",
                _ => "B",
            },
            GateKind::FullAdder | GateKind::FullSubtractor => match idx {
                0 => "A",
                1 => "B",
                _ => "Cin",
            },
            GateKind::SrLatch => match idx {
                0 => "S",
                _ => "R",
            },
            GateKind::DLatch => match idx {
                0 => "D",
                _ => "EN",
            },
            GateKind::DFlipFlop => match idx {
                0 => "D",
                _ => "CLK",
            },
            GateKind::JkFlipFlop => match idx {
                0 => "J",
                1 => "K",
                _ => "CLK",
            },
            GateKind::TFlipFlop => match idx {
                0 => "T",
                _ => "CLK",
            },
            GateKind::ClockDivider => match idx {
                0 => "CLK",
                _ => "RST",
            },
            GateKind::Mux2 => match idx {
                0 => "I0",
                1 => "I1",
                _ => "S",
            },
            GateKind::Mux4 => match idx {
                0 => "I0",
                1 => "I1",
                2 => "I2",
                3 => "I3",
                4 => "S0",
                _ => "S1",
            },
            GateKind::Demux2 => match idx {
                0 => "IN",
                _ => "S",
            },
            GateKind::Demux4 => match idx {
                0 => "IN",
                1 => "EN",
                2 => "S0",
                _ => "S1",
            },
            GateKind::Comparator2 => match idx {
                0 => "A0",
                1 => "A1",
                2 => "B0",
                _ => "B1",
            },
            GateKind::RippleCarryAdder4 | GateKind::CarryLookaheadAdder4 => match idx {
                0..=3 => ["A0", "A1", "A2", "A3"][idx],
                4..=7 => ["B0", "B1", "B2", "B3"][idx - 4],
                _ => "Cin",
            },
            _ => "IN",
        }
    }

    pub fn port_label_out(&self, idx: usize) -> &'static str {
        match self {
            GateKind::And
            | GateKind::Or
            | GateKind::Not
            | GateKind::Nand
            | GateKind::Nor
            | GateKind::Xor
            | GateKind::Xnor
            | GateKind::ToggleSwitch
            | GateKind::Mux2
            | GateKind::Mux4 => "Q",
            GateKind::Clock => "CLK",
            GateKind::HalfAdder | GateKind::FullAdder => match idx {
                0 => "SUM",
                _ => "COUT",
            },
            GateKind::HalfSubtractor | GateKind::FullSubtractor => match idx {
                0 => "DIFF",
                _ => "BOUT",
            },
            GateKind::SrLatch
            | GateKind::DLatch
            | GateKind::DFlipFlop
            | GateKind::JkFlipFlop
            | GateKind::TFlipFlop => match idx {
                0 => "Q",
                _ => "!Q",
            },
            GateKind::ClockDivider => match idx {
                0 => "/2",
                _ => "/4",
            },
            GateKind::Comparator2 => match idx {
                0 => "A>B",
                1 => "A=B",
                _ => "A<B",
            },
            GateKind::RippleCarryAdder4 | GateKind::CarryLookaheadAdder4 => match idx {
                0..=3 => ["S0", "S1", "S2", "S3"][idx],
                _ => "Cout",
            },
            _ => "OUT",
        }
    }

    pub fn eval(&self, inputs: &[Signal], state_flag: bool) -> Vec<Signal> {
        let mut node = crate::graph::ComponentNode::new(self.clone(), (0.0, 0.0));
        node.input_signals = inputs.to_vec();
        node.state_flag = state_flag;
        crate::sim::Simulator::eval_node(&mut node)
    }
}

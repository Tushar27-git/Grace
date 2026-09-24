use crate::component::GateKind;
use crate::signal::Signal;
use serde::{Deserialize, Serialize};
use slotmap::{SlotMap, new_key_type};
use std::collections::HashMap;

new_key_type! {
    pub struct ComponentId;
    pub struct NetId;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Rotation {
    #[default]
    R0,
    R90,
    R180,
    R270,
}

impl Rotation {
    pub fn next(self) -> Self {
        match self {
            Rotation::R0 => Rotation::R90,
            Rotation::R90 => Rotation::R180,
            Rotation::R180 => Rotation::R270,
            Rotation::R270 => Rotation::R0,
        }
    }

    pub fn angle_rad(self) -> f32 {
        match self {
            Rotation::R0 => 0.0,
            Rotation::R90 => std::f32::consts::FRAC_PI_2,
            Rotation::R180 => std::f32::consts::PI,
            Rotation::R270 => 3.0 * std::f32::consts::FRAC_PI_2,
        }
    }

    pub fn rotate_point(self, pt: (f32, f32)) -> (f32, f32) {
        match self {
            Rotation::R0 => pt,
            Rotation::R90 => (-pt.1, pt.0),
            Rotation::R180 => (-pt.0, -pt.1),
            Rotation::R270 => (pt.1, -pt.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortEndpoint {
    pub component_id: ComponentId,
    pub is_output: bool,
    pub port_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcircuitDef {
    pub name: String,
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
    pub circuit: Circuit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentNode {
    pub kind: GateKind,
    pub pos: (f32, f32),
    pub rotation: Rotation,
    /// For toggle switches: on/off; for clocks: tick state; for latches: Q state
    pub state_flag: bool,
    /// Internal memory / register state (RAM, ROM, registers, counters)
    pub memory: Vec<u8>,
    /// Previous clock signal for edge detection (0 -> 1 rising edge)
    pub clock_prev: bool,
    /// Cached input signals
    pub input_signals: Vec<Signal>,
    /// Cached output signals
    pub output_signals: Vec<Signal>,
    /// Custom label / tag for component
    #[serde(default)]
    pub label: String,
    /// Configured bit width
    #[serde(default = "default_bit_width")]
    pub bit_width: u8,
    /// Custom clock frequency in Hz for clock sources
    #[serde(default = "default_clock_hz")]
    pub clock_hz: f32,
}

fn default_bit_width() -> u8 {
    1
}

fn default_clock_hz() -> f32 {
    2.0
}

impl ComponentNode {
    pub fn new(kind: GateKind, pos: (f32, f32)) -> Self {
        let input_count = kind.input_count();
        let output_count = kind.output_count();

        // Default memory initialization
        let memory = match kind {
            GateKind::Rom16x4 => {
                // Initialize ROM with 16 4-bit nibbles (Fibonacci / Powers: 0, 1, 1, 2, 3, 5, 8, 13, 1, 2, 4, 8, 3, 6, 12, 15)
                vec![0, 1, 1, 2, 3, 5, 8, 13, 1, 2, 4, 8, 3, 6, 12, 15]
            }
            GateKind::Ram16x4 => vec![0; 16],
            GateKind::Counter4 | GateKind::Register4 | GateKind::ShiftRegister4 => vec![0; 4],
            _ => Vec::new(),
        };

        Self {
            kind,
            pos,
            rotation: Rotation::R0,
            state_flag: false,
            memory,
            clock_prev: false,
            input_signals: vec![Signal::Zero; input_count],
            output_signals: vec![Signal::Zero; output_count],
            label: String::new(),
            bit_width: 1,
            clock_hz: 2.0,
        }
    }

    pub fn input_count(&self) -> usize {
        self.input_signals.len()
    }

    pub fn output_count(&self) -> usize {
        self.output_signals.len()
    }

    pub fn set_input_count(&mut self, new_count: usize) {
        let count = new_count.clamp(1, 16);
        self.input_signals.resize(count, Signal::Zero);
    }

    pub fn set_output_count(&mut self, new_count: usize) {
        let count = new_count.clamp(1, 16);
        self.output_signals.resize(count, Signal::Zero);
    }

    pub fn port_offset_unrotated(&self, is_output: bool, port_index: usize) -> (f32, f32) {
        let (half_w, half_h) = self.half_dimensions();

        let count = if is_output {
            self.output_signals.len()
        } else {
            self.input_signals.len()
        };

        let x_pos = if is_output {
            match self.kind {
                GateKind::Nand | GateKind::Nor | GateKind::Xnor => half_w + 17.0,
                _ => half_w + 10.0,
            }
        } else {
            -half_w - 10.0
        };

        if count <= 1 {
            (x_pos, 0.0)
        } else {
            let available_span = (half_h * 2.0 - 16.0).max(16.0);
            let step = (available_span / (count as f32 - 1.0)).min(20.0);
            let start_y = -((count - 1) as f32) * step * 0.5;
            (x_pos, start_y + (port_index as f32) * step)
        }
    }

    pub fn half_dimensions(&self) -> (f32, f32) {
        match self.kind {
            GateKind::ToggleSwitch | GateKind::Clock => (20.0, 15.0),
            GateKind::Led => (18.0, 18.0),
            GateKind::BinaryDisplay4 => (40.0, 18.0),
            GateKind::HexDisplay => (25.0, 25.0),
            GateKind::SevenSegment => (25.0, 35.0),
            GateKind::Not => (25.0, 18.0),
            GateKind::And
            | GateKind::Or
            | GateKind::Nand
            | GateKind::Nor
            | GateKind::Xor
            | GateKind::Xnor => {
                let inputs = self.input_signals.len();
                let extra = inputs.saturating_sub(2) as f32;
                (25.0, 20.0 + extra * 6.5)
            }
            GateKind::HalfAdder | GateKind::HalfSubtractor => (30.0, 24.0),
            GateKind::FullAdder | GateKind::FullSubtractor => (35.0, 30.0),
            GateKind::RippleCarryAdder4 | GateKind::CarryLookaheadAdder4 | GateKind::Alu4 => {
                (50.0, 55.0)
            }
            GateKind::SrLatch | GateKind::DLatch | GateKind::DFlipFlop | GateKind::TFlipFlop => {
                (30.0, 25.0)
            }
            GateKind::JkFlipFlop => (32.0, 30.0),
            GateKind::Register4 | GateKind::ShiftRegister4 | GateKind::Counter4 => (38.0, 38.0),
            GateKind::ClockDivider => (30.0, 25.0),
            GateKind::Mux2 | GateKind::Demux2 => (30.0, 25.0),
            GateKind::Mux4 | GateKind::Demux4 => (35.0, 35.0),
            GateKind::Encoder4to2 | GateKind::Decoder2to4 | GateKind::Comparator2 => (35.0, 35.0),
            GateKind::Rom16x4 | GateKind::Ram16x4 => (45.0, 45.0),
            GateKind::SubcircuitInstance(_) => {
                let max_ports = self
                    .input_signals
                    .len()
                    .max(self.output_signals.len())
                    .max(2);
                let h = (max_ports as f32 * 14.0 + 16.0).max(35.0);
                (40.0, h)
            }
        }
    }

    pub fn port_world_pos(&self, is_output: bool, port_index: usize) -> (f32, f32) {
        let unrotated = self.port_offset_unrotated(is_output, port_index);
        let rotated = self.rotation.rotate_point(unrotated);
        (self.pos.0 + rotated.0, self.pos.1 + rotated.1)
    }

    pub fn bounding_box(&self) -> ((f32, f32), (f32, f32)) {
        let (half_w, half_h) = self.half_dimensions();
        let (rx, ry) = match self.rotation {
            Rotation::R0 | Rotation::R180 => (half_w, half_h),
            Rotation::R90 | Rotation::R270 => (half_h, half_w),
        };
        (
            (self.pos.0 - rx, self.pos.1 - ry),
            (self.pos.0 + rx, self.pos.1 + ry),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Net {
    pub source: PortEndpoint,
    pub sinks: Vec<PortEndpoint>,
    pub current_signal: Signal,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Circuit {
    pub components: SlotMap<ComponentId, ComponentNode>,
    pub nets: SlotMap<NetId, Net>,
    pub subcircuits: HashMap<String, SubcircuitDef>,
}

impl Circuit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_component(&mut self, kind: GateKind, pos: (f32, f32)) -> ComponentId {
        let node = ComponentNode::new(kind, pos);
        self.components.insert(node)
    }

    pub fn remove_component(&mut self, id: ComponentId) {
        if self.components.remove(id).is_some() {
            let dead_nets: Vec<NetId> = self
                .nets
                .iter()
                .filter(|(_, net)| net.source.component_id == id)
                .map(|(nid, _)| nid)
                .collect();
            for nid in dead_nets {
                self.nets.remove(nid);
            }

            let mut empty_nets = Vec::new();
            for (nid, net) in &mut self.nets {
                net.sinks.retain(|sink| sink.component_id != id);
                if net.sinks.is_empty() {
                    empty_nets.push(nid);
                }
            }
            for nid in empty_nets {
                self.nets.remove(nid);
            }
        }
    }

    pub fn connect_ports(&mut self, source: PortEndpoint, sink: PortEndpoint) -> Option<NetId> {
        if !source.is_output || sink.is_output {
            return None;
        }
        if source.component_id == sink.component_id {
            return None;
        }

        self.disconnect_sink(sink);

        let existing_net_id = self
            .nets
            .iter()
            .find(|(_, net)| net.source == source)
            .map(|(nid, _)| nid);

        let initial_signal = self
            .components
            .get(source.component_id)
            .and_then(|c| c.output_signals.get(source.port_index).copied())
            .unwrap_or(Signal::Zero);

        if let Some(net_id) = existing_net_id {
            if let Some(net) = self.nets.get_mut(net_id) {
                if !net.sinks.contains(&sink) {
                    net.sinks.push(sink);
                }
                net.current_signal = initial_signal;
            }
            Some(net_id)
        } else {
            let net = Net {
                source,
                sinks: vec![sink],
                current_signal: initial_signal,
                label: None,
            };
            Some(self.nets.insert(net))
        }
    }

    pub fn disconnect_sink(&mut self, sink: PortEndpoint) {
        let mut empty_nets = Vec::new();
        for (nid, net) in &mut self.nets {
            net.sinks.retain(|s| *s != sink);
            if net.sinks.is_empty() {
                empty_nets.push(nid);
            }
        }
        for nid in empty_nets {
            self.nets.remove(nid);
        }
        if let Some(comp) = self.components.get_mut(sink.component_id)
            && let Some(sig) = comp.input_signals.get_mut(sink.port_index)
        {
            *sig = Signal::Zero;
        }
    }

    pub fn find_net_driving_sink(&self, sink: PortEndpoint) -> Option<NetId> {
        self.nets
            .iter()
            .find(|(_, net)| net.sinks.contains(&sink))
            .map(|(nid, _)| nid)
    }

    pub fn find_net_from_source(&self, source: PortEndpoint) -> Option<NetId> {
        self.nets
            .iter()
            .find(|(_, net)| net.source == source)
            .map(|(nid, _)| nid)
    }

    pub fn set_component_input_count(&mut self, id: ComponentId, count: usize) {
        if let Some(comp) = self.components.get_mut(id) {
            comp.set_input_count(count);
        }
        // Remove any wire connected to ports that were pruned
        let mut empty_nets = Vec::new();
        for (nid, net) in &mut self.nets {
            net.sinks
                .retain(|s| !(s.component_id == id && s.port_index >= count));
            if net.sinks.is_empty() {
                empty_nets.push(nid);
            }
        }
        for nid in empty_nets {
            self.nets.remove(nid);
        }
    }

    pub fn set_component_output_count(&mut self, id: ComponentId, count: usize) {
        if let Some(comp) = self.components.get_mut(id) {
            comp.set_output_count(count);
        }
        // Remove any nets originating from ports that were pruned
        let dead_nets: Vec<NetId> = self
            .nets
            .iter()
            .filter(|(_, net)| net.source.component_id == id && net.source.port_index >= count)
            .map(|(nid, _)| nid)
            .collect();
        for nid in dead_nets {
            self.nets.remove(nid);
        }
    }

    pub fn set_component_rotation(&mut self, id: ComponentId, rot: Rotation) {
        if let Some(comp) = self.components.get_mut(id) {
            comp.rotation = rot;
        }
    }

    pub fn set_component_label(&mut self, id: ComponentId, label: String) {
        if let Some(comp) = self.components.get_mut(id) {
            comp.label = label;
        }
    }

    pub fn set_component_bit_width(&mut self, id: ComponentId, bit_width: u8) {
        if let Some(comp) = self.components.get_mut(id) {
            comp.bit_width = bit_width;
        }
    }

    pub fn set_component_clock_hz(&mut self, id: ComponentId, clock_hz: f32) {
        if let Some(comp) = self.components.get_mut(id) {
            comp.clock_hz = clock_hz;
        }
    }
}

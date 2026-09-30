# Logic Lab (Logic Circuit Simulator & Design Laboratory)

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui%20%2F%20eframe%200.36-blue.svg)](https://github.com/emilk/egui)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-15%20passed-brightgreen.svg)]()
[![Sim Engine](https://img.shields.io/badge/simulation-5M%20ops%2Fs-purple.svg)]()

> A high-performance, tactile, minimal digital logic circuit editor and simulation laboratory written in 100% pure Rust. Design complex combinatorial and sequential circuits on an infinite canvas, wire components with real-time feedback, verify truth tables, explore universal gate synthesis, compose hierarchical subcircuits, inspect digital waveforms, and export directly to Verilog and VHDL.

---

## Table of Contents

- [Vision & Design Philosophy](#vision--design-philosophy)
- [Visual Design System](#visual-design-system)
- [Architecture & Simulation Model](#architecture--simulation-model)
- [Benchmark & Performance Report](#benchmark--performance-report)
- [Feature Matrix](#feature-matrix)
  - [Phase 1: Core Simulation Loop & Canvas](#phase-1-core-simulation-loop--canvas)
  - [Phase 2: Formal Verification & Universal Gates Lab](#phase-2-formal-verification--universal-gates-lab)
  - [Phase 3: Multi-Bit Systems, Arithmetic & Hierarchy](#phase-3-multi-bit-systems-arithmetic--hierarchy)
  - [Phase 4: Advanced Tooling, Waveforms & HDL Synthesis](#phase-4-advanced-tooling-waveforms--hdl-synthesis)
- [Component Library](#component-library)
- [Keyboard Shortcuts & Workflows](#keyboard-shortcuts--workflows)
- [Installation & Getting Started](#installation--getting-started)
- [Running Benchmarks & Tests](#running-benchmarks--tests)
- [Circuit Serialization Format (.ron)](#circuit-serialization-format-ron)
- [Android App & APK](#android-app--apk)
- [Project Structure](#project-structure)
- [License](#license)

---

## Vision & Design Philosophy

**Logic Lab** is designed from the ground up to feel like a **precision desktop instrument** rather than a web-wrapper or SaaS dashboard:

- **Tactile & Responsive**: Instantaneous event-driven simulation that never drops frames, delivering a consistent 60+ FPS experience even with hundreds of gates active.
- **Cute, Technical, Minimal**: Characterized by a hand-crafted dark-and-pink aesthetic (`#151217` panel base with `#FF4FA3` energetic signal glow).
- **ANSI / IEC Vector Glyphs**: Every gate is rendered using mathematically accurate geometric primitives (flat-back D-shapes for AND, curved shield hulls for OR, bezier leads for XOR, crisp inversion bubbles) drawn directly with `egui::Painter` — **zero generic text-box placeholders or emoji**.
- **Strict Separation of Concerns**: The simulation engine (`logic-core`) is completely detached from the UI layer (`logic-app`), containing zero graphical dependencies, zero IPC, and zero async runtimes.
- **Zero AI Clichés**: No rainbow gradient decorations, no bloated hero cards, no glassmorphism plastered across every panel, and no decorative animations that slow down workflow.

---

## Visual Design System

The application strictly adheres to a centralized color token palette defined in [`theme.rs`](file:///d:/DLCD/logic-app/src/theme.rs):

| Token | Hex Value | Semantic Usage |
|:---|:---:|:---|
| `bg_canvas` | `#F4F1EC` | Circuit canvas background (warm off-white paper canvas in light mode) |
| `bg_panel` | `#151217` | Sidebar palettes, top toolbars, status bar (deep near-black with subtle purple undertone) |
| `bg_panel_raised` | `#1E1A21` | Floating dialogs, verification tables, inspectors, and modal sheets |
| `accent_purple` | `#6E4C9E` | Structural accents, panel headers, borders, and inactive selection outlines |
| `accent_pink` | `#FF4FA3` | Active/HIGH signal state, primary action buttons, active selection halo, hover glow |
| `accent_red` | `#E14B4B` | Reserved strictly for delete actions, syntax errors, and truth-table mismatches |
| `text_primary` | `#EDEAF0` | Crisp high-legibility text on dark panels |
| `text_on_canvas` | `#2A2630` | Pin labels, gate annotations, and net tags on paper canvas |
| `grid_line` | `#DDD8CE` | Low-contrast canvas dot/line grid (aligned to 20px spacing) |
| `signal_low` | `#4A4550` | De-energized wire and port pin state (`0` / False) |
| `signal_high` | `#FF4FA3` | Energized wire and port pin state (`1` / True) |
| `signal_x` | `#B8A8C4` | Undefined/contention state (drawn with diagonal cross-hatching) |
| `signal_z` | *Outline* | High-impedance disconnected state (drawn as a hollow pin/wire) |

### Shapes & Typography
- **Corner Radii**: 4px on small interactive controls (buttons, chips), 8px on panels and modals.
- **Outlines**: 1.0–1.5px subtle purple on idle elements; 2px pink highlight on hover or selection.
- **Shadows**: Precise 2px Y-offset soft drop-shadows on floating inspectors and drag previews.
- **Typography**: Clean proportional sans-serif for UI controls paired with monospace fonts for truth tables, hexadecimal registers, and memory addresses.

---

## Architecture & Simulation Model

Logic Lab is structured as a two-crate Cargo workspace:

```
logic-lab/
├── logic-core/               # Pure simulation engine (zero UI dependencies)
│   ├── src/
│   │   ├── signal.rs         # 4-state logic (Zero, One, X, Z)
│   │   ├── component.rs      # SimComponent trait, GateKind definitions
│   │   ├── graph.rs          # SlotMap-backed circuit graph, ports, nets, buses
│   │   ├── sim.rs            # Event-driven queue propagation engine (settle & tick)
│   │   ├── verify.rs         # Exhaustive truth-table generator & diff verifier
│   │   └── hdl.rs            # Verilog-2001 and VHDL-93/2008 synthesizers
│   └── benches/
│       └── simulator_benchmark.rs  # Standalone criterion-grade performance harness
└── logic-app/                # Native desktop GUI powered by egui & eframe
    └── src/
        ├── app.rs            # Application state, undo/redo, file I/O, breadcrumbs
        ├── canvas.rs         # Infinite pan/zoom, grid snapping, wire engine, marquee
        ├── glyphs.rs         # ANSI/IEC vector symbols rendered via egui::Painter
        ├── palette.rs        # Categorized component library & quick-add search
        ├── properties_ui.rs  # Live component property inspector
        ├── verification_ui.rs# Truth table analyzer & Universal Gates Lab
        ├── waveform.rs       # Digital timing analyzer & logic trace viewer
        ├── hdl_ui.rs         # HDL code export & syntax review modal
        ├── settings_ui.rs    # Preferences, theme toggling, updater & keybindings
        ├── updater.rs        # GitHub release checker and binary self-updater
        ├── tools.rs          # Tool state machine (Select, Pan, Wire, Marquee)
        └── theme.rs          # Design token definitions and egui style overrides
```

### The Event-Driven Simulation Engine

Traditional educational simulators evaluate every component in the schematic every graphical frame ($O(N)$ per render pass). In contrast, `logic-core` uses an **event-driven queue**:
1. When an input changes (e.g. flipping a switch or clock rising edge), only the nets connected to that source output are marked dirty.
2. Only downstream components connected to those dirty nets are queued for evaluation (`SimComponent::eval`).
3. If a component's output transitions, its fanout nets are pushed onto the propagation queue.
4. Combinational feedback cycles and ripple networks propagate until the network reaches equilibrium (`settle()`), guarded by cycle-detection thresholds ($O(k)$ where $k \ll N$).
5. Sequential components (flip-flops, registers, counters) sample their inputs on synchronous clock edges (`tick(ClockEdge)`), updating internal states before running a settle pass.

### Unidirectional Data Flow

```mermaid
graph LR
    User[User Input] -->|Click / Drag| Canvas[Canvas Event]
    Canvas -->|Mutate Graph| Core[Circuit Graph (SlotMap)]
    Core -->|Dirty Net Queue| Sim[Simulator::settle / tick]
    Sim -->|Signal Propagation| Core
    Core -->|Read Node Signals| Renderer[egui::Painter / Glyphs]
    Renderer -->|60 FPS V-Sync| Screen[Display Output]
```

The UI never maintains shadow simulation state. Every wire color, LED brightness, and 7-segment digit is rendered directly from `logic-core`'s canonical truth state on every frame.

---

## Benchmark & Performance Report

Engine performance was measured on an AMD / Intel x86_64 host running Rust 1.85+ with release profile optimizations (`cargo bench -p logic-core`).

```
================================================================================
                       LOGIC LAB SIMULATOR BENCHMARK SUITE                      
                       Engine: logic-core v1.0.0 (Rust 2024 / release)          
================================================================================
```

### 1. Raw Gate Evaluation Throughput (1,000,000 evaluations per gate)

| Gate Type | Total Time (ms) | Throughput (ops/sec) | Average Latency |
|:---|:---:|:---:|:---:|
| **AND** | 255.72 ms | **3.91 × 10⁶ ops/s** | 255.72 ns |
| **OR** | 242.96 ms | **4.12 × 10⁶ ops/s** | 242.96 ns |
| **NOT** | 258.33 ms | **3.87 × 10⁶ ops/s** | 258.33 ns |
| **NAND** | 314.72 ms | **3.18 × 10⁶ ops/s** | 314.72 ns |
| **NOR** | 304.91 ms | **3.28 × 10⁶ ops/s** | 304.91 ns |
| **XOR** | 251.89 ms | **3.97 × 10⁶ ops/s** | 251.89 ns |
| **XNOR** | 216.13 ms | **4.63 × 10⁶ ops/s** | 216.13 ns |
| **Half Adder** | 279.86 ms | **3.57 × 10⁶ ops/s** | 279.86 ns |
| **Full Adder** | 392.10 ms | **2.55 × 10⁶ ops/s** | 392.10 ns |
| **4-bit ALU** | 319.16 ms | **3.13 × 10⁶ ops/s** | 319.16 ns |

### 2. Inverter Cascade Convergence (Ripple Propagation Settle)

Tests ripple settle time across serial chains of NOT gates driven by a single toggle switch:

| Cascade Depth | Graph Build Time (ms) | Settle Convergence Time (ms) | Convergence Status |
|:---|:---:|:---:|:---:|
| **10 Inverters** | 0.071 ms | **0.004 ms** (4.0 µs) | Converged |
| **50 Inverters** | 0.038 ms | **0.017 ms** (17.0 µs) | Converged |
| **100 Inverters** | 0.056 ms | **0.038 ms** (38.0 µs) | Converged |
| **250 Inverters** | 0.196 ms | **0.137 ms** (137.0 µs) | Converged |
| **500 Inverters** | 0.670 ms | **0.277 ms** (277.0 µs) | Converged |
| **1,000 Inverters** | 3.024 ms | **0.941 ms** (941.0 µs) | Converged |

*Even an extreme 1,000-gate serial cascade completes event propagation in under 1 millisecond.*

### 3. Complex Arithmetic Block Settle Throughput (10,000 full-circuit settles)

| Circuit Subsystem | Total Time (ms) | Settle Throughput | Latency per Settle |
|:---|:---:|:---:|:---:|
| **4-Bit Ripple Carry Adder** | 21.45 ms | **4.66 × 10⁵ settles/s** | 2.15 µs |
| **4-Bit Carry Lookahead Adder** | 25.25 ms | **3.96 × 10⁵ settles/s** | 2.53 µs |
| **4-Bit Complete ALU (Add/Sub/Logic)** | 35.87 ms | **2.79 × 10⁵ settles/s** | 3.59 µs |
| **Full Subtractor** | 13.47 ms | **7.43 × 10⁵ settles/s** | 1.35 µs |

### 4. Synchronous Clock Tick Throughput (100,000 active clock edge cycles)

| Sequential Subsystem | Total Time (ms) | Equivalent Clock Frequency | Edge Latency |
|:---|:---:|:---:|:---:|
| **D Flip-Flop** | 54.72 ms | **1.83 MHz** | 547.24 ns |
| **JK Flip-Flop** | 72.28 ms | **1.38 MHz** | 722.78 ns |
| **T Flip-Flop** | 42.87 ms | **2.33 MHz** | 428.74 ns |
| **4-Bit Parallel Register** | 39.42 ms | **2.54 MHz** | 394.23 ns |
| **4-Bit Shift Register** | 40.92 ms | **2.44 MHz** | 409.18 ns |
| **4-Bit Synchronous Counter** | 37.89 ms | **2.64 MHz** | 378.94 ns |

### 5. Truth Table Exhaustive Sweep Generation

| Sub-Circuit | Input Vector Width | Total State Combinations | Synthesis Time |
|:---|:---:|:---:|:---:|
| **2-Input XOR** | 2 bits | 4 states | **0.030 ms** (30 µs) |
| **Full Adder (A, B, Cin)** | 3 bits | 8 states | **0.023 ms** (23 µs) |
| **4-Input Multiplexer** | 4 bits | 16 states | **0.057 ms** (57 µs) |

### 6. Hardware Description Language (HDL) Synthesis Speed

Synthesizing a 100-component schematic into industry-standard RTL code:

| Target Language Standard | Components | Generation Time | Synthesis Throughput |
|:---|:---:|:---:|:---:|
| **Verilog 2001 (IEEE 1364-2001)** | 100 gates | **0.102 ms** | **9,820 circuits/sec** |
| **VHDL 93 / 2008 (IEEE 1076)** | 100 gates | **0.110 ms** | **9,051 circuits/sec** |

---

## Feature Matrix

### Phase 1: Core Simulation Loop & Canvas
- **Infinite Virtual Canvas**: Smooth panning (middle-click or Space+drag) and exponential zoom (scroll wheel) with configurable zoom presets (25% to 400%).
- **Interactive Grid System**: Dot-matrix canvas grid with automatic coordinate alignment, snap-to-grid toggle, and dark/paper canvas modes.
- **Vector Symbol Glyphs**: Accurate ANSI/IEC standard shapes for all logic primitives rendered with smooth anti-aliased curves.
- **Tactile Wiring System**:
  - Click-and-drag from output pins to input pins.
  - Automatic orthogonal route guidance.
  - Wire junction nodes (dots) where multi-wire nets intersect.
  - Net hover illumination: hovering any wire segment highlights the entire electrical net in energetic pink.
- **Manipulation Primitives**:
  - Box marquee multi-selection.
  - Shift-click cumulative selection.
  - Live drag-and-drop movement.
  - 90° clockwise rotation (`R` key).
  - Component duplication (`Ctrl+D`).
  - Delete with automatic net cleanup (`Delete` / `Backspace`).
- **History & Persistence**:
  - Full multi-level snapshot Undo/Redo (`Ctrl+Z`, `Ctrl+Y`).
  - Human-readable `.ron` file serialization with native file picker dialogs (`rfd`).

### Phase 2: Formal Verification & Universal Gates Lab
- **Automatic Boundary Analysis**: Select any arbitrary cluster of gates on canvas and the analyzer automatically identifies all floating inputs (switches/constants) and observing outputs (LEDs/displays).
- **Interactive Truth Table Generator**:
  - Exhaustive state generation for up to 16 input vectors ($2^{16}$ states).
  - Sortable input/output columns.
  - **Live State Injection**: Click any row in the generated truth table to immediately drive the canvas schematic with those exact logic states.
  - Plaintext and CSV export, with one-click clipboard copy.
- **Truth Table Diff Verifier**:
  - Paste an expected reference truth table or select a known specification.
  - The verifier compares actual circuit behavior row-by-row against the reference.
  - Any mismatching row is highlighted with `accent_red` indicating the failing input vector, expected bit, and actual simulated bit.
- **Universal Gates Laboratory**:
  - Interactive challenge workshop dedicated to constructing elementary logic using only NAND or NOR gates.
  - Scaffolds for: NOT from NAND, AND from NAND, OR from NAND, XOR from NAND, NOT from NOR, OR from NOR, AND from NOR, XNOR from NOR.
  - One-click formal verification against reference truth tables with instant pass/fail validation.

### Phase 3: Multi-Bit Systems, Arithmetic & Hierarchy
- **Multi-Bit Buses & Routing**:
  - 2-bit, 4-bit, 8-bit, and 16-bit bus wires.
  - Bus Splitters: expand a packed multi-bit bus into individual 1-bit lines.
  - Bus Joiners: bundle individual signal lines into a composite bus.
  - Wire Net Labels for non-physical net connection across large schematics.
- **Bus-Aware Numeric Displays**:
  - Single-bit LED indicators.
  - 4-bit Binary Bitfield Displays.
  - Hexadecimal 7-Segment Displays.
  - Dual 2-Digit 7-Segment Displays with integrated BCD decoding.
- **Arithmetic Logic Units**:
  - Combinational Half Adder and Full Adder blocks.
  - Half Subtractor and Full Subtractor blocks.
  - 4-bit Ripple-Carry Adder with carry-in and carry-out flags.
  - 4-bit Arithmetic Logic Unit (ALU) supporting ADD, SUB, AND, OR, XOR, NOT, PASS, and ZERO.
- **Sequential Storage & Timing**:
  - SR Latch, D Latch.
  - Edge-triggered D Flip-Flop, JK Flip-Flop, and T Flip-Flop.
  - 4-bit Parallel-In/Parallel-Out (PIPO) Register with synchronous load and clear.
  - 4-bit Shift-Left / Shift-Right Register.
  - 4-bit Synchronous Modulo Counter with reset.
  - Configurable Frequency Master Clock (0.5 Hz to 50 Hz).
- **Data Routing & Memory**:
  - Multiplexers: 2:1, 4:1, and 8:1 data selectors.
  - Demultiplexers: 1:2, 1:4, and 1:8 decoders.
  - 4-to-2 Priority Encoder and 2-to-4 Binary Decoder.
  - 4-bit Magnitude Comparator ($A > B$, $A = B$, $A < B$).
  - 16 × 4-bit Random Access Memory (RAM) with read/write enable.
  - 16 × 4-bit Programmable Read-Only Memory (ROM) with interactive hex table editor.
- **Hierarchical Subcircuits**:
  - Select any functional circuit region $\rightarrow$ Click **"Create Subcircuit"**.
  - Encapsulates the schematic into a reusable custom chip package with named pins.
  - Place multiple instances of custom subcircuits onto parent canvases.
  - **Hierarchical Schematics**: Double-click any subcircuit chip to push into its internal schematic; navigate upward using breadcrumb links.

### Phase 4: Advanced Tooling, Waveforms & HDL Synthesis
- **4-bit Carry Lookahead Adder (CLA)**: Ultra-low latency addition utilizing fast generate ($G_i$) and propagate ($P_i$) carry lookahead logic.
- **Multi-Channel Digital Waveform Analyzer**:
  - Real-time digital logic analyzer docked at the canvas base.
  - Probes any selected wire or bus in the schematic.
  - Tracks signal transitions ($0$, $1$, $X$, $Z$) over sequential clock ticks.
  - Zoomable time base, channel toggles, clear trace, and pause/resume recording.
- **Hardware Description Language (HDL) Synthesis**:
  - 1-click RTL synthesis from graphical canvas schematics.
  - Generates synthesizable **Verilog-2001** (`.v`) and **VHDL-93/2008** (`.vhd`).
  - Supports gate-level instantiations, net declarations, hierarchical subcircuit modules, and input/output port mapping.
- **Workflow & Productivity Upgrades**:
  - **Single vs. Multi-Stamp Placement**: Quick-place one component or hold Shift / toggle multi-stamp to place repeating gates continuously.
  - **Component Inspector**: Real-time property panel for modifying gate input counts (2 to 8 inputs), clock frequencies, memory contents, and labels.
  - **Quick-Add Search (Space / Tab)**: Fuzzy-search popup for instantaneous component insertion without lifting hands from keyboard.
  - **Auto-Updater**: Embedded release inspector checking GitHub releases and downloading newer binaries with a single click.

---

## Component Library

Logic Lab provides a comprehensive library of 40+ built-in digital components:

```
├── Logic Gates
│   ├── Buffer (1-in, 1-out)
│   ├── Tri-State Buffer (Data in, Enable in, Output)
│   ├── NOT Gate (Inverter)
│   ├── AND Gate (2–8 inputs)
│   ├── OR Gate (2–8 inputs)
│   ├── NAND Gate (Universal, 2–8 inputs)
│   ├── NOR Gate (Universal, 2–8 inputs)
│   ├── XOR Gate (Exclusive-OR, 2–8 inputs)
│   └── XNOR Gate (Equivalence, 2–8 inputs)
├── Input / Output Primitives
│   ├── Toggle Switch (Bistable logic level 0/1)
│   ├── Push Button (Momentary contact)
│   ├── Constant High / VCC (Fixed 1)
│   ├── Constant Low / GND (Fixed 0)
│   ├── Clock Source (Configurable 0.5–50 Hz square wave)
│   ├── LED Indicator (Single-bit luminescent probe)
│   ├── Hex Digit Display (4-bit binary to hex 0–F)
│   ├── 7-Segment Display (Direct a–g cathode lines)
│   └── Dual 2-Digit 7-Segment Display (8-bit value display)
├── Arithmetic Units
│   ├── Half Adder (A, B → Sum, Carry)
│   ├── Full Adder (A, B, Cin → Sum, Cout)
│   ├── Half Subtractor (A, B → Diff, Borrow)
│   ├── Full Subtractor (A, B, Bin → Diff, Bout)
│   ├── 4-Bit Ripple Carry Adder
│   └── 4-Bit Carry Lookahead Adder (CLA)
├── Sequential Memory & Registers
│   ├── SR Latch (Active-high Set/Reset)
│   ├── D Latch (Transparent data latch)
│   ├── D Flip-Flop (Positive edge-triggered)
│   ├── JK Flip-Flop (Universal sequential element)
│   ├── T Flip-Flop (Toggle counter element)
│   ├── 4-Bit Parallel Register (Load, Clear, D0..D3)
│   ├── 4-Bit Shift Register (Serial In, Parallel Out)
│   └── 4-Bit Synchronous Counter (Clock, Reset, Q0..Q3)
├── Routing & Data Path
│   ├── 2:1, 4:1, and 8:1 Multiplexers
│   ├── 1:2, 1:4, and 1:8 Demultiplexers
│   ├── 4-to-2 Priority Encoder
│   ├── 2-to-4 Binary Decoder
│   ├── 4-Bit Magnitude Comparator (A>B, A=B, A<B)
│   ├── 4-Bit Arithmetic Logic Unit (ALU)
│   ├── Bus Splitter (Bus to individual pins)
│   └── Bus Joiner (Individual pins to Bus)
└── Memory & Custom Hierarchy
    ├── 16 × 4-bit Static RAM (CS, WE, Addr[3:0], DataIn[3:0])
    ├── 16 × 4-bit Programmable ROM (CS, Addr[3:0], DataOut[3:0])
    └── Custom Hierarchical Subcircuit Chips
```

---

## Keyboard Shortcuts & Workflows

| Shortcut | Context | Action |
|:---|:---:|:---|
| `Space` + Drag | Canvas | Pan camera viewport |
| `Middle Mouse` + Drag | Canvas | Pan camera viewport |
| `Scroll Wheel` | Canvas | Zoom in / Zoom out centered on cursor |
| `Left Click` | Canvas | Select component or wire segment |
| `Shift` + `Left Click` | Canvas | Toggle component in multi-selection |
| `Drag on Canvas` | Empty Area | Marquee box multi-select |
| `R` | Selection | Rotate selected components 90° clockwise |
| `Ctrl` + `D` | Selection | Duplicate selected components with offset |
| `Delete` / `Backspace` | Selection | Delete selected components and connected nets |
| `Ctrl` + `Z` | Global | Undo last schematic modification |
| `Ctrl` + `Y` / `Ctrl`+`Shift`+`Z` | Global | Redo previously undone action |
| `Ctrl` + `S` | Global | Save circuit to `.ron` file |
| `Ctrl` + `O` | Global | Open / Load `.ron` circuit file |
| `Ctrl` + `N` | Global | Create a fresh blank circuit canvas |
| `Tab` / `Space` (Tap) | Canvas | Open Quick-Add component fuzzy search menu |
| `Escape` | Global | Cancel active wire, close popups, clear selection |

---

## Installation & Getting Started

### Prerequisites

- [Rust Toolchain](https://rustup.rs/) (edition 2024 / version 1.85 or later recommended)
- Standard C compiler and graphics runtime drivers (OpenGL / DirectX / Vulkan supported via `wgpu`)

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/Tushar27-git/Grace.git logic-lab
   cd logic-lab
   ```

2. Run the application in release mode:
   ```bash
   cargo run -p logic-app --release
   ```

*(Note: Always run with `--release` for full 60 FPS performance and optimal simulation throughput).*

---

## Running Benchmarks & Tests

### Running the Benchmark Suite

To execute the official criterion-grade simulation benchmark:

```bash
cargo bench -p logic-core
```

This evaluates:
- Raw single-gate evaluation latency and throughput across all gate types.
- Inverter cascade ripple convergence across 10 to 1,000 chained stages.
- Arithmetic convergence speed (ALU, adders, subtractors).
- Synchronous sequential clock frequencies across flip-flops, registers, and counters.
- Full truth table synthesis and state sweeps.
- Verilog and VHDL HDL code generation speed.

### Running Workspace Unit & Integration Tests

```bash
cargo test --all-targets
```

Logic Lab includes comprehensive automated test suites covering:
- Combinational gate evaluation truth tables.
- Net fanout and cascading deletion safety.
- Switch-to-inverter-to-LED live signal propagation.
- Multi-input gates (3–8 inputs) and property bounds.
- Synchronous clock edge advancement and sequential state retention.
- ALU operations and RAM read/write cycles.
- Subcircuit packaging, nested instantiation, and hierarchical simulation.
- HDL export validity for Verilog and VHDL standards.
- Canvas wire candidate filtering and Quick-Add component search ranking.

---

## Circuit Serialization Format (.ron)

Logic Lab saves circuit schematics using **Rusty Object Notation (`.ron`)**. It is human-readable, version-control friendly, and mirrors the in-memory SlotMap graph:

```ron
Circuit(
    components: {
        0: ComponentNode(
            kind: ToggleSwitch,
            pos: (-120.0, -40.0),
            rotation: North,
            input_signals: [],
            output_signals: [One],
            state_flag: true,
            label: Some("Input A"),
        ),
        1: ComponentNode(
            kind: Not,
            pos: (0.0, -40.0),
            rotation: North,
            input_signals: [One],
            output_signals: [Zero],
            state_flag: false,
            label: None,
        ),
        2: ComponentNode(
            kind: Led,
            pos: (120.0, -40.0),
            rotation: North,
            input_signals: [Zero],
            output_signals: [],
            state_flag: false,
            label: Some("Output"),
        ),
    },
    nets: {
        0: Net(
            source: PortEndpoint(component_id: 0, is_output: true, port_index: 0),
            sinks: [
                PortEndpoint(component_id: 1, is_output: false, port_index: 0),
            ],
            current_signal: One,
        ),
        1: Net(
            source: PortEndpoint(component_id: 1, is_output: true, port_index: 0),
            sinks: [
                PortEndpoint(component_id: 2, is_output: false, port_index: 0),
            ],
            current_signal: Zero,
        ),
    },
    subcircuits: {},
)
```

---

## Android App & APK

Logic Lab is also packaged as an offline, hardware-accelerated **Android APK** designed for touchscreens:

- **Touch-Optimized Canvas**: Smooth single-finger panning, two-finger pinch-to-zoom, touch-to-wire routing, and live toggle switches.
- **Full Feature Parity**: Real-time event-driven 4-state simulation, Truth Table Generator with live row injection, Universal Gates Lab, Digital Waveforms, and Verilog/VHDL code generation.
- **100% Offline & Native**: Bundled standalone inside a signed APK. No internet or external servers required.
- **Universal Android Compatibility**: Supports Android 7.0 Nougat through Android 15/16 (API 24 to 36+).

### Installing the APK on Android

1. Download [`LogicLab.apk`](file:///d:/DLCD/LogicLab.apk) (also located at `dist/LogicLab.apk`) to your Android device or install via USB:
   ```bash
   adb install LogicLab.apk
   ```
2. Open the file on your device and tap **Install** (allow installation from this source if prompted).
3. Tap the **Logic Lab** icon on your home screen to launch.

### Rebuilding the Android APK

To rebuild the APK from source using the automated Android toolchain:
```bash
cd android-build
build_apk.bat
```
The compiled, signed, and zipaligned binary will be output to `dist/LogicLab.apk`.

---

## Project Structure

```
d:\DLCD\
├── Cargo.lock
├── Cargo.toml                    # Workspace definition (members: logic-core, logic-app)
├── README.md                     # Comprehensive documentation & architecture guide
├── logic-core/                   # Pure Rust simulation engine
│   ├── Cargo.toml
│   ├── benches/
│   │   └── simulator_benchmark.rs# High-throughput benchmark suite
│   └── src/
│       ├── lib.rs                # Library root & integration tests
│       ├── signal.rs             # 4-state logic (Zero, One, X, Z)
│       ├── component.rs          # GateKind, SimComponent trait, pin specs
│       ├── graph.rs              # Circuit, ComponentNode, Net, SlotMap graph
│       ├── sim.rs                # Event-driven Simulator (settle & tick)
│       ├── verify.rs             # TruthTableGenerator & diff verifier
│       └── hdl.rs                # Verilog & VHDL synthesizer
└── logic-app/                    # egui / eframe desktop application
    ├── Cargo.toml
    └── src/
        ├── main.rs               # Desktop entrypoint & native window setup
        ├── app.rs                # Main LogicApp struct, menus, status, hierarchy
        ├── canvas.rs             # Infinite canvas, wire routing, hit-testing
        ├── glyphs.rs             # Custom ANSI/IEC vector symbol painter
        ├── palette.rs            # Categorized component palette & quick-add
        ├── properties_ui.rs      # Dynamic property inspector
        ├── verification_ui.rs    # Truth table dialog & Universal Gates Lab
        ├── waveform.rs           # Logic analyzer & timing diagram inspector
        ├── hdl_ui.rs             # Verilog/VHDL export modal
        ├── settings_ui.rs        # Settings, theme modes, updates & keybindings
        ├── updater.rs            # GitHub releases auto-updater
        ├── tools.rs              # Tool state machine
        └── theme.rs              # Design tokens and egui visual style
```

---

## License

Dual-licensed under either:
- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

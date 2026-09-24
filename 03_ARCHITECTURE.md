# Architecture

## Workspace layout

```
logic-lab/
├── Cargo.toml                 # workspace root
├── logic-core/                # simulation engine, zero UI deps
│   └── src/
│       ├── lib.rs
│       ├── signal.rs          # Signal enum: Zero, One, X, Z
│       ├── component.rs       # SimComponent trait + built-in gate impls
│       ├── graph.rs           # Circuit graph: components, ports, nets, buses
│       ├── sim.rs             # event-driven propagation engine, tick/settle
│       └── verify.rs          # truth-table generation + expected-vs-actual diff
└── logic-app/                 # egui/eframe UI, depends on logic-core
    └── src/
        ├── main.rs
        ├── theme.rs           # color tokens + egui::Style setup from 01_DESIGN.md
        ├── canvas.rs          # pan/zoom, grid, hit-testing, rendering
        ├── palette.rs         # left-side categorized component palette
        ├── tools.rs           # select/marquee/wire/rotate tool state machine
        ├── glyphs.rs          # Painter-based gate/component symbol drawing
        └── app.rs             # top-level App struct, egui::App impl
```

## `logic-core`: the simulation model

```rust
pub enum Signal { Zero, One, X, Z }

pub struct PortSpec { pub name: String, pub width: u8 } // width 1 = single bit

pub trait SimComponent {
    fn ports_in(&self) -> &[PortSpec];
    fn ports_out(&self) -> &[PortSpec];
    /// Pure combinational evaluation given current input signals.
    fn eval(&self, inputs: &[Signal]) -> Vec<Signal>;
    /// Called once per clock edge for stateful components. Default: no-op.
    fn tick(&mut self, _edge: ClockEdge) {}
    fn is_sequential(&self) -> bool { false }
}
```

- The circuit graph (`graph.rs`) holds components (keyed with `slotmap` so IDs stay stable through deletes), nets connecting output ports to input ports, and buses as grouped nets.
- The simulator (`sim.rs`) is **event-driven**: when a net's value changes, only the components with an input on that net are re-evaluated, queued, and their outputs propagated further. Do not re-evaluate the entire graph every frame — that's what breaks the 60fps target as circuits grow.
- Distinguish two operations clearly: `settle()` — run combinational propagation to a stable state with no clock advance — and `tick(edge)` — advance sequential state by one clock edge, then settle. Live simulation in the UI calls these on user interaction (a switch flip, a clock pulse) — not on every UI frame regardless of whether anything changed.
- `verify.rs` reuses this same graph model to enumerate all input combinations of a selected sub-graph, run `settle()` for each, and produce a truth table — no UI logic, no shortcuts baked in here, since this same code path also powers the Universal Gates verifier later.

## `logic-app`: UI structure

- `app.rs` holds one `App` struct: the current `Circuit` (from `logic-core`), UI-only state (camera pan/zoom, current tool, selection set, undo/redo stacks), and implements `eframe::App::update`.
- `canvas.rs` draws the grid, all components (via `glyphs.rs`), all wires, and handles pointer input for pan/zoom/select/drag/marquee — translating screen coordinates to canvas-space coordinates once, consistently.
- Selection, move, delete, duplicate, rotate all operate on the `Circuit` graph directly (through `logic-core`'s public API) — the UI never mutates simulation state through a side channel.
- Undo/redo: snapshot-based to start (clone the `Circuit` before each mutating action, push to a stack) — simple and correct; only move to a diff-based command pattern later if snapshot cloning becomes a measured performance problem.
- Save/load: serialize `Circuit` with `serde` to `.ron` via `rfd`'s save dialog. Keep the save format equal to the in-memory graph shape as closely as possible — no separate "file format" translation layer to maintain in Phase 1.

## Data flow, one interaction end to end (example: user flips a toggle switch)

1. `canvas.rs` detects a click on a `ToggleSwitch` component's hit-box.
2. It calls a method on the `Circuit` (in `logic-core`) to flip that component's output signal.
3. The circuit's `settle()` runs event-driven propagation from that changed net outward.
4. `canvas.rs` re-reads the now-updated signal values from the `Circuit` on the next paint and colors wires/pins accordingly using the `01_DESIGN.md` signal tokens.

This same one-directional flow (UI event → mutate core → settle/tick → re-render from core state) should hold for every interaction — the UI never holds its own shadow copy of signal state.

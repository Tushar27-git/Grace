# Phased Plan

Work strictly in order. Do not begin a phase until the previous phase's checklist is fully checked and working end-to-end, not just compiling.

## Phase 0 — Scaffold
- [ ] Cargo workspace created exactly per `03_ARCHITECTURE.md`
- [ ] `logic-app` opens a blank `eframe` window with the theme from `01_DESIGN.md` applied (canvas background, panel colors, font) — even with nothing on the canvas yet
- [ ] `logic-core` compiles standalone with zero UI dependencies

## Phase 1 — Core loop (this is the MVP; get it feeling good before anything else)
- [ ] Infinite pan (space+drag or middle-mouse) and zoom (scroll) canvas with dot/line grid, snap-to-grid on by default, grid visibility toggle
- [ ] Left palette panel with: AND, OR, NOT, NAND, NOR, XOR, XNOR, toggle switch, LED, clock
- [ ] Drag a component from the palette onto the canvas; components render as real gate glyphs per `01_DESIGN.md`, not boxes with text
- [ ] Select (click), multi-select (shift-click), marquee select (drag on empty canvas), move (drag selection), delete, duplicate, rotate (90° increments)
- [ ] Wire tool: click-drag from an output port to an input port creates a wire; junction dots where wires meet; hovering a wire/net highlights the whole connected net
- [ ] Undo/redo, copy/paste, and at minimum: Ctrl+Z/Y, Ctrl+C/V, Delete, Ctrl+D (duplicate)
- [ ] Live simulation: flipping a toggle switch immediately propagates through the graph and updates LED/wire colors per the signal-state tokens, at 60fps with no visible lag on a small circuit (~20 components)
- [ ] Save/load a circuit to/from a `.ron` file

**Do not start Phase 2 until all of the above works by actually clicking through it, not just "the code exists."**

## Phase 2 — Verification tooling
- [ ] Select a region of the circuit → auto-detect its input pins (unconnected switch/const inputs) and output pins (LEDs/displays) → "Generate Truth Table"
- [ ] Truth table UI: sortable columns, click a row to drive the canvas to that exact input combination live, export to CSV and plain text, copy to clipboard
- [ ] Verification mode: user provides/pastes an expected truth table; app diffs expected vs actual and visually flags mismatching rows with the `accent_red` token, showing input combo / expected / actual per mismatch
- [ ] Universal Gates Lab section: scaffolds for building NOT/AND/OR/XOR out of NAND-only or NOR-only gates, each with a one-click "Verify" that runs the same truth-table diff against the known-correct table

## Phase 3 — Breadth
- [ ] Multi-bit buses: bus wire type, bus connector/splitter/joiner components, named nets/wire labels
- [ ] Bus-aware displays: binary display, hex display, 7-segment, two-digit 7-segment
- [ ] Adders/subtractors: half adder, full adder, half/full subtractor, ripple-carry adder — each with truth-table verification available
- [ ] Sequential library: SR latch, D latch, D/JK/T flip-flop, register, shift register, counter, clock divider; configurable clock frequency component
- [ ] Computer-design blocks: multiplexer, demultiplexer, encoder, decoder, comparator, ALU, ROM, RAM
- [ ] Subcircuits: select a region → "Create Subcircuit" → produces a named reusable component with defined input/output (and optional bus) ports; double-click opens/edits its internals; supports nesting for hierarchical designs

## Phase 4 — Deferred (do not build until explicitly asked)
- [ ] Carry-lookahead adder
- [ ] Waveform viewer
- [ ] HDL export
- [ ] Real GPU-blur glassmorphism panels (see `01_DESIGN.md` glassmorphism note)

## After each phase

Report back against that phase's checklist item-by-item (done / not done), note anything you deviated from and why, and wait before continuing.

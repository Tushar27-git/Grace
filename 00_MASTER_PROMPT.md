# Master Prompt — paste this first, into a repo containing the other 5 files

You are building **Logic Lab** (working name), a Rust desktop application for designing and simulating digital logic circuits, using `egui`/`eframe`. This is a from-scratch build — any lab software mentioned in screenshots is reference for *feature scope only*; do not imitate its UI, code, or structure.

Before writing any code, read these files in this order, fully:

1. `01_DESIGN.md` — the exact visual language (colors, shapes, spacing, signal states). Non-negotiable — don't substitute your own defaults.
2. `02_TECH_STACK.md` — the crates to use and how to add them. Don't introduce a dependency not listed there without stopping to ask first.
3. `03_ARCHITECTURE.md` — the workspace layout, the core simulation trait, and how the UI and simulation crates talk to each other.
4. `04_PHASES.md` — the build order. **Work one phase at a time.** Do not start Phase *N+1* until every item in Phase *N*'s "definition of done" checklist is real and working, and you've said so explicitly.
5. `05_GUARDRAILS.md` — rules you follow throughout, including when to stop and ask instead of guessing.

## What to do right now

1. Confirm you've read all 5 files by summarizing the Phase 1 scope back in 3-4 sentences before writing code.
2. Scaffold the Cargo workspace exactly as described in `03_ARCHITECTURE.md`.
3. Implement Phase 1 from `04_PHASES.md` only. Nothing from Phase 2+.
4. Apply the visual system from `01_DESIGN.md` from the very first window that appears — don't build with default egui gray theme "for now, style it later." The aesthetic is a first-class requirement, not polish added at the end.
5. When Phase 1's checklist is fully done, stop, report what's done against the checklist, and wait before moving to Phase 2.

## The one-line pitch, if you need to explain the app to yourself

A digital logic circuit editor: an infinite grid canvas where you drag gates from a palette, wire them together by hand, flip switches and watch signals propagate live, and eventually generate/verify truth tables, build with universal gates, and compose reusable subcircuits — styled minimal, cute, dark-and-pink, zero "AI app" visual clichés.

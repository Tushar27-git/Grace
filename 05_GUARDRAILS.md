# Guardrails

Rules that apply throughout the whole build, every phase.

## Scope discipline
- Implement exactly one phase from `04_PHASES.md` at a time. Do not "get ahead" and scaffold Phase 3 components while working on Phase 1, even if it seems efficient — it isn't; it makes review harder and usually means re-doing work once the core interaction model settles.
- If a requirement is ambiguous, make the smallest reasonable assumption, state it explicitly in your report, and keep moving — don't block on it, but don't silently guess on anything architecturally significant (e.g. save format, the `SimComponent` trait shape) without flagging it.

## Dependencies
- Only use crates listed in `02_TECH_STACK.md` without asking. Anything else: stop and propose it with a one-line reason before adding it.
- Never add a crate "just in case" for a future phase. Add it when that phase actually needs it.

## Code quality
- Idiomatic, `clippy`-clean Rust. Run `cargo clippy` and `cargo fmt` before considering a phase done.
- `logic-core` must never import `egui` or any UI crate — if you find yourself wanting to, that logic belongs in `logic-app` instead.
- No dead code, no commented-out experiments left in the tree, no TODO-and-abandon — either finish it or don't add it.

## Design fidelity
- Follow `01_DESIGN.md` color tokens and shape rules exactly. No inline hex colors scattered through widget code — reference the theme constants.
- If the aesthetic ever conflicts with an egui default (e.g. default button padding, default rounding), override it — don't ship the default and call it done.
- No emoji, gradients-as-decoration, neon glow, or generic dashboard/hero-section patterns anywhere, per `01_DESIGN.md`.

## Communication
- After each phase, report against that phase's checklist explicitly — don't just say "Phase 1 done," list each item.
- If something in these documents turns out to be wrong once you're actually building (a crate deprecated, an approach that doesn't perform), say so and propose the fix rather than silently working around it.
- Never claim a feature works without having actually run the app and exercised it.

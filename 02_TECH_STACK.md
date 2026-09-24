# Tech Stack

Pure Rust, single binary, no web technologies, no IPC layer. Two-crate Cargo workspace (see `03_ARCHITECTURE.md`).

## Setup, if not already done

```bash
# Rust toolchain (skip if already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# from the repo root
cargo new --lib logic-core
cargo new logic-app
```

Then create a root `Cargo.toml` workspace file:

```toml
[workspace]
members = ["logic-core", "logic-app"]
resolver = "2"
```

## Core dependencies — check each is not already present in `Cargo.toml` before adding; if already installed and in use, prefer the installed version rather than adding a duplicate

Run these from inside each crate's directory (`cargo add` fetches the current compatible version automatically — don't hardcode version numbers from memory, let Cargo resolve them):

**In `logic-app/` (UI):**
```bash
cargo add eframe
cargo add egui
cargo add egui_extras          # for image/svg loading if needed for icons
cargo add rfd                  # native file dialogs (save/load/export)
cargo add arboard               # clipboard (copy/paste of selections)
cargo add serde --features derive
cargo add ron                  # human-readable save-file format for circuits
```

**In `logic-core/` (simulation engine — must stay UI-free, no egui import here at all):**
```bash
cargo add serde --features derive
cargo add slotmap              # stable IDs for components/nets in the graph, survives removal
cargo add thiserror            # error types for the simulation/validation layer
```
`logic-app` depends on `logic-core` as a path dependency:
```toml
# in logic-app/Cargo.toml
[dependencies]
logic-core = { path = "../logic-core" }
```

## Rules for adding anything beyond this list

- If a phase genuinely needs something not listed here (e.g. a vector-path helper crate like `lyon` for complex gate glyphs, or `egui-phosphor` for a small icon set), search crates.io for current, actively-maintained options rather than assuming a remembered crate name/version still applies — the Rust GUI ecosystem moves fast.
- Never add a crate that pulls in a full second GUI toolkit, a web framework, or async runtime (`tokio`, etc.) — this app has no network I/O and doesn't need one.
- Prefer `egui::Painter` custom drawing over pulling in a heavier vector-graphics crate for gate glyphs; only reach for `lyon` if hand-rolled bezier paths genuinely become unmanageable.

## Build & run

```bash
cargo run -p logic-app --release   # release mode matters for the 60fps target once circuits get larger
```

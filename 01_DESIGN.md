# Design System

Goal: minimal, cute, technical, tactile. Reads as a hand-crafted tool, not a generated SaaS dashboard or an "AI app." No emoji, no gradients-as-decoration, no neon glow, no giant hero sections, no glassmorphism plastered everywhere.

## Color tokens

Define these once in code (a `Theme` struct or const table) and use them everywhere — never inline a hex color in a widget.

| Token | Hex | Use |
|---|---|---|
| `bg_canvas` | `#F4F1EC` | the circuit canvas itself (warm off-white "paper") |
| `bg_panel` | `#151217` | palette / toolbar / side panel backgrounds (near-black, slight purple tint) |
| `bg_panel_raised` | `#1E1A21` | floating panels, modals, dropdowns |
| `accent_purple` | `#6E4C9E` | structural accents: panel headers, borders, tool icons, non-active selection outline |
| `accent_pink` | `#FF4FA3` | the one "energetic" color: active/HIGH signal, primary buttons, active selection, hover glow |
| `accent_red` | `#E14B4B` | reserved *only* for delete, error, and truth-table mismatch states — never decorative |
| `text_primary` | `#EDEAF0` | text on dark panels |
| `text_on_canvas` | `#2A2630` | text/labels drawn on the canvas |
| `grid_line` | `#DDD8CE` | canvas dot/line grid, low contrast against `bg_canvas` |
| `signal_low` | `#4A4550` (desaturated gray-purple) | wire/pin drawn in state 0 |
| `signal_high` | `accent_pink` | wire/pin drawn in state 1 |
| `signal_x` | hatched pattern, base `#B8A8C4` | unknown state |
| `signal_z` | hollow/outline only, no fill | high-impedance |

## Shape & spacing

- Corner radius: 4px on small controls (buttons, chips), 8px on panels/cards. Never more than 10px — this is "tiny rounded corners," not bubbly.
- Outlines: 1–1.5px, `accent_purple` at low opacity on unselected components, full `accent_pink` at 2px on selected/hovered.
- Shadows: one soft shadow style only (small blur radius, low opacity, offset y+2px), used on: dragged component, floating toolbar, modal/dialog. Not on every panel — a shadow on everything reads as "generated UI."
- Grid: dot-grid or thin-line grid at a fixed spacing (e.g. 20px at zoom 1.0), `grid_line` color, toggleable off entirely. Snap-to-grid on by default.

## Glassmorphism — how far to actually take it

egui has no built-in blur-behind-content effect; a true frosted-glass look needs a custom `wgpu` paint callback sampling the framebuffer, which is real engineering effort. Two options, pick based on how much time you want to spend on this:

- **Cheap approximation (do this first):** semi-transparent panel fill (`bg_panel_raised` at ~85% alpha) + a 1px light top border + the standard soft shadow above. This reads as "glassy" at a glance and costs nothing.
- **Real blur (optional, later):** a custom `egui::PaintCallback` with a two-pass Kawase/box blur `wgpu` shader sampling the canvas behind a floating panel. Only worth it for the floating toolbar/inspector panels — never the whole app background. Treat as a Phase 4 nice-to-have, not a blocker.

## Gate & component glyphs

Draw real IEC/ANSI-style symbols with `egui::Painter` path/line/bezier primitives — not rectangles with text labels:

- AND: flat-back D-shape (rounded front). OR/NOR: curved-back shield shape with pointed front. XOR/XNOR: OR shape plus an extra leading curved line. NOT: triangle + small bubble at the tip. Buffer: triangle, no bubble. NAND/NOR: their base gate shape + bubble at output.
- Bubbles (inversion) are always a small hollow circle, same visual weight regardless of which gate it's on.
- Input/output pins: short stub lines ending in a small filled/hollow dot depending on signal state (see signal tokens above). Consistent pin length across all components so wires connect at predictable offsets.
- Displays (LED, 7-seg, binary, hex): flat rounded-rect housings in `bg_panel_raised`, digits/LED rendered in `signal_low`/`signal_high` tokens.

## Typography

- One UI font (e.g. Inter or the default egui font is fine — don't add a decorative font). One monospace font for values/hex/binary displays and truth-table cells.
- Panel headers: small caps or medium-weight, `text_primary`, no all-caps shouting.

## Explicitly avoid

Emoji anywhere in the UI. Rainbow/multi-hue gradients. Global glow/bloom. "Powered by AI" or chat-style UI elements. Big empty hero/welcome screens. Dashboard-style stat cards. Overuse of shadows/blur making everything look "floaty."

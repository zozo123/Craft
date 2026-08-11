# Craft → Rust: end-to-end demo

![Craft (Rust) networked demo](./demo.gif)

*(full-quality [`demo.mp4`](./demo.mp4) — 960×540, H.264)*

This clip is rendered **headlessly, entirely in Rust**, driving the whole stack
end to end — no window, no display, no C. It was recorded on a fresh Linux VM
using a software Vulkan device (Mesa **lavapipe**), the same path CI uses.

## What you're seeing

An orbit camera circles a live, server-authoritative chunk while a day/night
cycle animates the sky. A second networked "bot" client walks in a circle and
**builds a tower** — every block you see appear is a real `Block` packet routed
through the server, persisted in SQLite, broadcast back, applied to the
recorder's world, and re-meshed with ambient occlusion. The white crosshair is
the HUD.

## The pipeline (all Rust)

```
craft-server (Tokio TCP)  ──▶  craft-protocol (line packets)  ──▶  recorder client
      ▲  persist (craft-db / SQLite)                                    │
      │                                                                 ▼
   bot client ── Block/Position packets ──▶ server ── broadcast ──▶ OnlineWorld
                                                                        │ mesh_map + AO (craft-core)
                                                                        ▼
                                                         wgpu render-to-texture (WGSL)
                                                                        │ copy_texture_to_buffer
                                                                        ▼
                                                            PNG frames ──▶ ffmpeg ──▶ mp4 + gif
```

Crates exercised: `craft-core` (world/noise/mesh/AO/physics/matrix), `craft-protocol`,
`craft-db`, `craft-server`, `craft-client` (`src/demo.rs` recorder + `online.rs` engine).

## Reproduce locally

```bash
cd rust
cargo run -p craft-server -- 127.0.0.1:4080 &
cargo run -p craft-client -- --demo 127.0.0.1:4080 --frames 150 --out frames --size 960x540
ffmpeg -framerate 30 -i frames/frame_%05d.png -c:v libx264 -pix_fmt yuv420p -crf 24 demo.mp4
```

On a headless Linux box, install `mesa-vulkan-drivers ffmpeg` and set
`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json WGPU_BACKEND=vulkan`
first. The `rust-e2e` GitHub Actions workflow does exactly this and uploads the
`craft-rust-demo` artifact on every push/PR.

## Play the real thing

```bash
cargo run -p craft-server -- 0.0.0.0:4080
cargo run -p craft-client -- --connect 127.0.0.1:4080   # WASD + mouse, 1-9 hotbar
```

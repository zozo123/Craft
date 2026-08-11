# Craft → Rust — Reoptimized finish-line graph ($80)

See also [COST_TRACE.md](COST_TRACE.md) for the live dollar board.

```
spent ~$23.55 → after reopt bookkeeping ~$24 / $80
critical path: walkable wgpu client → HUD → net merge
parallel off-path: sqlite ∥ protocol → server
```

## Why reoptimize

| Old | New |
|---|---|
| Serial J5…J16 | Fan-out H∥I after physics |
| Perfect AO before playable | **Walkable flat-lit client first** |
| Nested agents in islo | Cursor write + islo snapshot gates only |
| $20 tranche stop | **$80 finish-line** with ~$13 reserve |

## Wave board

| Wave | Deliverable | Est $ | islo |
|---|---|---:|---|
| A–D | DONE core/mesh/physics | 23.55 | `craft-sim-v1` |
| **E** | `craft-client`: wgpu + textured chunk + WASD | 10 | bake `craft-client-v1` |
| **F** | crosshair/hotbar/daylight | 5 | gate |
| **G** | AO + workers (can be lite) | 6 | gate |
| **H** | `craft-db` rusqlite | 4 | fork ∥ I |
| **I** | `craft-protocol` packets | 4 | fork ∥ H |
| **J** | `craft-server` Tokio | 6 | after H+I |
| **K+L** | net client + rust-full-v0 e2e | 8 | fanout |
| reserve | | ≤13 | |

## Run loop (every wave)

```bash
# 1 write + local gate
cd rust && cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test --all
# 2 push public e2e
git push origin rust-rewrite   # GH Actions rust-core
# 3 islo fork gate (deterministic)
islo job deploy --path jobs/craft-gate/job.toml && islo job run craft-gate --watch
# 4 cost
python3 rust/tools/append_spend.py --job WAVE --kind cursor_write --usd N --notes "..."
python3 rust/tools/cost_report.py
```

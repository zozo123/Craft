# Craft → Rust — Finish-line graph ($80)

See [COST_TRACE.md](COST_TRACE.md) for the live dollar board.

```
spent $49.05 / $80   remain $30.95
critical path DONE: walkable client → db∥protocol → server → net e2e
deferred polish: F HUD, G AO+workers (reserve covers if needed)
```

## Why this order

| Old | Shipped |
|---|---|
| Serial everything | Fan-out H∥I after physics |
| Perfect AO before playable | Walkable flat-lit client first |
| Nested agents in islo | Cursor write + islo/GH gates |
| F/G before multiplayer | **Deferred** — hit `rust-full-v0` net e2e under budget |

## Wave board

| Wave | Deliverable | Est $ | Status |
|---|---|---:|---|
| A–D | core/mesh/physics | 23.55 | DONE |
| E | wgpu walkable + smoke | 9 | DONE |
| H∥I | db + protocol | 5 | DONE |
| J | Tokio server | 5 | DONE |
| K+L | net-smoke + CI e2e | 6 | DONE |
| F/G | HUD / AO | — | deferred |
| reserve | | ≤30.95 | |

## Run loop

```bash
cd rust && cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test --all
git push origin rust-rewrite
islo job deploy --path jobs/craft-gate/job.toml && islo job run craft-gate --watch
python3 rust/tools/cost_report.py
```

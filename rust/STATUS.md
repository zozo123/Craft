# Craft → Rust rewrite — STATUS & spend ledger

Append-only. Update after every job, gate, or significant Cursor turn burst.
Hard rule: **do not start a new job if projected cumulative spend would exceed the active cap.**

## Active budget

| Field | Value |
|---|---|
| Cap | **$20 USD** total for Tranche 1 (deterministic core + parity gates) |
| Execution model | Cursor writes Rust; islo jobs run build/test only (no nested agents) |
| Nested agents in jobs | **Blocked** — `api.cursor.com` unreachable from job sandboxes; phantom `CURSOR_API_KEY` |
| Cost model source | Plan COST MODEL (Sonnet-class prior) + measured islo agent spend |

---

## Story so far (what we actually did)

### 0. Scope decision
Forked [`fogleman/Craft`](https://github.com/fogleman/Craft) → [`zozo123/Craft`](https://github.com/zozo123/Craft). Full 1:1 Rust rewrite planned; graphics stack = **winit + wgpu + WGSL** (not OpenGL). Budget conflict ($20 asked vs ~$48 full parity) resolved by **tranches**: $20 = Tranche 1 hard-stop.

### 1. Substrate proven on islo (agent spend ≈ $0)
| Finding | Detail |
|---|---|
| Interactive `cursor-agent -p --force` | Works; returns sync output |
| Job-sandbox nested agent | **Fails** — `api.cursor.com → 000`, invalid phantom key |
| `claude` / `codex` in jobs | Unusable (no Anthropic key / OpenAI credits exhausted) |
| Decision | **You write Rust here; islo only builds/tests/snapshots** |

### 2. Golden oracle (zero agent spend)
Headless C dumpers under [`oracle/`](../oracle/) stub 4 GL typedefs + 3 GLFW keys so pure sources compile without GL/X11.

| Fixture | Lines | Cross-platform |
|---|---|---|
| `item.tsv` | 887 | **exact match** macOS ↔ Linux |
| `world_*.tsv` (6 chunks) | ~111k | **exact match** |
| `noise.tsv` | 639 | float drift ≤ ~1.5e-7 abs |
| `matrix.tsv` | 20 | near-zeros differ; use abs/rel tol |
| `cube.tsv` | 15 | float drift |

Parity rule: integers exact; floats use mixed tolerance (abs `1e-5` or rel `1e-5`, never bare ULP).

### 3. Snapshot `craft-base-v1` (238 MB)
Job [`jobs/craft-oracle`](../jobs/craft-oracle/job.toml) installed apt+Rust, cloned `rust-rewrite`, generated goldens, verified line counts, baked snapshot. Restore verified (~5s): goldens + `rustc 1.97.1` + source at `b03dce9`/`0183714`.

islo quirks discovered (documented in job commits):
- `timeout_secs` on steps rejected; run `timeout` must be integer seconds
- `{param}` not substituted in multi-line strings
- step `workdir` ignored → use `make -C`
- egress flaky → retry with backoff

### 4. `craft-core` complete + e2e parity gate (this machine)
Under [`rust/`](./):
- Workspace + `craft-core`: `config`, `item`, `noise`, `world`, `map`, `lib`
- Parity tests vs C goldens: `item` (exact), `noise` (tol 1e-5), `world` (exact, ordered)
- `map` behavioural unit tests (7)
- **Local e2e green**: `make -C oracle golden` -> `cargo fmt --check` + `clippy -D warnings` + `cargo test` all pass
- GitHub Actions [`rust-core.yml`](../.github/workflows/rust-core.yml): builds C oracle -> goldens -> Rust parity, on every push/PR to `rust-rewrite`

---

## Spend ledger (running)

Currency: **USD estimate**. Cursor does not expose exact per-chat billing here; estimates use the plan's Sonnet-class prior (~$0.04–0.08/turn for coding) and measured islo agent tokens (0).

| When | Job / activity | Agent? | Est. $ | Cumul. $ | Gate |
|---|---|---|---:|---:|---|
| plan phase | Full DAG / cost model / distribution plan | Cursor plan turns | ~8.00 | 8.00 | plan accepted |
| exec | islo verify (cursor-agent OK, toolchain) | 1 tiny probe | 0.05 | 8.05 | pass |
| exec | oracle C dumpers + Makefile | Cursor write | 1.50 | 9.55 | build+determinism pass |
| exec | craft-oracle job (v1–v7 iterate) | **none** | 0.00 | 9.55 | `craft-base-v1` ready |
| exec | craft-diag (auth probe) | **none** | 0.00 | 9.55 | documented blocker |
| exec | craft-core start (config/item/noise/world) | Cursor write | 2.00 | 11.55 | not gated yet |
| exec | craft-core finish (map/lib/tests + GH CI) | Cursor write | 3.00 | **14.55** | local fmt+clippy+test green |
| — | **Remaining under $20 cap** | | | **~5.45** | |

### Remaining budget allocation (proposed)

| Next | Est. $ | Cumul after | Must ship |
|---|---:|---:|---|
| Finish map + lib + item/map unit tests | 1.50 | 13.05 | compile |
| Noise+world golden parity tests (Linux goldens) | 2.00 | 15.05 | gate G-core |
| islo gate job on `craft-base-v1` | 0.00 agent | 15.05 | `cargo test` green |
| Contingency / retries | ≤4.95 | ≤20.00 | stop |

**Stop conditions**
1. Cumulative estimate ≥ **$20** → stop coding, only document.
2. Any parity gate fails twice → pause and ask before more spend.
3. Do not start wgpu/client (Tranche 2) under this $20 cap.

---

## Commits on `rust-rewrite`

| SHA | Summary |
|---|---|
| `b03dce9` | Headless golden-fixture oracle |
| `0183714` | craft-oracle job → craft-base-v1 |

---

## What $20 buys under the chosen model

Because nested agents are out, **your Cursor turns are the only model cost**. That stretches Tranche 1: deterministic core (noise, worldgen, map, item) + golden gates should fit under $20 with ~$5–8 headroom if we stay focused and skip client/shaders.

Full parity (wgpu client, DB, protocol, server) remains **~$48** prior — out of scope for this cap.

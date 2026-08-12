# Launch YC–style post

**Copy-paste ready.** Short version for X/Twitter at the bottom.

---

## Title options

1. **Craft → Rust in $111 under a software factory (islo gates, $0-token e2e)**
2. **We rewrote Minecraft-clone Craft in Rust with a DAG budget and sandbox factory**
3. **Show HN: All-Rust Craft + the SW factory that shipped it under $120**

---

## Post body (Launch YC / Show HN)

**TL;DR:** We rewrote [fogleman/Craft](https://github.com/fogleman/Craft) end-to-end in Rust (wgpu client + Tokio server), shipped a headless lavapipe demo, and ran the whole thing as a **software factory**: Cursor writes code, [islo.dev](https://islo.dev) + GitHub Actions run deterministic gates at **$0 agent tokens**, hard-capped at **$120**.

**Live**
- Demo + invoice + DAG: https://zozo123.github.io/Craft/
- Fork: https://github.com/zozo123/Craft
- Factory lessons: https://zozo123.github.io/Craft/#factory

### The problem

Agent coding is great at *writing* — terrible as an unmetered factory. Without a DAG, a hard budget, and gates that don’t burn tokens, you get infinite loops of “almost green” and no finish line.

### What we built

1. **Product** — Craft in Rust: walkable wgpu client, AO meshing, HUD/hotbar, Tokio multiplayer, SQLite persistence, C-oracle bit parity.
2. **Factory** — Token-primary ledger (`$0.02/1k`), wave board, islo snapshot ladder (`craft-base-v1` → `craft-full-v2`), public `rust-e2e` CI (parity + lavapipe demo artifact), GitHub Pages invoice.
3. **Proof** — Headless networked demo (software Vulkan) recorded in an islo VM and committed to the repo.

### How the factory works

```
Cursor (writes, costs tokens)
    ↓
islo sandboxes + GH Actions (compile / test / bake / record = $0 tokens)
    ↓
snapshot gates → craft-full-v2 → Pages invoice
```

Nested agents *inside* sandboxes failed (API unreachable) — that became a hard rule: **agents write, floors gate**.

[Incredibuild](https://www.incredibuild.com) sits in the same SW-factory ecosystem (distributed builds); this run’s critical path was islo + GH for Cargo e2e.

### Traction / numbers

| | |
|---|---|
| Cap | $120 |
| Spent | **$111.25** (93%) |
| Tokens | ~3.11M |
| Gates | islo + GH = **$0** agent tokens |
| PRs merged to fork master | #1–#5 |
| Snapshots | 7 (`base` → `full-v2`) |

### What worked / less so

**Worked:** DAG + budget ledger, parallel islo signals, C goldens, writer/gate split, lavapipe demo, shipping playable+net before polish.

**Less:** nested VM agents, rustc/clippy skew across runners, silent `set -e` failures, snapshot name collisions, latent WGSL uniform stride until offscreen render.

Full writeup: https://zozo123.github.io/Craft/#factory

### Ask

- Feedback on the factory model (DAG + $0-token gates + hard cap)
- Teams who want the same pattern for other C→Rust / port factories
- Eyes on the live demo / parity approach

We’re not selling Craft — it’s the **worked example**. The product is the factory discipline: budget, graph, sandboxes, e2e or it didn’t happen.

---

## Short (X / LinkedIn)

We rewrote Craft (Minecraft clone) in Rust under a $120 hard cap — as a software factory.

Cursor writes. islo.dev + GitHub Actions gate at $0 tokens. Lavapipe e2e demo + invoice live:

https://zozo123.github.io/Craft/

$111.25 spent · DAG · snapshots craft-base→full-v2 · what worked / less: https://zozo123.github.io/Craft/#factory

Fork: https://github.com/zozo123/Craft

---

## Credits

Original Craft: [Michael Fogleman](https://www.michaelfogleman.com/) / [fogleman/Craft](https://github.com/fogleman/Craft).  
Factory floor: [islo.dev](https://islo.dev). Build ecosystem: [Incredibuild](https://www.incredibuild.com).  
Rewrite fork: [zozo123/Craft](https://github.com/zozo123/Craft).

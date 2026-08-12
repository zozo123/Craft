# Launch post (YC / Show HN)

Concise, copy-paste ready. Plain text — HN doesn't render markdown.

---

## Title

**Show HN: I rewrote Fogleman's Craft in Rust under a $120 budgeted "software factory"**

---

## Body

```
I rewrote the game part of Michael Fogleman's Craft (a small Minecraft clone in
C + OpenGL with a Python server) in Rust: winit + wgpu (WGSL) client, Tokio TCP
server, SQLite persistence, same line-based multiplayer protocol.

Honesty first: this is NOT an all-Rust repo. GitHub shows ~45% Rust / ~43% C /
~10% Python. The Rust product is in rust/. The C stays on purpose — the original
client as reference, plus a small C "oracle" that generates golden fixtures so
the Rust port is bit-checked against the original noise/mesh/matrix output in CI.

The part I want feedback on is the process. I ran it as a software factory:

- A DAG of small units, each with one deterministic gate.
- A token-primary ledger, hard-capped at $120. Only agent coding turns cost
  money; every gate is free.
- islo.dev sandboxes + GitHub Actions do all compile/test/snapshot/record work
  at $0 agent tokens. The agent only writes code.
- A snapshot ladder (craft-base-v1 -> craft-full-v2) so e2e restores instead of
  rebuilding the world every run.

The demo is rendered headlessly on a Linux VM with software Vulkan (Mesa
lavapipe): a live server plus a bot that walks and builds a tower while an orbit
camera records frames, encoded to mp4/gif. CI runs the same path and uploads the
artifact on every push.

Worked: the DAG + cap forced a real finish line; C goldens caught port drift;
parallel sandboxes made retries cheap; agents stayed on the write side of the
line.

Didn't: toolchain skew (rust-1.97 clippy flagged an unwrap that local 1.92 and
GH stable missed); silent `set -e` failures with no logs; a snapshot name
collision on rebake; and a latent wgpu uniform-stride bug (WGSL 112B vs Rust
96B) that never hit CI until I added offscreen rendering.

Total: ~$113 of $120, ~3.2M tokens, parity + headless demo green.

Demo, invoice and DAG: https://zozo123.github.io/Craft/
What worked / less:    https://zozo123.github.io/Craft/#factory
Code:                  https://github.com/zozo123/Craft
Original:              https://github.com/fogleman/Craft

Craft is just the worked example — I'm curious whether budgeting agent work this
way (cap + ledger + free deterministic gates) holds up for other C->Rust ports.
```

---

## First comment (post immediately after)

```
Caveats:

- "Rewrite" means the playable product (rust/). I deliberately kept the C: the
  oracle is ground truth for parity, so it earns its place. That plus vendored
  deps/ is why the language bar is ~43% C.
- The budget is self-reported token estimates, not a vendor invoice. The point
  is the discipline (cap + ledger + free gates), not the exact dollar.
- Demo lighting is the original Craft shader ported to WGSL, so it's faithfully
  dim in places. AO follows the 0fps.wordpress.com method.

Happy to go deep on the islo snapshot/gate setup or the wgpu offscreen recorder.
```

---

## Short (X / LinkedIn)

```
Rewrote Craft (Minecraft clone) in Rust under a $120 hard cap — run as a
software factory: the agent writes, islo.dev + GitHub Actions gate at $0 tokens.

Headless lavapipe e2e demo + live invoice:
https://zozo123.github.io/Craft/
```

---

## Credits

Original Craft: [Michael Fogleman](https://www.michaelfogleman.com/) / [fogleman/Craft](https://github.com/fogleman/Craft).
Factory floor: [islo.dev](https://islo.dev). Build ecosystem: [Incredibuild](https://www.incredibuild.com).
Rewrite fork: [zozo123/Craft](https://github.com/zozo123/Craft).

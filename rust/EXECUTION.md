# Craft → Rust — Finish-line execution graph ($80 cap)

```
spent ~$14.55 / cap $80.00   remaining ≈ $65.45
target: rust-full-v0 (full C/Python parity)
model: Cursor writes code; islo forkable microVMs + GitHub Actions run gates
```

Nested agents still **cannot** auth inside job sandboxes. islo value = **snapshot forks**,
deterministic gates, parallel crate-isolated VMs, rollback on fail.

---

## Optimal critical path (waves)

```mermaid
flowchart TB
  subgraph done [DONE]
    J0[J0_Oracle]
    Core[Core_item_map_noise_world]
    CI0[CI_green]
  end

  subgraph t1 [T1_pure ~11usd]
    B[B_matrix_cube]
    C[C_sim_mesh]
    D[D_physics]
  end

  subgraph t2 [T2_client ~26usd]
    E[E_wgpu_pipelines]
    F[F_HUD_local]
    G[G_AO_workers]
  end

  subgraph t3 [T3_net ~26usd]
    H[H_sqlite]
    I[I_protocol]
    J[J_server]
    K[K_net_client]
    L[L_full_e2e]
  end

  J0 --> Core --> CI0 --> B --> C --> D
  B --> E
  D --> F
  E --> F --> G
  Core --> H
  Core --> I
  H --> J
  I --> J
  G --> K
  J --> K --> L
```

| Wave | Ships | Est. $ | Cumul | islo use |
|---|---|---:|---:|---|
| A done | oracle + core + CI | 14.55 | 14.55 | `craft-base-v1` |
| **B** | matrix + cube parity | 3.50 | 18 | fork base → gate → `craft-mesh-v1` |
| **C** | craft-sim mesh stats | 3.00 | 21 | fork mesh |
| **D** | physics headless | 4.00 | 25 | fork mesh |
| **E** | wgpu block/sky/line/text | 12.00 | 37 | fork; GPU may be limited — GH + local |
| **F** | HUD / local client | 6.00 | 43 | smoke gate |
| **G** | AO + chunk workers | 8.00 | 51 | fork |
| **H** | SQLite + signs | 5.00 | 56 | fork `∥ I` |
| **I** | protocol 14 packets | 5.00 | 61 | fork `∥ H` |
| **J** | Tokio server | 8.00 | 69 | fork after H+I |
| **K+L** | net client + full e2e | 8.00 | 77 | multi-sandbox fanout |
| reserve | retries | ≤3 | ≤80 | — |

**Parallel safe:** H ∥ I (different crates). Everything else serial on critical path.

---

## How islo runs the graph

```mermaid
flowchart LR
  Base[craft_base_v1] -->|fork| GateB[cargo_test_mesh]
  GateB -->|bake| Mesh[craft_mesh_v1]
  Mesh -->|fork| GateC[craft_sim]
  Mesh -->|fork parallel| GateH[sqlite]
  Mesh -->|fork parallel| GateI[protocol]
  GateH --> Srv[craft_server_v1]
  GateI --> Srv
  Srv -->|fanout 3 clients| E2E[convergence_gate]
```

Per wave:
1. Cursor lands code on `rust-rewrite`
2. `islo job run` with `snapshot_name` = last green snap
3. Job: `git pull` → `cargo test` / binary gate (no agent)
4. On green: `snapshot` next name
5. On fail: discard VM, fix, retry ≤2
6. `append_spend.py` after each wave

GitHub Actions remains the public e2e oracle for pure crates (same-runner goldens).

---

## Finish definition (`rust-full-v0`)

1. Local client plays: walk, place/break, chat UI, day/night sky  
2. Server + 3 clients converge on blocks/signs  
3. SQLite roundtrip matches C schema  
4. All golden parity tests green on CI  
5. Spend ledger ≤ $80  

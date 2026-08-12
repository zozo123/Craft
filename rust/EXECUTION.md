# Craft → Rust — End graph (product e2e v2)

Cap **$120** token-primary. See [COST_TRACE.md](COST_TRACE.md).

```mermaid
flowchart TD
  A[A_oracle] --> B[B_matrix_cube]
  B --> C[C_mesh]
  C --> D[D_physics]
  D --> E[E_client]
  D --> H[H_db]
  D --> I[I_proto]
  H --> J[J_server]
  I --> J
  C --> G[G_AO]
  E --> G
  G --> S[S_full_stream]
  J --> S
  E --> F[F_HUD_daylight]
  S --> K[K_connect_peers]
  F --> K
  K --> L[L_craft_full_v2]
```

## Cleared

| Node | Gate |
|---|---|
| A–L1 | `craft-full-v1` |
| S full chunk stream | e2e_net / online_e2e / `--net-play` |
| F HUD + daylight + hotbar 1–9 | interactive `--connect` |
| P peer markers | drawn in `--connect` |
| L2 | GH + islo → **`craft-full-v2`** |
| Demo + CI | `--demo` + lavapipe artifact + PR#1 merged |
| README finish | Rust-first README + docs demo on `master` |

## Run

```bash
cd rust
cargo run -p craft-server -- 0.0.0.0:4080
cargo run -p craft-client -- --connect 127.0.0.1:4080
cargo run -p craft-client -- --net-play 127.0.0.1:4080
islo job deploy --path jobs/craft-mp-e2e/job.toml && islo job run craft-mp-e2e --watch
```

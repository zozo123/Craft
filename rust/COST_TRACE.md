# Live cost trace — take it to the end ($120)

```
spent $68.05 → +stream/HUD/peers (pending append) / $120
```

## End DAG

```mermaid
flowchart TD
  done[A_to_full_v1] --> S[S_full_stream]
  done --> F[F_HUD]
  done --> P[P_peers]
  S --> L2[L2_craft_full_v2]
  F --> L2
  P --> L2
```

| Package | Status |
|---|---|
| Full chunk stream + DB overlay | DONE (local e2e green) |
| HUD crosshair + daylight + hotbar | DONE |
| Peer markers | DONE |
| islo/GH `craft-full-v2` | IN FLIGHT |

```bash
python3 rust/tools/cost_report.py
```

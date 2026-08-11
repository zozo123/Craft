#!/usr/bin/env python3
"""Append a spend-ledger entry. Token-primary under $120 cap.

Usage:
  python3 rust/tools/append_spend.py \\
    --job wave-G --kind cursor_write --tokens 180000 --notes "AO mesh"

USD is derived: tokens * usd_per_1k_tokens / 1000 (from ledger pricing_prior).
Pass --usd only to override derivation. Exits 2 if projected spend exceeds cap.
"""
from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

LEDGER = Path(__file__).with_name("spend-ledger.json")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--job", required=True)
    ap.add_argument("--kind", required=True)
    ap.add_argument("--tokens", type=int, default=None, help="total tokens (in+out)")
    ap.add_argument("--tokens-in", type=int, default=None)
    ap.add_argument("--tokens-out", type=int, default=None)
    ap.add_argument("--usd", type=float, default=None, help="override derived USD")
    ap.add_argument("--attempt", type=int, default=1)
    ap.add_argument("--notes", default="")
    ap.add_argument("--force", action="store_true")
    args = ap.parse_args()

    data = json.loads(LEDGER.read_text())
    cap = float(data["cap"])
    cumul = float(data["cumulative_est_usd"])
    prior = data.get("pricing_prior", {})
    usd_per_1k = float(prior.get("usd_per_1k_tokens", 0.02))

    tokens = args.tokens
    tin = args.tokens_in
    tout = args.tokens_out
    if tokens is None and tin is not None and tout is not None:
        tokens = tin + tout
    if tokens is None and args.usd is None:
        print("need --tokens or --tokens-in/--tokens-out or --usd", file=sys.stderr)
        return 2

    if args.usd is not None:
        usd = args.usd
    else:
        usd = round((tokens or 0) * usd_per_1k / 1000.0, 4)

    projected = cumul + usd
    if projected > cap and data.get("hard_stop", True) and not args.force:
        print(
            f"REFUSED: {cumul:.2f} + {usd:.2f} = {projected:.2f} exceeds cap {cap:.2f}",
            file=sys.stderr,
        )
        return 2

    agent_tokens = None
    if tokens is not None or tin is not None:
        agent_tokens = {
            "in": tin if tin is not None else (tokens or 0),
            "out": tout if tout is not None else 0,
            "total": tokens if tokens is not None else (tin or 0) + (tout or 0),
        }

    entry = {
        "id": f"{args.job}-{len(data['entries']) + 1}",
        "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "job": args.job,
        "attempt": args.attempt,
        "kind": args.kind,
        "est_usd": usd,
        "agent_tokens": agent_tokens,
        "notes": args.notes,
    }
    data["entries"].append(entry)
    data["cumulative_est_usd"] = round(projected, 2)
    data["remaining_usd"] = round(cap - projected, 2)
    data["cumulative_tokens"] = int(data.get("cumulative_tokens", 0)) + int(
        (agent_tokens or {}).get("total", 0)
    )
    LEDGER.write_text(json.dumps(data, indent=2) + "\n")
    print(
        f"ok +{agent_tokens and agent_tokens.get('total', 0) or 0} tok  "
        f"+${usd:.4f}  cumul=${data['cumulative_est_usd']:.2f}  "
        f"remain=${data['remaining_usd']:.2f}  tok_cumul={data['cumulative_tokens']}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

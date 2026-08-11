#!/usr/bin/env python3
"""Append a spend-ledger entry and print remaining budget.

Usage:
  python3 rust/tools/append_spend.py \\
    --job craft-core --kind cursor_write --usd 1.5 --notes "map+tests"

Exits 2 if adding this entry would exceed the hard cap.
"""
from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

LEDGER = Path(__file__).with_name("spend-ledger.json")
STATUS = Path(__file__).resolve().parents[1] / "STATUS.md"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--job", required=True)
    ap.add_argument("--kind", required=True)
    ap.add_argument("--usd", type=float, required=True)
    ap.add_argument("--attempt", type=int, default=1)
    ap.add_argument("--notes", default="")
    ap.add_argument("--force", action="store_true", help="allow exceeding cap")
    args = ap.parse_args()

    data = json.loads(LEDGER.read_text())
    cap = float(data["cap"])
    cumul = float(data["cumulative_est_usd"])
    projected = cumul + args.usd

    if projected > cap and data.get("hard_stop", True) and not args.force:
        print(
            f"REFUSED: {cumul:.2f} + {args.usd:.2f} = {projected:.2f} exceeds cap {cap:.2f}",
            file=sys.stderr,
        )
        return 2

    entry = {
        "id": f"{args.job}-{len(data['entries']) + 1}",
        "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "job": args.job,
        "attempt": args.attempt,
        "kind": args.kind,
        "est_usd": args.usd,
        "agent_tokens": None,
        "notes": args.notes,
    }
    data["entries"].append(entry)
    data["cumulative_est_usd"] = round(projected, 2)
    data["remaining_usd"] = round(cap - projected, 2)
    LEDGER.write_text(json.dumps(data, indent=2) + "\n")
    print(
        f"ok +${args.usd:.2f}  cumul=${data['cumulative_est_usd']:.2f}  "
        f"remaining=${data['remaining_usd']:.2f}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

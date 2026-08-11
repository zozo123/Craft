#!/usr/bin/env python3
"""Print spend-ledger summary for the finish-line run."""
from __future__ import annotations

import json
from pathlib import Path

LEDGER = Path(__file__).with_name("spend-ledger.json")


def main() -> None:
    d = json.loads(LEDGER.read_text())
    spent = float(d["cumulative_est_usd"])
    cap = float(d["cap"])
    remain = float(d.get("remaining_usd", cap - spent))
    print(f"cap=${cap:.2f}  spent=${spent:.2f}  remain=${remain:.2f}  ({100*spent/cap:.0f}% used)")
    print("-" * 72)
    print(f"{'id':<28} {'kind':<22} {'usd':>7}  notes")
    for e in d["entries"]:
        notes = (e.get("notes") or "")[:40]
        print(f"{e['id']:<28} {e['kind']:<22} {e['est_usd']:>7.2f}  {notes}")
    print("-" * 72)
    if d.get("next_allocation"):
        print("next allocation:")
        for n in d["next_allocation"]:
            wave = n.get("wave", "?")
            print(f"  [{wave}] {n.get('job','?'):<24} ${n.get('est_usd',0):.2f}")


if __name__ == "__main__":
    main()

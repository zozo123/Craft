#!/usr/bin/env python3
"""Print spend-ledger summary (token-primary)."""
from __future__ import annotations
import json
from pathlib import Path
LEDGER = Path(__file__).with_name("spend-ledger.json")

def main() -> None:
    d = json.loads(LEDGER.read_text())
    spent = float(d["cumulative_est_usd"])
    cap = float(d["cap"])
    remain = float(d.get("remaining_usd", cap - spent))
    toks = int(d.get("cumulative_tokens", 0))
    print(f"cap=${cap:.2f}  spent=${spent:.2f}  remain=${remain:.2f}  ({100*spent/cap:.0f}% used)  tokens={toks}")
    print("-" * 78)
    print(f"{'id':<28} {'kind':<22} {'tok':>8} {'usd':>7}  notes")
    for e in d["entries"]:
        notes = (e.get("notes") or "")[:36]
        at = e.get("agent_tokens") or {}
        tok = at.get("total", at.get("in", 0) or 0)
        if isinstance(at, dict) and "out" in at and "total" not in at:
            tok = (at.get("in") or 0) + (at.get("out") or 0)
        print(f"{e['id']:<28} {e['kind']:<22} {tok:>8} {e['est_usd']:>7.2f}  {notes}")
    print("-" * 78)
    if d.get("next_allocation"):
        print("next allocation:")
        for n in d["next_allocation"]:
            print(f"  [{n.get('wave','?')}] {n.get('job','?'):<24} ${n.get('est_usd',0):.2f}  ~{n.get('est_tokens',0)} tok")

if __name__ == "__main__":
    main()

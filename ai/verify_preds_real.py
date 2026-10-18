# -*- coding: utf-8 -*-
"""Реальная проверка предсказаний через mavc build+run (не эвристика).

    python ai/verify_preds_real.py --preds ai/preds_haiku45.jsonl

Использует CASES из verify_hard.py. Пишет предсказание в source/main.mavl,
собирает реальным mavc, запускает exe, сравнивает вывод.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from verify_hard import CASES, EXE, run, ROOT, SRC

HARD = os.path.join(ROOT, "ai", "hard")


def load_lookup():
    lookup = {}
    for fn in sorted(os.listdir(HARD)):
        if fn.endswith(".mavl"):
            lookup[fn[3:-5]] = fn
    return lookup


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--preds", required=True)
    a = ap.parse_args()
    preds = {}
    with open(os.path.join(ROOT, a.preds), encoding="utf-8") as f:
        for line in f:
            if line.strip():
                r = json.loads(line)
                preds[r["id"]] = r.get("output", "")
    with open(SRC, encoding="utf-8") as f:
        saved = f.read()
    lookup = load_lookup()
    ok = fail = skip = 0
    try:
        for c in CASES:
            tag = c.get("tag", "")
            name = c["id"] + ("[" + tag + "]" if tag else "")
            out = preds.get(c["id"])
            if out is None:
                print(f"[--] {name:22} no pred")
                skip += 1
                continue
            mem = c["mem"]
            # ptroffset требует unsafe => mem=manual
            uses_unsafe = "unsafe" in out
            if uses_unsafe and mem != "manual":
                mem = "manual"
            for wf, wt in c.get("write", {}).items():
                with open(os.path.join(ROOT, wf), "w", encoding="utf-8") as f:
                    f.write(wt)
            with open(SRC, "w", encoding="utf-8") as f:
                f.write(out)
            b = run(["mavc", "build", "--mem=" + mem])
            if b.returncode != 0:
                lines = [ln.strip() for ln in b.stdout.strip().splitlines() if "error" in ln.lower()]
                tail = " | ".join(ln[:110] for ln in lines[:3]) or "no-error-lines"
                print(f"[XX] {name:22} BUILD FAIL  {tail}")
                fail += 1
                continue
            exe = EXE + (".exe" if os.name == "nt" else "")
            r = run([exe] + c.get("run_args", []), input=c.get("stdin"))
            got = r.stdout.replace("\r\n", "\n") if r.stdout else ""
            if r.returncode == 0 and got == c["expected"]:
                print(f"[OK] {name:22} PASS")
                ok += 1
            else:
                print(f"[XX] {name:22} rc={r.returncode} got={got!r} want={c['expected']!r}")
                fail += 1
    finally:
        with open(SRC, "w", encoding="utf-8") as f:
            f.write(saved)
        for junk in ("hard_w.txt", "arg_h.txt", "app.log", "hw.txt", "f2.txt"):
            p = os.path.join(ROOT, junk)
            if os.path.exists(p):
                os.remove(p)
        run(["mavc", "build"])
    print(f"--- real mavc ---")
    print(f"ok {ok} / fail {fail}" + (f" / skip {skip}" if skip else ""))
    return fail == 0 and skip == 0


if __name__ == "__main__":
    sys.exit(0 if main() else 1)

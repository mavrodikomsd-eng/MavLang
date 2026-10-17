"""Прогон хард-референсов через НАСТОЯЩИЙ mavc (build+run, не эвристика).

    python ai/verify_hard.py   # из корня C:\\1\\аап, нужен mavc в PATH
"""
import os
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "source", "main.mavl")
HARD = os.path.join(ROOT, "ai", "hard")

FIZZ = "\n".join(["1", "2", "fizz", "4", "buzz", "fizz", "7", "8", "fizz",
                  "buzz", "11", "fizz", "13", "14", "fizzbuzz"]) + "\n"

CASES = [
    {"id": "fizzbuzz", "mem": "borrow", "expected": FIZZ},
    {"id": "csvsum", "mem": "borrow", "expected": "60\n"},
    {"id": "wordcount", "mem": "borrow", "expected": "3\nHELLO\n"},
    {"id": "fileroundtrip", "mem": "borrow", "expected": "50\n"},
    {"id": "argsdefault", "mem": "borrow", "expected": "none\n"},
    {"id": "argsdefault", "mem": "borrow", "run_args": ["yo"], "expected": "yo\n",
     "tag": "with-arg"},
    {"id": "counter", "mem": "borrow", "expected": "5\n"},
    {"id": "fib", "mem": "borrow", "expected": "55\n"},
    {"id": "normalize", "mem": "borrow", "expected": "rust lang\n"},
    {"id": "borrowseries", "mem": "borrow", "expected": "1\n2\n"},
    {"id": "ptroffset", "mem": "manual", "expected": "99\n"},
    {"id": "inputmath", "mem": "borrow", "stdin": "21\n", "expected": "n? 42\n"},
    {"id": "repconcat", "mem": "borrow", "expected": "ababab\n"},
    {"id": "argfile", "mem": "borrow", "expected": "usage\n"},
    {"id": "argfile", "mem": "borrow", "run_args": ["arg_h.txt"],
     "write": {"arg_h.txt": "hello"}, "expected": "hello\n", "tag": "with-arg"},
    {"id": "linesjoin", "mem": "borrow", "expected": "3\na-b-c\n"},
    {"id": "containscheck", "mem": "borrow", "expected": "found\nok\n"},
    {"id": "appendlog", "mem": "borrow", "expected": "3\n"},
]

EXE = os.path.join(ROOT, "target", "debug", "mavapp")


def run(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT, **kw)


def main():
    with open(SRC, encoding="utf-8") as f:
        saved = f.read()
    ok = fail = 0
    # реальные имена файлов 01_xxx.mavl
    files = sorted(os.listdir(HARD))
    lookup = {}
    for fn in files:
        if fn.endswith(".mavl"):
            lookup[fn[3:-5]] = fn
    try:
        for c in CASES:
            tag = c.get("tag", "")
            name = c["id"] + ("[" + tag + "]" if tag else "")
            fn = lookup[c["id"]]
            shutil.copy(os.path.join(HARD, fn), SRC)
            for wf, wt in c.get("write", {}).items():
                with open(os.path.join(ROOT, wf), "w", encoding="utf-8") as f:
                    f.write(wt)
            b = run(["mavc", "build", "--mem=" + c["mem"]])
            if b.returncode != 0:
                print(f"[XX] {name:22} BUILD FAIL\n{b.stdout[-800:]}")
                fail += 1
                continue
            exe = EXE + (".exe" if os.name == "nt" else "")
            r = run([exe] + c.get("run_args", []),
                    input=c.get("stdin"))
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
    print(f"hard real: {ok} ok / {fail} fail")
    return fail == 0


if __name__ == "__main__":
    sys.exit(0 if main() else 1)

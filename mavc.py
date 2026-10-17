#!/usr/bin/env python3
"""MavLang compiler v0.1 — транспилер .mavl -> Rust (intRs) -> rustc/cargo.

Раскладка:
  source/*.mavl   исходники
  intRs/          вся раст-кухня (Cargo.toml, src/*.rs)
  target/debug|release  чистые exe/dll для пользователя
"""
import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE_DIR = ROOT / "source"
INTRS_DIR = ROOT / "intRs"
TARGET_DIR = ROOT / "target"

HEADER = {
    "borrow": "// MavLang mem=borrow | transpiled to Rust (safe borrows)\n",
    "arena": "// MavLang mem=arena | TODO: bump allocator wrapper\n",
    "manual": "// MavLang mem=manual | unsafe allowed\n",
}


def find_entry() -> Path:
    cands = sorted(SOURCE_DIR.glob("*.mavl"))
    if not cands:
        sys.exit("source/*.mavl не найден. Положи main.mavl в source/")
    # main.mavl приоритетно
    for p in cands:
        if p.name == "main.mavl":
            return p
    return cands[0]


def parse_and_transpile(src: str, mem: str) -> str:
    if "fn main" not in src:
        raise SyntaxError("нужен fn main() {...} в .mavl")
    # базовая проверка баланса
    if src.count("{") != src.count("}"):
        raise SyntaxError("дисбаланс { } в .mavl")
    out_lines = []

    def _print_repl(m):
        arg = m.group(1).strip()
        if arg.startswith('"') or arg.startswith("'"):
            return f'println!({arg})'
        return f'println!("{{}}", {arg})'

    for i, line in enumerate(src.splitlines(), start=1):
        if "println!" not in line:
            # print(x) -> println!("{}", x), работает и внутри строки
            line = re.sub(r"(?<!:)print\((.+?)\)", _print_repl, line)
        out_lines.append(line)
    return HEADER[mem] + "\n".join(out_lines) + "\n"


def map_error(err: str, header_lines: int, mavl_name: str) -> str:
    # cargo пишет intRs/src/main.rs:12 -> маппим обратно в source/*.mavl
    def repl(m):
        rs_line = int(m.group(1))
        mavl_line = rs_line - header_lines
        if mavl_line < 1:
            mavl_line = 1
        return f"source/{mavl_name}:{mavl_line}"
    err = re.sub(r"intRs[/\\]src[/\\]main\.rs:(\d+)", repl, err)
    err = re.sub(r"src[/\\]main\.rs:(\d+)", repl, err)
    return err


def ensure_cargo_project(rs_code: str):
    (INTRS_DIR / "src").mkdir(parents=True, exist_ok=True)
    cargo_toml = INTRS_DIR / "Cargo.toml"
    if not cargo_toml.exists():
        cargo_toml.write_text(
            '[package]\nname = "mavapp"\nversion = "0.1.0"\nedition = "2021"\n',
            encoding="utf-8",
        )
    (INTRS_DIR / "src" / "main.rs").write_text(rs_code, encoding="utf-8")
    return HEADER_LINES


HEADER_LINES = 1


def do_build(release: bool, mem: str) -> int:
    entry = find_entry()
    src = entry.read_text(encoding="utf-8")
    try:
        rs_code = parse_and_transpile(src, mem)
    except SyntaxError as e:
        print(f"MavLang error in source/{entry.name}: {e}")
        return 1
    ensure_cargo_project(rs_code)
    cmd = ["cargo", "build", "--manifest-path", str(INTRS_DIR / "Cargo.toml")]
    if release:
        cmd.append("--release")
    # тихий билд, чтобы выглядело как свой компилятор
    p = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    if p.returncode != 0:
        print(f"[mavc] ошибка компиляции MavLang (mem={mem}):")
        print(map_error(p.stderr, HEADER_LINES, entry.name))
        return p.returncode
    profile = "release" if release else "debug"
    src_bin_dir = INTRS_DIR / "target" / profile
    dst_bin_dir = TARGET_DIR / profile
    dst_bin_dir.mkdir(parents=True, exist_ok=True)
    copied = []
    for ext in (".exe", ".dll", ".pdb", ""):
        for f in src_bin_dir.glob(f"mavapp{ext}"):
            shutil.copy2(f, dst_bin_dir / f.name)
            copied.append(f.name)
    # linux/mac без расширения тоже подхватим
    if not copied:
        for f in src_bin_dir.glob("mavapp*"):
            if f.is_file() and f.suffix not in (".d", ".rlib", ".rmeta"):
                shutil.copy2(f, dst_bin_dir / f.name)
                copied.append(f.name)
    print(f"[mavc] ok ({mem}, {profile}): {', '.join(copied) if copied else 'собрано'} -> target/{profile}/")
    return 0


def do_run(release: bool, mem: str) -> int:
    rc = do_build(release, mem)
    if rc != 0:
        return rc
    profile = "release" if release else "debug"
    exe = TARGET_DIR / profile / "mavapp.exe"
    if not exe.exists():
        # non-windows fallback
        exe = TARGET_DIR / profile / "mavapp"
    p = subprocess.run([str(exe)])
    return p.returncode


def do_clean() -> int:
    for d in (INTRS_DIR / "target", TARGET_DIR / "debug", TARGET_DIR / "release"):
        if d.exists():
            shutil.rmtree(d)
    print("[mavc] cleaned")
    return 0


def main():
    ap = argparse.ArgumentParser(prog="mavc", description="MavLang compiler v0.1")
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("build", "run", "check"):
        s = sub.add_parser(name)
        s.add_argument("--release", action="store_true")
        s.add_argument("--mem", choices=["borrow", "arena", "manual"], default="borrow")
    sub.add_parser("clean")
    a = ap.parse_args()
    if a.cmd == "clean":
        sys.exit(do_clean())
    if a.cmd == "check":
        entry = find_entry()
        try:
            parse_and_transpile(entry.read_text(encoding="utf-8"), a.mem)
            print(f"[mavc] check ok: source/{entry.name}")
            sys.exit(0)
        except SyntaxError as e:
            print(f"MavLang error in source/{entry.name}: {e}")
            sys.exit(1)
    if a.cmd == "build":
        sys.exit(do_build(a.release, a.mem))
    if a.cmd == "run":
        sys.exit(do_run(a.release, a.mem))


if __name__ == "__main__":
    main()

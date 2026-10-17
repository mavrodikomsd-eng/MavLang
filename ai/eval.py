"""Оценка MavLang-выходов модели (10 тестов). Только стандартная библиотека.

Проверяет БЕЗ зависимости от compiler/ и mavc.py: правила parse_and_transpile
продублированы ниже 1:1 (шапка --mem, требование fn main, баланс скобок,
замена print(...) -> println!(...) тем же regex).

Запуск:
    python ai/eval.py                 # самопроверка 10 референсов (ожидается 10/10)
    python ai/eval.py --preds preds.jsonl   # оценить выходы модели:
        preds.jsonl — строки {"id": "<id теста>", "output": "<код MavLang>"}

Метрики для тиктока (печатаются в конце при --preds):
    fn_main_rate, transpile_pass_rate, sugar_rate
"""
import argparse
import json
import re
import sys

HEADER = {
    "borrow": "// MavLang mem=borrow | transpiled to Rust (safe borrows)\n",
    "arena": "// MavLang mem=arena | TODO: bump allocator wrapper\n",
    "manual": "// MavLang mem=manual | unsafe allowed\n",
}


def parse_and_transpile(src, mem="borrow"):
    """Копия правил mavc.py::parse_and_transpile (эвристика, без cargo)."""
    if "fn main" not in src:
        raise SyntaxError("нужен fn main() {...} в .mavl")
    if src.count("{") != src.count("}"):
        raise SyntaxError("дисбаланс { } в .mavl")

    def _print_repl(m):
        arg = m.group(1).strip()
        if arg.startswith('"') or arg.startswith("'"):
            return "println!(" + arg + ")"
        return 'println!("{}", ' + arg + ")"

    out_lines = []
    for line in src.splitlines():
        if "println!" not in line:
            line = re.sub(r"(?<!:)print\((.+?)\)", _print_repl, line)
        out_lines.append(line)
    return HEADER[mem] + "\n".join(out_lines) + "\n"


def check_output(output, mem="borrow"):
    """Возвращает список проваленных проверок (пусто = PASS)."""
    fails = []
    if "```" in output:
        fails.append("markdown-fence (код без заборов ```)")
    if "fn main" not in output:
        fails.append("no-fn-main")
    if output.count("{") != output.count("}"):
        fails.append("brace-imbalance")
    if "println!" in output:
        fails.append("println-leak (в MavLang только сахар print(); println! запрещён)")
    try:
        rs = parse_and_transpile(output, mem)
    except SyntaxError as e:
        fails.append("transpile-error: " + str(e))
        rs = ""
    if "print(" in output and "println!" not in rs:
        fails.append("sugar-not-transpiled")
    # мягкая проверка ';': let / присваивания / print / return должны кончаться на ';'
    for i, line in enumerate(output.splitlines(), start=1):
        s = line.strip()
        if not s or s.startswith("//") or s.endswith("{") or s.endswith("}"):
            continue
        if s.startswith(("fn ", "if ", "if(", "else", "while ", "for ",
                         "match ", "struct ", "enum ", "impl ", "unsafe")):
            continue
        head = s.split()[0] if s.split() else ""
        is_stmt = (s.startswith("let ") or s.startswith("print(")
                   or s.startswith("return ") or s.startswith("break")
                   or re.match(r"^[\w\)\]\*][\w\.\[\]\(\)\*\s]*=[^=>]", s))
        if is_stmt and not s.endswith(";"):
            fails.append("missing-semicolon line %d: %r" % (i, s))
    return fails


EVAL_CASES = [
    {"id": "print-var", "mem": "borrow",
     "instruction": "Объяви x: i32 = 5 и выведи.",
     "input": "",
     "reference": "fn main() {\n    let x: i32 = 5;\n    print(x);\n}"},
    {"id": "if-else", "mem": "borrow",
     "instruction": "Если x > 5 выведи big, иначе small.",
     "input": "x = 8",
     "reference": ("fn main() {\n    let x: i32 = 8;\n    if x > 5 {\n"
                   '        print("big");\n    } else {\n        print("small");\n    }\n}')},
    {"id": "while-loop", "mem": "borrow",
     "instruction": "Выведи числа 0..2 через while.",
     "input": "",
     "reference": ("fn main() {\n    let mut i = 0;\n    while i < 3 {\n"
                   "        print(i);\n        i = i + 1;\n    }\n}")},
    {"id": "for-range", "mem": "borrow",
     "instruction": "Выведи числа 0..2 через for.",
     "input": "0..3",
     "reference": "fn main() {\n    for i in 0..3 {\n        print(i);\n    }\n}"},
    {"id": "function", "mem": "borrow",
     "instruction": "Функция сложения, выведи add(3, 4).",
     "input": "Сигнатура: fn add(a: i32, b: i32) -> i32",
     "reference": ("fn add(a: i32, b: i32) -> i32 {\n    return a + b;\n}\n\n"
                   "fn main() {\n    let r: i32 = add(3, 4);\n    print(r);\n}")},
    {"id": "struct", "mem": "borrow",
     "instruction": "Структура Point, выведи поля.",
     "input": "struct Point",
     "reference": ("struct Point { x: i32, y: i32 }\n\nfn main() {\n"
                   "    let p = Point { x: 3, y: 4 };\n"
                   "    print(p.x);\n    print(p.y);\n}")},
    {"id": "enum-match", "mem": "borrow",
     "instruction": "Разбери enum через match.",
     "input": "enum Direction",
     "reference": ("enum Direction { Up, Down }\n\nfn main() {\n"
                   "    let d = Direction::Up;\n    match d {\n"
                   "        Direction::Up => {\n            print(\"up\");\n        }\n"
                   "        Direction::Down => {\n            print(\"down\");\n        }\n"
                   "    }\n}")},
    {"id": "borrow-shared", "mem": "borrow",
     "instruction": "Функция берёт &str по ссылке.",
     "input": "fn greet(name: &str)",
     "reference": ("fn greet(name: &str) {\n    print(name);\n}\n\n"
                   "fn main() {\n    greet(\"mira\");\n}")},
    {"id": "borrow-mut", "mem": "borrow",
     "instruction": "Функция удваивает число через &mut.",
     "input": "fn double(x: &mut i32)",
     "reference": ("fn double(x: &mut i32) {\n    *x = *x * 2;\n}\n\n"
                   "fn main() {\n    let mut n = 21;\n    double(&mut n);\n    print(n);\n}")},
    {"id": "unsafe-manual", "mem": "manual",
     "instruction": "В режиме manual запиши число через *mut i32 в unsafe.",
     "input": "--mem=manual",
     "reference": ("fn main() {\n    // manual mem: raw pointer write\n"
                   "    let mut x: i32 = 5;\n    let p: *mut i32 = &mut x;\n"
                   "    unsafe {\n        *p = 10;\n    }\n    print(x);\n}")},
]


def run_selftest():
    ok = 0
    for case in EVAL_CASES:
        fails = check_output(case["reference"], case["mem"])
        mark = "PASS" if not fails else "FAIL " + "; ".join(fails)
        print("[%s] %-13s %s" % ("OK" if not fails else "XX", case["id"], mark))
        ok += 0 if fails else 1
    print("selftest: %d/%d" % (ok, len(EVAL_CASES)))
    return ok == len(EVAL_CASES)


def run_preds(path):
    preds = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            if line.strip():
                r = json.loads(line)
                preds[r["id"]] = r.get("output", "")
    n = len(EVAL_CASES)
    fn_main = transpiled = sugar = full = 0
    for case in EVAL_CASES:
        out = preds.get(case["id"], "")
        fails = check_output(out, case["mem"])
        passed = not fails
        full += passed
        fn_main += "fn main" in out
        try:
            rs = parse_and_transpile(out, case["mem"])
            transpiled += 1
            sugar += "println!" in rs
        except SyntaxError:
            pass
        print("[%s] %-13s %s" % ("OK" if passed else "XX", case["id"],
                                 "PASS" if passed else "FAIL " + "; ".join(fails)))
    print("--- metrics (для тиктока) ---")
    print("fn_main_rate:        %d/%d = %.0f%%" % (fn_main, n, 100 * fn_main / n))
    print("transpile_pass_rate: %d/%d = %.0f%%" % (transpiled, n, 100 * transpiled / n))
    print("sugar_rate:          %d/%d = %.0f%%" % (sugar, n, 100 * sugar / n))
    print("full_pass:           %d/%d = %.0f%%" % (full, n, 100 * full / n))
    return full == n


def main():
    ap = argparse.ArgumentParser(description="Eval MavLang-выходов (10 тестов)")
    ap.add_argument("--preds", default=None, help="preds.jsonl с полями id/output")
    a = ap.parse_args()
    good = run_preds(a.preds) if a.preds else run_selftest()
    sys.exit(0 if good else 1)


if __name__ == "__main__":
    main()

"""Хард-тир: 12 настоящих программ. Эвристика как в eval.py + --preds.

    python ai/eval_hard.py                 # референсы (ожидается 12/12)
    python ai/eval_hard.py --preds ai/preds_hard_lora.jsonl

Референсы отдельно прогнаны через настоящий rustc: python ai/verify_hard.py
"""
import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from eval import check_output, parse_and_transpile

HARD_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "hard")

HARD_CASES = [
    {"id": "fizzbuzz", "mem": "borrow",
     "instruction": "Выведи fizzbuzz для чисел 1..15: кратно 3 и 5 — fizzbuzz, 3 — fizz, 5 — buzz, иначе число.",
     "input": "1..15"},
    {"id": "csvsum", "mem": "borrow",
     "instruction": "Посчитай сумму чисел в строке через запятую; выведи ТОЛЬКО число.",
     "input": '"10,20,30" -> 60'},
    {"id": "wordcount", "mem": "borrow",
     "instruction": "Посчитай слова в строке с лишними пробелами; первое слово выведи ВЕРХНИМ регистром (upper, не capitalize).",
     "input": '"  hello world hello  " -> 3, HELLO'},
    {"id": "fileroundtrip", "mem": "borrow",
     "instruction": "Запиши 42 в файл, прочитай обратно и выведи +8.",
     "input": "файл hard_w.txt"},
    {"id": "argsdefault", "mem": "borrow",
     "instruction": "Выведи ПЕРВЫЙ аргумент командной строки (аргумент 1, не имя программы) или РОВНО 'none' строчными если его нет.",
     "input": "args()"},
    {"id": "counter", "mem": "borrow",
     "instruction": "Структура-счётчик: подними через &mut 5 раз в цикле while.",
     "input": "struct Counter"},
    {"id": "fib", "mem": "borrow",
     "instruction": "Рекурсивный фибоначчи, fib(10). Печатай ТОЛЬКО число 55 — без меток, подписей и текста.",
     "input": "fib(10) = 55"},
    {"id": "normalize", "mem": "borrow",
     "instruction": "Нормализуй строку: обрежь пробелы и в нижний регистр, одним выражением.",
     "input": '"  Rust Lang  " -> rust lang'},
    {"id": "borrowseries", "mem": "borrow",
     "instruction": "Покажи borrow-области: let v = 1; в блоке напечатай shared-заимствование &v (это 1), после блока v = 2 и напечатай v (это 2). Вывод: ровно '1' и '2'.",
     "input": "&v в блоке, затем v+1"},
    {"id": "ptroffset", "mem": "manual",
     "instruction": "В режиме manual запиши 99 во ВТОРОЙ элемент массива через *mut i32 и offset, затем выведи ТОЛЬКО этот второй элемент — число 99 (не весь массив).",
     "input": "--mem=manual, [10, 20, 30]"},
    {"id": "inputmath", "mem": "borrow",
     "instruction": "Спроси число через input; промпт РОВНО 'n? '. Для ввода 21 выведи 42. Точный вывод: 'n? 42'.",
     "input": "stdin 21 -> 42"},
    {"id": "repconcat", "mem": "borrow",
     "instruction": "Функция rep(s, n) возвращает строку s повторённую n раз.",
     "input": 'rep("ab", 3) = ababab'},
    {"id": "argfile", "mem": "borrow",
     "instruction": "Прочитай файл из аргумента 1 и выведи его содержимое; если аргумента нет — выведи РОВНО 'usage' строчными без двоеточий.",
     "input": "arg(1), файл arg_h.txt"},
    {"id": "linesjoin", "mem": "borrow",
     "instruction": "Разбей текст по строкам, выведи количество и склейку через дефис.",
     "input": '"a\nb\nc" -> 3, a-b-c'},
    {"id": "containscheck", "mem": "borrow",
     "instruction": "Лог = строка с 'error'. Если contains(log, 'error') — печатай 'found'; затем check(2 * 3 == 6) и печатай 'ok'. Вывод: 'found' и 'ok' двумя строками.",
     "input": "contains + check"},
    {"id": "appendlog", "mem": "borrow",
     "instruction": "Запиши строку в файл, допиши ещё две строки (итого в файле 3 строки), выведи len(lines(...)) = 3.",
     "input": "app.log -> 3"},
]


def load_ref(cid):
    for fn in os.listdir(HARD_DIR):
        if fn.endswith(".mavl") and fn[3:-5] == cid:
            with open(os.path.join(HARD_DIR, fn), encoding="utf-8") as f:
                return f.read()
    raise KeyError(cid)


def run_selftest():
    ok = 0
    for case in HARD_CASES:
        ref = load_ref(case["id"])
        fails = check_output(ref, case["mem"])
        mark = "PASS" if not fails else "FAIL " + "; ".join(fails)
        print("[%s] %-13s %s" % ("OK" if not fails else "XX", case["id"], mark))
        ok += 0 if fails else 1
    print("hard selftest: %d/%d" % (ok, len(HARD_CASES)))
    return ok == len(HARD_CASES)


def run_preds(path):
    preds = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            if line.strip():
                r = json.loads(line)
                preds[r["id"]] = r.get("output", "")
    n = len(HARD_CASES)
    fn_main = transpiled = full = 0
    for case in HARD_CASES:
        out = preds.get(case["id"], "")
        fails = check_output(out, case["mem"])
        full += not fails
        fn_main += "fn main" in out
        try:
            parse_and_transpile(out, case["mem"])
            transpiled += 1
        except SyntaxError:
            pass
        print("[%s] %-13s %s" % ("OK" if not fails else "XX", case["id"],
                                 "PASS" if not fails else "FAIL " + "; ".join(fails)))
    print("--- hard metrics ---")
    print("fn_main_rate:        %d/%d = %.0f%%" % (fn_main, n, 100 * fn_main / n))
    print("transpile_pass_rate: %d/%d = %.0f%%" % (transpiled, n, 100 * transpiled / n))
    print("full_pass:           %d/%d = %.0f%%" % (full, n, 100 * full / n))
    return full == n


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--preds", default=None)
    a = ap.parse_args()
    good = run_preds(a.preds) if a.preds else run_selftest()
    sys.exit(0 if good else 1)


if __name__ == "__main__":
    main()

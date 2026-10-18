# -*- coding: utf-8 -*-
"""Генерация предсказаний хард-бенча через OpenRouter chat completions.

    python ai/gen_openrouter_preds.py --model anthropic/claude-haiku-4.5 --out ai/preds_haiku45.jsonl

Ключ читает из C:\\Users\\admin\\.config\\opencode\\or_key.txt
"""
import argparse
import json
import os
import urllib.request

sys_dir = os.path.dirname(os.path.abspath(__file__))
spec_path = os.path.join(sys_dir, "..", "LLMS.md")

with open(spec_path, encoding="utf-8") as _f:
    SPEC = _f.read()

from eval_hard import HARD_CASES

KEY_PATH = r"C:\Users\admin\.config\opencode\or_key.txt"
API = "https://openrouter.ai/api/v1/chat/completions"

SYSTEM = """Ты — компилятор-ассистент языка MavLang (.mavl), низкоуровневого языка в духе Rust/Zig.
Ниже — ПОЛНАЯ спецификация языка MavLang v0.5.

{spec}

ПРАВИЛА ОТВЕТА:
- Отвечай ТОЛЬКО кодом на MavLang, без markdown-заборов и пояснений.
- Обязательно fn main() {{...}}.
- Каждый стейтмент заканчивается ';' (кроме блоков {{ }}).
- Встроенные функции доступны без импортов: print, input, len, read_file, write_file,
  append_file, split, lines, join, trim, upper, lower, capitalize, to_int, sleep,
  args, arg, contains, check, env, exists, time_ms, dir, gui::window.
- Вызывай их как ФУНКЦИИ: trim(s), upper(s) — не как методы s.trim().
- Если задача требует unsafe — используй блок unsafe {{...}} (программа будет собрана с --mem=manual)."""


def call(model, user_text, key):
    body = {
        "model": model,
        "messages": [
            {"role": "system", "content": SYSTEM.format(spec=SPEC)},
            {"role": "user", "content": user_text},
        ],
        "max_tokens": 1500,
        "temperature": 0.0,
    }
    req = urllib.request.Request(
        API, method="POST",
        headers={"Authorization": "Bearer " + key, "Content-Type": "application/json"},
        data=json.dumps(body).encode("utf-8"))
    with urllib.request.urlopen(req, timeout=180) as r:
        data = json.loads(r.read().decode("utf-8"))
    return data["choices"][0]["message"]["content"], data.get("usage", {})


def clean(text):
    t = text.strip()
    if t.startswith("```"):
        lines = t.split("\n")
        lines = [ln for ln in lines if not ln.strip().startswith("```")]
        t = "\n".join(lines)
    return t.strip()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default="anthropic/claude-haiku-4.5")
    ap.add_argument("--out", default="ai/preds_haiku45.jsonl")
    a = ap.parse_args()
    with open(KEY_PATH, encoding="utf-8") as f:
        key = f.read().strip()
    out_path = os.path.join(sys_dir, "..", a.out) if not os.path.isabs(a.out) else a.out
    tok_in = tok_out = 0
    with open(out_path, "w", encoding="utf-8") as w:
        for case in HARD_CASES:
            task = case["instruction"] + "\n" + case["input"]
            try:
                text, usage = call(a.model, task, key)
            except Exception as e:
                text, usage = "", {"error": str(e)}
                print("[XX] %-13s ERROR %s" % (case["id"], str(e)[:120]))
            tok_in += usage.get("prompt_tokens", 0)
            tok_out += usage.get("completion_tokens", 0)
            w.write(json.dumps({"id": case["id"], "model": a.model,
                                "output": clean(text)}, ensure_ascii=False) + "\n")
            w.flush()
            print("[OK] %-13s %d chars" % (case["id"], len(text)))
    print("--- usage total ---")
    print("prompt tokens: %d, completion tokens: %d" % (tok_in, tok_out))


if __name__ == "__main__":
    main()

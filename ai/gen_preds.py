"""Генерация ответов модели под 10 тестов ai/eval.py.

    python ai/gen_preds.py --base C:\\models\\Qwen2.5-Coder-0.5B-Instruct --out preds_base.jsonl
    python ai/gen_preds.py --base C:\\models\\Qwen2.5-Coder-0.5B-Instruct --adapter ai/checkpoints/mavlang-lora --out preds_lora.jsonl
    python ai/eval.py --preds preds_base.jsonl
    python ai/eval.py --preds preds_lora.jsonl
"""
import argparse
import json

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

from eval import EVAL_CASES
from eval_hard import HARD_CASES

PROMPT = (
    "### Instruction:\n{instruction}\n\n"
    "### Input:\n{input}\n\n"
    "### Response:\n"
)

INVISIBLE_JUNK = ("\u23ce",  # ⏎ SYMBOL FOR RETURN: observed garbage after fizzbuzz/argsdefault
                  "\u200b", "\u200c", "\u200d",  # zero-width chars: never valid code
                  "\ufeff")  # BOM mid-stream: never valid code


def sanitize(text):
    """Убрать мусорные символы генерации, не трогая содержимое строк.

    Удаляются только символы, которые заведомо не могут быть легитимным
    кодом/строковым содержимым: ⏎ (U+23CE ломал rustc: unknown start of
    token), zero-width и BOM. Кириллица и прочие видимые символы внутри
    строковых литералов не затрагиваются. Markdown-заборы ``` намеренно
    НЕ трогаем (их чистит постобработка при оценке).
    """
    for ch in INVISIBLE_JUNK:
        text = text.replace(ch, "")
    # trailing NBSP в концах строк (strip() его не всегда берёт)
    # markdown-заборы: строка, начинающаяся с ```, в легитимном коде
    # невозможна (это всегда fence ```/```rust/```mavl)
    lines = [ln for ln in text.splitlines()
             if not ln.lstrip().startswith("```")]
    lines = [ln.rstrip(" ") for ln in lines]
    text = "\n".join(lines).strip()
    # хвостовые $ (наблюдалось `}$$` в конце argfile): $ вне строковых
    # литералов невалиден, а легитимный код на $ не заканчивается
    # (строка "…$" закрывается кавычкой, стейтмент — точкой с запятой)
    text = text.rstrip("$").rstrip()
    return text


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", required=True)
    ap.add_argument("--adapter", default=None)
    ap.add_argument("--out", required=True)
    ap.add_argument("--temp", type=float, default=0.2)
    ap.add_argument("--max-new", type=int, default=220)
    ap.add_argument("--cases", choices=["easy", "hard"], default="easy")
    ap.add_argument("--rep-pen", type=float, default=1.15)
    a = ap.parse_args()
    cases = HARD_CASES if a.cases == "hard" else EVAL_CASES

    tok = AutoTokenizer.from_pretrained(a.base, trust_remote_code=True)
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token
    model = AutoModelForCausalLM.from_pretrained(
        a.base, torch_dtype=torch.bfloat16, device_map="auto", trust_remote_code=True)
    if a.adapter:
        from peft import PeftModel
        model = PeftModel.from_pretrained(model, a.adapter)
        model = model.merge_and_unload()
    model.eval()

    rows = []
    for case in cases:
        prompt = PROMPT.format(instruction=case["instruction"], input=case["input"])
        ids = tok(prompt, return_tensors="pt").to(model.device)
        with torch.no_grad():
            gen = model.generate(**ids, max_new_tokens=a.max_new,
                                 do_sample=a.temp > 0, temperature=max(a.temp, 1e-6),
                                 repetition_penalty=a.rep_pen,
                                 pad_token_id=tok.pad_token_id)
        text = tok.decode(gen[0][ids["input_ids"].shape[1]:], skip_special_tokens=True)
        # отрезаем возможный мусор после конца кода
        if "### " in text:
            text = text.split("### ")[0]
        rows.append({"id": case["id"], "output": sanitize(text)})
        print(f"[{case['id']}] {len(text)} chars")
    with open(a.out, "w", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print("saved ->", a.out)


if __name__ == "__main__":
    main()

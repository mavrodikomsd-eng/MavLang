"""LoRA файнтюн Qwen2.5-Coder-0.5B-Instruct под MavLang v0.1.

Железо: RTX 4060 Laptop 8GB. НЕ запускать без venv Python 3.11 + torch CUDA 12 (см. ai/README.md).

Примеры запуска:
    python ai/train_lora.py --data ai/dataset.jsonl --out ai/checkpoints/mavlang-lora
    python ai/train_lora.py --data ai/dataset.jsonl --out ai/checkpoints/mavlang-lora --no-4bit
    python ai/train_lora.py --data ai/dataset.jsonl --out ai/checkpoints/mavlang-lora --epochs 5 --batch 1 --grad-accum 16

Требует (ставить в venv, НЕ в системный Python):
    pip install torch --index-url https://download.pytorch.org/whl/cu121
    pip install transformers peft datasets accelerate bitsandbytes
"""
import argparse
import json

PROMPT = (
    "### Instruction:\n{instruction}\n\n"
    "### Input:\n{input}\n\n"
    "### Response:\n{output}"
)

BASE_MODEL = "Qwen/Qwen2.5-Coder-0.5B-Instruct"


def load_rows(path, val_split=0.05, seed=42):
    with open(path, encoding="utf-8") as f:
        rows = [json.loads(line) for line in f if line.strip()]
    assert rows, "dataset пуст: " + path
    for r in rows:
        assert {"instruction", "input", "output"} <= set(r), "строка без полей instruction/input/output"
        assert "fn main" in r["output"], "output без fn main"
    n_val = max(1, int(len(rows) * val_split))
    return rows[:-n_val] or rows, rows[-n_val:]


def main():
    ap = argparse.ArgumentParser(description="LoRA SFT для MavLang на Qwen2.5-Coder-0.5B")
    ap.add_argument("--data", default="ai/dataset.jsonl")
    ap.add_argument("--out", default="ai/checkpoints/mavlang-lora")
    ap.add_argument("--base", default=BASE_MODEL)
    ap.add_argument("--epochs", type=float, default=3)
    ap.add_argument("--batch", type=int, default=1, help="per-device batch (8GB: держи 1)")
    ap.add_argument("--grad-accum", type=int, default=16, help="эффективный батч = batch * grad-accum")
    ap.add_argument("--lr", type=float, default=2e-4)
    ap.add_argument("--max-len", type=int, default=1024)
    ap.add_argument("--lora-r", type=int, default=16)
    ap.add_argument("--lora-alpha", type=int, default=32)
    ap.add_argument("--no-4bit", action="store_true", help="выключить 4bit (нужно больше VRAM)")
    ap.add_argument("--seed", type=int, default=42)
    a = ap.parse_args()

    import torch
    from datasets import Dataset
    from transformers import (
        AutoModelForCausalLM,
        AutoTokenizer,
        DataCollatorForLanguageModeling,
        Trainer,
        TrainingArguments,
    )
    from peft import LoraConfig, get_peft_model, prepare_model_for_kbit_training

    train_rows, val_rows = load_rows(a.data)
    print(f"train={len(train_rows)} val={len(val_rows)} base={a.base}")

    tok = AutoTokenizer.from_pretrained(a.base, trust_remote_code=True)
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token
    tok.padding_side = "right"

    def fmt(r):
        return PROMPT.format(instruction=r["instruction"], input=r["input"], output=r["output"])

    def tokenize(batch):
        return tok(batch["text"], truncation=True, max_length=a.max_len)

    train_ds = Dataset.from_list([{"text": fmt(r)} for r in train_rows]).map(
        tokenize, batched=True, remove_columns=["text"])
    val_ds = Dataset.from_list([{"text": fmt(r)} for r in val_rows]).map(
        tokenize, batched=True, remove_columns=["text"])

    use_4bit = not a.no_4bit
    if use_4bit:
        from transformers import BitsAndBytesConfig
        bnb = BitsAndBytesConfig(
            load_in_4bit=True,
            bnb_4bit_quant_type="nf4",
            bnb_4bit_compute_dtype=torch.bfloat16,
            bnb_4bit_use_double_quant=True,
        )
        model = AutoModelForCausalLM.from_pretrained(
            a.base, quantization_config=bnb, device_map="auto", trust_remote_code=True)
        model = prepare_model_for_kbit_training(model)
    else:
        model = AutoModelForCausalLM.from_pretrained(
            a.base, torch_dtype=torch.bfloat16, device_map="auto", trust_remote_code=True)

    model.gradient_checkpointing_enable()  # обязательно под 8GB
    model.config.use_cache = False

    lora = LoraConfig(
        r=a.lora_r,
        lora_alpha=a.lora_alpha,
        lora_dropout=0.05,
        bias="none",
        task_type="CAUSAL_LM",
        target_modules=["q_proj", "k_proj", "v_proj", "o_proj",
                        "gate_proj", "up_proj", "down_proj"],
    )
    model = get_peft_model(model, lora)
    model.print_trainable_parameters()

    args = TrainingArguments(
        output_dir=a.out,
        num_train_epochs=a.epochs,
        per_device_train_batch_size=a.batch,
        per_device_eval_batch_size=1,
        gradient_accumulation_steps=a.grad_accum,
        learning_rate=a.lr,
        lr_scheduler_type="cosine",
        warmup_steps=2,  # transformers>=5: warmup_ratio убран
        bf16=True,              # RTX 4060 (Ada) умеет bf16
        tf32=True,
        optim="paged_adamw_8bit" if use_4bit else "adamw_torch",
        gradient_checkpointing=True,
        logging_steps=10,
        eval_strategy="steps",
        eval_steps=50,
        save_steps=50,
        save_total_limit=2,
        report_to="none",
        seed=a.seed,
    )

    trainer = Trainer(
        model=model,
        args=args,
        train_dataset=train_ds,
        eval_dataset=val_ds,
        data_collator=DataCollatorForLanguageModeling(tok, mlm=False),
    )
    trainer.train()
    model.save_pretrained(a.out)
    tok.save_pretrained(a.out)
    print("saved LoRA ->", a.out)


if __name__ == "__main__":
    main()

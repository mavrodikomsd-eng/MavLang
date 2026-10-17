import argparse, time, json, torch
from transformers import AutoModelForCausalLM, AutoTokenizer
from transformers import BitsAndBytesConfig

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", required=True)
    ap.add_argument("--data", default="ai/dataset.jsonl")
    ap.add_argument("--n", type=int, default=5)
    ap.add_argument("--max-new", type=int, default=150)
    ap.add_argument("--load-in-4bit", action="store_true", default=True)
    a = ap.parse_args()

    tok = AutoTokenizer.from_pretrained(a.base, trust_remote_code=True)
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token

    if a.load_in_4bit:
        bnb = BitsAndBytesConfig(
            load_in_4bit=True,
            bnb_4bit_quant_type="nf4",
            bnb_4bit_compute_dtype=torch.bfloat16,
            bnb_4bit_use_double_quant=True,
        )
        model = AutoModelForCausalLM.from_pretrained(
            a.base, quantization_config=bnb, device_map="auto", trust_remote_code=True)
    else:
        model = AutoModelForCausalLM.from_pretrained(
            a.base, torch_dtype=torch.bfloat16, device_map="auto", trust_remote_code=True)
    model.eval()

    rows = [json.loads(l) for l in open(a.data, encoding="utf-8")][:a.n]
    prompts = []
    for r in rows:
        prompts.append(
            f"### Instruction:\n{r['instruction']}\n\n### Input:\n{r['input']}\n\n### Response:\n"
        )

    total_in = 0
    total_out = 0
    total_t = 0.0
    for i, p in enumerate(prompts):
        ids = tok(p, return_tensors="pt").to(model.device)
        inp = ids.input_ids.shape[1]
        t0 = time.perf_counter()
        with torch.no_grad():
            gen = model.generate(
                **ids,
                max_new_tokens=a.max_new,
                do_sample=False,
                pad_token_id=tok.pad_token_id,
            )
        dt = time.perf_counter() - t0
        out = gen.shape[1] - inp
        total_in += inp
        total_out += out
        total_t += dt
        print(f"[{i}] in={inp} out={out} t={dt:.2f}s tok/s={out/dt:.1f}")
    print(f"\n=== 4bit quantized ===")
    print(f"input tokens : {total_in}")
    print(f"output tokens: {total_out}")
    print(f"gen time     : {total_t:.2f}s")
    print(f"TOKENS/SEC   : {total_out/total_t:.1f}")

if __name__ == "__main__":
    main()

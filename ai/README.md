# AI-трек MavLang: дообучение Qwen2.5-Coder-0.5B под MavLang v0.1

Цель: маленькая локальная модель пишет валидный MavLang
(`fn main` обязателен, сахар `print(x);`, типы `i32/u32/f64/bool/String`,
`if/else/while/for/match/fn/struct`, `&`/`&mut`, `unsafe` для `--mem=manual`).

Состав папки:

| Файл | Что это |
|---|---|
| `ai/dataset.jsonl` | 120 примеров SFT (`instruction/input/output`), все с `fn main` |
| `ai/train_lora.py` | LoRA файнтюн (transformers + peft, r=16, 4bit опционально) |
| `ai/eval.py` | 10 тестов MavLang, только стандартная библиотека |
| `ai/README.md` | этот файл |

Датасет по категориям (строки в файле идут блоками по порядку):

| Блок | Категория | Штук |
|---|---|---|
| A (строки 1–30) | `print` / переменные (`let`, `mut`, арифметика, `u32/f64/bool/String`) | 30 |
| B (строки 31–60) | `if/else`, `else if`, `while`, `for`, вложенные циклы | 30 |
| C (строки 61–80) | функции `fn`, возврат, композиция, рекурсия | 20 |
| D (строки 81–100) | `struct`, `impl`, `enum`, `match` | 20 |
| E (строки 101–120) | borrow `&` / `&mut`, `Vec`, `unsafe` под `--mem=manual` | 20 |

## 1. Окружение (один раз)

В системе стоит Python 3.14, но для ML нужен **Python 3.11** отдельным venv
(torch + CUDA 12 стабильнее всего именно на 3.11).

```powershell
# поставить Python 3.11 с python.org, затем:
py -3.11 -m venv C:\1\аап\ai\.venv
C:\1\аап\ai\.venv\Scripts\Activate.ps1
python -m pip install --upgrade pip
# torch под CUDA 12.x (RTX 4060 Laptop):
pip install torch --index-url https://download.pytorch.org/whl/cu121
# остальное для обучения:
pip install transformers peft datasets accelerate bitsandbytes
python -c "import torch; print(torch.__version__, torch.cuda.is_available())"
# ожидается: 2.x.x+cu121 True
```

Тяжёлые пакеты в системный Python 3.14 НЕ ставить — только в `ai/.venv`.

## 2. Проверка данных (без GPU, прямо сейчас)

```powershell
# 120 строк, все с fn main:
python -c "import json; rows=[json.loads(l) for l in open('ai/dataset.jsonl', encoding='utf-8')]; print(len(rows), all('fn main' in r['output'] for r in rows))"
# 10/10 референсных тестов:
python ai/eval.py
```

## 3. Обучение (нужен GPU 8GB)

```powershell
C:\1\аап\ai\.venv\Scripts\Activate.ps1
# быстрый прогон (4bit, ~4-5GB VRAM):
python ai/train_lora.py --data ai/dataset.jsonl --out ai/checkpoints/mavlang-lora
# точнее/longer:
python ai/train_lora.py --data ai/dataset.jsonl --out ai/checkpoints/mavlang-lora --epochs 5 --grad-accum 16
# без 4bit (больше VRAM, чуть точнее):
python ai/train_lora.py --data ai/dataset.jsonl --out ai/checkpoints/mavlang-lora --no-4bit
```

Ключевые настройки под 8GB (уже вшиты): LoRA `r=16`, `batch=1` + `grad-accum=16`,
градиентный чекпоинтинг, `bf16`, `paged_adamw_8bit` в 4bit-режиме, `max-len=1024`.

## 4. Оценка модели

Сгенерируй 10 ответов модели (по одному на каждый `instruction` из `ai/eval.py`,
температура ~0.2), сложи в `preds.jsonl` строками `{"id": "<id>", "output": "<код>"}`,
затем:

```powershell
python ai/eval.py --preds preds.jsonl
```

Скрипт напечатает 4 метрики. Правила проверки продублированы из `mavc.py`
1:1 (шапка `--mem`, `fn main`, баланс `{ }`, `print()` → `println!`), но `cargo`
не вызывается — это эвристика, финальный прогон делай через `mavc check`.

## 5. Таблица для тиктока (3 цифры)

Метрики (считает `ai/eval.py --preds`):

| Метрика | Что меряет | Референс* | Base (замерь до LoRA) | LoRA (после) |
|---|---|---|---|---|
| `fn_main_rate` | доля выходов с `fn main` | 10/10 = **100%** | ___ % | ___ % |
| `transpile_pass_rate` | доля выходов, проходящих эвристику транспилера | 10/10 = **100%** | ___ % | ___ % |
| `full_pass` | доля выходов без единого замечания (сахар `print`, `;`, скобки) | 10/10 = **100%** | ___ % | ___ % |

\* Референс = эталонные ответы из `ai/eval.py` (потолок, уже замерен: `python ai/eval.py` → 10/10).
«До» — прогони base `Qwen2.5-Coder-0.5B-Instruct` без файнтюна через `ai/eval.py --preds`,
«после» — тем же скриптом LoRA-адаптер. Разница трёх цифр — сюжет ролика:
«модель не знала наш язык → выучила за N минут на 4060».

Датасет тоже самопроверен: 120/120 строк содержат `fn main`, баланс `{ }` сходится,
утечек `println!` в исходниках нет.

## 6. Хард-тир (замерено 16.10.2026, RTX 4060 Laptop)

- `ai/hard/` — 12 настоящих программ (fizzbuzz, csv-сумма, файлы, args,
  struct+&mut, рекурсивный fib, пайплайны, unsafe offset, input, конкатенация).
- `ai/verify_hard.py` — каждый референс прогнан через настоящий
  `mavc build+run`: **13/13** (включая вариант args).
- `ai/eval_hard.py` — эвристика для выходов модели + `--preds`.
- Датасет вырос до **160 строк** (блок F: 12 хард + 28 std-drills), обучение
  перезапущено (4 эпохи, ~7 мин, loss 1.60 → 0.51).

Итог замеров (`--preds`, температура 0.2):

| | easy 10 тестов | hard 12 задач (эвристика) | hard по-настоящему (mavc build+run) |
|---|---|---|---|
| Base Qwen2.5-Coder-0.5B | 0/10 | 0/12 | 0/12 |
| + наш LoRA | **10/10** | **10/12** | **6/12** |

6 реальных: fizzbuzz, wordcount (`3`, `HELLO`), counter (`5`), fib (`55`),
ptroffset (`99`), repconcat (`ababab`). 4 завала честно: модель выдумывает
API (`open/close`, `args().next()`), бормочет повторы и лепит ` ``` ` заборы
(лечится strip-постобработкой, файлы `*_clean.jsonl`). Следующая итерация:
добить файловые/args-примеры в датасет.

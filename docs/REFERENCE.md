# MavLang — полная документация (v0.4)

MavLang (`.mavl`) — маленький учебный low-level язык в духе Rust/Zig.
`mavc` транспиллирует `.mavl` в Rust (`intRs/`) и собирает через `cargo`/`rustc`
в настоящий `.exe`. Снаружи выглядит как свой компилятор: ошибки показывают
строки твоего `.mavl`. Быстрый старт: `docs/README.md`, спека для ИИ: `LLMS.txt`.

## 1. Установка

Нужны `cargo + rustc`. Затем:

```powershell
cargo install --path compiler   # кладёт mavc.exe в ~/.cargo/bin (уже в PATH)
mavc check
```

VS Code: папка `vscode-mavlang/` → скопировать в
`%USERPROFILE%/.vscode/extensions/mavlang-0.1.0/`, перезапустить VS Code.
Даёт подсветку `.mavl` + LSP (`mavc lsp`): диагностика, completion (24 штуки),
hover-подсказки.

## 2. Структура проекта

```
myapp/
  source/*.mavl    твои исходники (main.mavl — главный)
  intRs/           вся раст-кухня: Cargo.toml, src/main.rs. Руками не трогать
  target/debug/    готовые .exe/.dll
  target/release/  то же после --release
  mavlang.toml     (необязательно) зависимости крейтов [deps]
  compiler/        исходники самого mavc (Rust)
  docs/            эта дока, LLMS.txt — короткая спека для ИИ
  examples/        примеры .mavl (не собираются автоматически)
  ai/              датасет + скрипты файнтюна + eval для тиктока
```

## 3. CLI

```
mavc check [--mem=...]     проверить все source/*.mavl без сборки
mavc build [--release] [--mem=...]
mavc run   [--release] [--mem=...]   собрать и запустить
mavc clean                       снести сборки
mavc lsp                         LSP-сервер для редактора (stdio)
```

Флаги: `--release` — оптимизированный бинарь в `target/release/`,
`--mem=borrow|arena|manual` (дефолт `borrow`), см. §5.

## 4. Синтаксис и стандартная библиотека

База — подмножество Rust, проходит в `intRs` почти 1:1 (поэтому ошибки
маппятся построчно). Правила: `fn main() {...}` обязателен, стейтменты с `;`,
комменты `//`.

```rust
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let x: i32 = 5;
    let mut y = 10;
    y = y + add(x, 2);
    if y > 10 {
        print(y);
    }
    for i in 0..3 {
        print(i);
    }
}
```

Встроенные функции (без импортов, работают и вложенно: `print(trim(s))`).
Свои функции с теми же именами (`fn trim...`) транспилер не трогает:

| Функция | Что делает | Пример |
|---|---|---|
| `print(x);` | печать | `print(y);` → `println!("{}", y)` |
| `input(p);` | строка из stdin с промптом | `let n = input("name? ");` |
| `len(x);` | длина | `len(v)` |
| `read_file(p);` | файл → `String` | `read_file("a.txt")` |
| `write_file(p, d);` | строка → файл | `write_file("o.txt", "hi")` |
| `split(s, d);` | → `Vec<String>` | `split("a,b", ",")` |
| `trim(s);` | обрезать пробелы | `trim("  hi  ")` |
| `upper(s);` / `lower(s);` | регистр | `upper("abc")` → `"ABC"` |
| `to_int(s);` | → `i32` | `to_int("42") + 8` |
| `sleep(ms);` | пауза | `sleep(100);` |
| `args();` | аргументы → `Vec<String>` | `args()` |
| `arg(i);` | i-й аргумент или `""` (0 — имя программы) | `arg(1)` |
| `append_file(p, d);` | дописать в конец файла | `append_file("log.txt", "hi\n")` |
| `lines(s);` | разбить по строкам → `Vec<String>` | `lines(read_file("a.txt"))` |
| `join(v, d);` | склеить обратно → `String` | `join(v, ",")` |
| `contains(s, sub);` | подстрока → `bool` | `if contains(s, "err")` |
| `find(s, sub);` | позиция подстроки или `-1` → `i32` | `find("hi world", "world")` → `3` |
| `replace(s, o, n);` | заменить подстроки → `String` | `replace("a-b", "-", "+")` → `"a+b"` |
| `check(c);` | проверка → `assert!` (для тестов) | `check(1 + 1 == 2)` |
| `capitalize(s);` | первая буква заглавная → `String` | `capitalize("hi")` → `"Hi"` |
| `env(name);` | переменная окружения → `String` | `env("PATH")` |
| `exists(path);` | есть ли файл → `bool` | `exists("a.txt")` |
| `time_ms();` | мс с эпохи → `i64` | `time_ms()` |
| `dir(path);` | файлы в папке → `Vec<String>` | `dir(".")` |
| `gui::window(t);` | заглушка GUI (печатает `[gui-stub]`) | тизер под будущий `egui` |

Остальное — обычный Rust: `if/else`, `while`, `for`, `match`, `struct/enum/impl`,
типы `i32 u32 f64 bool String &str Vec`, `std::` напрямую.

Сахарная строковая близость (v0.5.1):
- `res = res + s` — автоматический заем в строковые операнды (`&s`)
- `print("itog: " + x)` — в развернутом виде вызывает `format!` (не ломает арифметические `+`)
- `args().get(n)` — автоматически превращается в `arg(n)`
- `for w in vec_of_strings` — итерация по элементам

Подсказки mavc при ошибках rustc:
При частых кодах ошибок E0308/E0382/E0425/E0499/E0369 после вывода ошибки rustc будет выведена подсказка `[mavc] подсказка: ...` — на русском, с идиомой для исправления.

## 5. Режимы памяти (`--mem`)

- `borrow` — обычные `& / &mut`, проверяет `rustc` (дефолт).
- `arena` — заглушка под будущий bump-аллокатор (шапка генерации).
- `manual` — можно `unsafe` + сырые указатели.

## 6. Несколько файлов и крейты (v0.2+)

Все `source/*.mavl` склеиваются в один `intRs/src/main.rs`: `main.mavl`
первым, остальные по алфавиту, каждый с маркером `// <file>`. Ошибки маппятся
обратно по файлам.

`mavlang.toml` в корне (необязательно):

```toml
[deps]
egui = "0.28"
```

`mavc build` дописывает `[dependencies]` в `intRs/Cargo.toml`. Без файла —
как раньше.

## 7. Как это работает внутри

`compiler/src/main.rs`: `transpile_line` (проходы до fixpoint: `print` →
`println!`, std-функции → `std::...`, `gui::window` → стаб), склейка файлов,
`cargo build` в `intRs/`, копирование `mavapp.exe/.dll` наверх,
переписывание `main.rs:12` → `source/<file>:12`.
`compiler/src/lsp.rs`: `mavc lsp`, диагностика + completion + hover.

## 8. AI-трек (файнтюн мелкой модели)

Всё в `ai/` (подробно: `ai/README.md`):

- `dataset.jsonl` — 120 примеров SFT (`instruction/input/output`), все с
  `fn main`: print/переменные 30, ветвления/циклы 30, функции 20,
  struct/enum/match 20, borrow/unsafe 20.
- `train_lora.py` — LoRA (`r=16`) под `Qwen2.5-Coder-0.5B-Instruct`,
  впритык под 8GB VRAM (4bit, `batch=1x16`, чекпоинтинг, `bf16`).
- `eval.py` — 10 тестов, метрики `fn_main_rate / transpile_pass_rate /
  full_pass`, режим `--preds` для замера «до/после» под тикток.
- Окружение: **Python 3.11 venv** (`ai/.venv`), torch cu121, пакеты
  `transformers peft datasets accelerate bitsandbytes`. В системный
  Python 3.14 ничего ML-ного не ставить.

## 9. Roadmap

- [x] транспилер + CLI + `--mem` + debug/release
- [x] подсветка VS Code + LSP
- [x] мультифайл + `mavlang.toml` + `gui`-стаб
- [x] std v0.3 + датасет 120 + LoRA-скрипт + eval
- [ ] настоящий `--mem=arena`, мини-borrow checker
- [ ] настоящий GUI (`egui`), async-сахар
- [ ] сниппеты VS Code, иконка языка, `mavc --version`
- [ ] расширить датасет std-функциями, прогнать файнтюн на 4060

## 10. FAQ

**Почему синтаксис растоподобный?** v0.1 сделан транспилером 1:1 в Rust:
маппинг ошибок тривиален, borrow checker и оптимизации бесплатны. Свой
сильно другой синтаксис — это полный парсер, он позже.

**Почему компилятор написан на Rust, а не на MavLang?** Первая версия была на
Python (быстро), переписали на Rust ради одного `mavc.exe`. Self-hosted
(компилятор на самом MavLang) — отдельный инфоповод.

**`.ml` vs `.mavl`?** `.ml` занято OCaml, поэтому `.mavl`/`.mavlang`.

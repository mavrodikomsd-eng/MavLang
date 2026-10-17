# MavLang — архитектура компилятора v0.5

Мavc — транспилер с языка MavLang (`.mavl`) в Rust, который затем собирается через `cargo`/`rustc` в нативный бинарник. Весь компилятор написан на чистом Rust (без внешних парсерных библиотек, только `serde`/`serde_json` для LSP).

---

## 1. Высокоуровневая схема

```
 source/*.mavl
     │
     ▼  mavc (транспилер)
 intRs/src/main.rs  ← сгенерированный Rust-код
 intRs/Cargo.toml   ← манифест пакета (с зависимостями из mavlang.toml)
     │
     ▼  cargo build / rustc
 target/debug/mavapp.exe   ← готовый нативный бинарь
 target/release/mavapp.exe
     │
     ▼  пользователь запускает
```

---

## 2. Папки проекта

| Папка | Назначение |
|-------|-----------|
| `source/` | Исходники `.mavl`. Обязателен `main.mavl`. |
| `intRs/` | Вся "раст-кухня": `Cargo.toml`, `src/main.rs`. Генерируется mavc, править вручную не нужно. |
| `target/debug/` | Готовые `.exe`/`.dll` (debug-сборка). |
| `target/release/` | Готовые `.exe`/`.dll` (релизная сборка с `--release`). |
| `compiler/` | Исходники самого компилятора `mavc` на Rust. |
| `tasks/` | Примеры хард-задач (сгенерированы субагентом для бенчмарка). |

---

## 3. Поток компиляции (пошагово)

### Этап 1 — Поиск входных файлов (`collect_sources`)
Мavc сканирует папку `source/`, собирает все `*.mavl`. `main.mavl` ставится первым, остальные сортируются по алфавиту. Возвращает вектор `(имя_файла, содержимое)`.

### Этап 2 — Транспиляция (`transpile_line` / `combine_and_transpile`)
Каждая строка `.mavl` обрабатывается независимо:

1. **Проверка границ слов** (`is_word_boundary`): отсекает встроенные функции внутри обычных идентификаторов (например, `my_print(` не сработает как `print(`).
2. **Проверка определения функции** (`fn_before`): если перед именем стоит `fn`, встроенная функция не раскрывается — это позволяет объявлять свои функции с именами `trim`, `print` и т.д. без поломки.
3. **Раскрытие встроенных функций**:
   - `print(x)` → `println!("{}", x)` или `println!("{:?}", x)` для векторов.
   - `input(prompt)` → блок чтения из stdin с промптом.
   - `read_file(p)` → `std::fs::read_to_string(p).unwrap()`.
   - `write_file(p, d)` → `std::fs::write(p, d).unwrap()`.
   - `append_file(p, d)` → открытие файла в режиме добавления (`OpenOptions::append(true)`).
   - `split(s, d)` → `.split(d).map(|x| x.to_string()).collect::<Vec<String>>()`.
   - `lines(s)` → `.lines().map(...).collect::<Vec<String>>()`.
   - `join(v, d)` → `.join(d)`.
   - `trim/upper/lower/capitalize(s)` → соответствующие методы строки.
   - `to_int(s)` → `.trim().parse::<i32>().unwrap()`.
   - `sleep(ms)` → `std::thread::sleep(Duration::from_millis(ms))`.
   - `args()` → `std::env::args().collect::<Vec<String>>()`.
   - `arg(i)` → `std::env::args().nth(i as usize).unwrap_or_default()`.
   - `contains(s, sub)` → `.contains(sub)`.
   - `check(c)` → `assert!(c)`.
   - `env(name)` → `std::env::var(name).unwrap_or_else(...)`.
   - `exists(path)` → `std::path::Path::new(path).exists()`.
   - `time_ms()` → системное время в мс.
   - `dir(path)` → содержимое папки в `Vec<String>`.
   - `gui::window(x)` → `println!("[gui-stub] {}")` (заглушка под будущий `egui`).
   - Остальные строки проходят как есть (транзит в Rust).

4. **Fixpoint-цикл**: транспиляция повторяется до 8 раз, пока вложенные вызовы (например, `print(trim(s))`) полностью не раскроются.

### Этап 3 — Склейка файлов (`combine_and_transpile`)
Все исходники склеиваются в один `intRs/src/main.rs`:
- Шапка с режимом памяти (`// MavLang mem=borrow ...`).
- Комментарий `// <имя_файла>` перед каждым файлом (для маппинга ошибок).
- Содержимое каждого файла.

### Этап 4 — Маппинг ошибок (`map_rust_line_to_mavl`)
Ошибки `rustc` вида `src\main.rs:12:5` преобразуются в `source/ИМЯ:MM`, вычитая количество строк шапки и учитывая маркеры файлов. Пользователь видит ошибки в своих `.mavl`, а не в сгенерированном Rust.

### Этап 5 — Сборка через cargo
`mavc` вызывает `cargo build --manifest-path intRs/Cargo.toml [--release]`. Готовый бинарник `mavapp.exe` копируется в `target/debug/` или `target/release/`.

---

## 4. MavLang → Rust: таблица соответствий

| MavLang | Генерируемый Rust | Примечание |
|---------|------------------|------------|
| `fn main() {...}` | `fn main() {...}` | проходит транзитом |
| `let x: i32 = 5;` | `let x: i32 = 5;` | транзит |
| `let mut y = 10;` | `let mut y = 10;` | транзит |
| `print(x);` | `println!("{}", x);` | `x` не строка |
| `print("hi");` | `println!("hi");` | строковый литерал |
| `print(dir("."));` | `println!("{:?}", dir("."));` | вектор — `{:?}` |
| `input("промпт")` | блок с `read_line`, `flush`, `trim` | см. исходник компилятора |
| `read_file("f.txt")` | `std::fs::read_to_string("f.txt").unwrap()` | |
| `write_file("f.txt", d)` | `std::fs::write("f.txt", d).unwrap()` | |
| `append_file("f.txt", d)` | `OpenOptions::new().append(true).open(...).write_all(d.as_bytes())` | |
| `split(s, ",")` | `s.split(",").map(|x| x.to_string()).collect::<Vec<String>>()` | |
| `lines(s)` | `s.lines().map(...).collect::<Vec<String>>()` | |
| `join(v, ",")` | `v.join(",")` | |
| `trim(s)` | `s.trim().to_string()` | |
| `upper(s)` | `s.to_uppercase().to_string()` | |
| `lower(s)` | `s.to_lowercase().to_string()` | |
| `capitalize(s)` | блок: первая буква `to_uppercase()` + остальное | |
| `to_int(s)` | `s.trim().parse::<i32>().unwrap()` | |
| `sleep(100)` | `std::thread::sleep(Duration::from_millis(100))` | |
| `args()` | `std::env::args().collect::<Vec<String>>()` | |
| `arg(1)` | `std::env::args().nth(1 as usize).unwrap_or_default()` | |
| `contains(s, "x")` | `s.contains("x")` | |
| `check(c)` | `assert!(c)` | |
| `env("PATH")` | `std::env::var("PATH").unwrap_or_else(|_| String::new())` | |
| `exists("f.txt")` | `std::path::Path::new("f.txt").exists()` | |
| `time_ms()` | `SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64` | |
| `dir(".")` | `read_dir(".").map(...).collect::<Vec<String>>()` | |

---

## 5. LSP-сервер (`mavc lsp`)

Реализован в `compiler/src/lsp.rs`. Общается по JSON-RPC через stdin/stdout.

### Поддерживаемые методы:
- `initialize` — возможности сервера: `textDocumentSync`, `completionProvider` (триггеры: `p`, `f`, `l`, `g`, `:`), `hoverProvider`.
- `textDocument/didOpen` — отправка диагностики при открытии файла.
- `textDocument/didChange` — пересчёт диагностики при изменении.
- `textDocument/completion` — список ключевых слов и встроенных функций со сниппетами (`fn`, `let`, `if`, `for`, `print`, `input`, `len`, `read_file`, `write_file`, `append_file`, `split`, `lines`, `join`, `trim`, `upper`, `lower`, `capitalize`, `to_int`, `sleep`, `args`, `arg`, `env`, `exists`, `time_ms`, `dir`, `contains`, `check`, `gui`, `gui::window`).
- `textDocument/hover` — markdown-подсказка по ключевому слову или функции.
- `shutdown` / `exit` — завершение.

### Диагностика:
- Отсутствие `fn main() {...}` — ошибка severity 1.
- Дисбаланс `{ }` — ошибка severity 1.
- `print(` без `;` в конце строки — hint severity 4.

---

## 6. `mavc watch`

Режим автосборки. Проверяет время модификации файлов в `source/` каждые 500 мс. При изменении любого `.mavl` запускает полную пересборку (`do_build`). Выводит статус в консоль. Останавливается по `Ctrl+C`.

---

## 7. Зависимости

- **Для сборки компилятора**: Rust toolchain (cargo, rustc).
- **Для сборки пользовательских программ**: Rust toolchain + `cargo`.
- **Зависимости компилятора**: `serde`, `serde_json` (только для LSP).
- **Зависимости пользовательских программ**: любые, указанные в `mavlang.toml` (`[deps]`).

---

## 8. Ограничения v0.5

- Нет своего полноценного borrow-checker (делегируется `rustc`).
- Нет своего кодогенератора (код génèreтся в Rust, а не в машинный код напрямую).
- Один файл `main.mavl` обязателен (точка входа).
- Нет модулей `mod` и системы крейтов внутри языка (крейты только через `mavlang.toml`).
- `println!` в исходном `.mavl` проходит как есть (не конфликтует с `print()`).
- `gui::window` — заглушка, реального GUI нет.

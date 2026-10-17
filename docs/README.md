# MavLang — маленький быстрый язык (.mavl)

MavLang v0.1 — учебный low-level язык в духе Rust / Zig.
Пишешь простой код в `source/*.mavl`, команда `mavc` превращает его в Rust (`intRs/`) и собирает через `cargo`/`rustc` в настоящий `.exe`. Снаружи выглядит как свой компилятор: ошибки показывают строки твоего `.mavl`, а не сгенерённого Rust.

## Что на нём уже можно сделать

v0.1 это почти 1:1 подмножество Rust + сахар `print`, поэтому работает всё базовое:

- консольные проги: математика, `print`, ввод через `std::io`
- `let / mut`, типы `i32 u32 f64 bool String &str Vec`, функции `fn`
- `if / else`, `while`, `for`, `match`
- структуры `struct`, `enum`, методы `impl`
- файлы, args, время через `std::`
- тесты логики: borrow checker rustc ловит `& / &mut` ошибки и показывает их как MavLang-ошибки

Пример — угадайка за 15 строк, файловый поиск, бенч цикла. Async — в roadmap, GUI пока заглушка `gui::window` (см. ниже), крейты — через `mavlang.toml`.

Чего пока нет: настоящего своего borrow checker и арены — флаг `--mem` пока только меняет шапку генерации.

## Установка за минуту

Нужны `cargo + rustc` (уже стоят) и сам `mavc`:

```powershell
cargo install --path compiler
mavc check
```

Теперь `mavc` в PATH (`C:\Users\admin\.cargo\bin\mavc.exe`).

## Старт за 30 секунд

```powershell
# 1. пишешь source/main.mavl
# 2. проверка / запуск / сборка:
mavc check
mavc run
mavc build --release --mem=borrow
# 3. забираешь target/debug/mavapp.exe или target/release/mavapp.exe
```

## Раскладка папок

```
myapp/
  source/main.mavl   <- пишешь сюда (v0.2: можно несколько *.mavl, склеятся)
  source/util.mavl   <- второй файл: main.mavl первым, остальные по алфавиту
  mavlang.toml       <- (v0.2, опционально) зависимости крейтов, см. CLI ниже
  intRs/             <- вся раст-кухня: Cargo.toml, src/main.rs. Руками не трогать
  target/debug/      <- готовые .exe/.dll (debug)
  target/release/    <- то же после --release
  compiler/          <- исходники самого mavc на Rust
  docs/ LLMS.txt     <- доки для людей и для ИИ
```

## CLI

```
mavc check [--mem=borrow|arena|manual]
mavc build [--release] [--mem=...]
mavc run   [--release] [--mem=...]
mavc clean
```

`--mem` в v0.1:
- `borrow` — обычные `& / &mut`, проверки rustc (дефолт)
- `arena` — заглушка под будущий bump-аллокатор
- `manual` — можно `unsafe` + сырые указатели

## Стандартные функции (v0.3)

Без импортов, работают из коробки (транспилляция в `std`):

```rust
fn main() {
    let name = input("name? ");   // ввод строки из stdin
    print(name);
    print(len(name));             // длина
    print(trim("  hi  "));        // "hi"
    print(upper("abc"));          // "ABC"
    print(lower("ABC"));          // "abc"
    print(to_int("42") + 8);      // 50
    let parts = split("a,b,c", ","); // Vec<String>
    write_file("out.txt", "hi");  // запись
    print(read_file("out.txt"));  // чтение
    print(len(args()));           // аргументы командной строки
    sleep(100);                   // пауза 100 мс
    print(arg(1));                // 1-й аргумент или ""
    append_file("out.txt", "x\n");// дописать в файл
    print(join(split("a,b", ","), "-")); // "a-b"
    if contains("abc", "b") {     // true
        print("has-b");
    }
    check(1 + 1 == 2);            // assert, для тестов
    print(capitalize("hi"));      // "Hi"
}
```

Вложенность работает: `print(trim(s))`, `print(len(split(s, ",")))`.

## Несколько файлов (v0.2)

Все `source/*.mavl` конкатенируются в один `intRs/src/main.rs`:
`main.mavl` первым, остальные по алфавиту, каждый с комментом `// <file>`.
`mavc check` проверяет все файлы сразу (склейку целиком).

```rust
// source/main.mavl
fn main() {
    let h = helper();
    print(h);
}

// source/util.mavl
fn helper() -> i32 {
    return 42;
}
```

## Зависимости крейтов (v0.2)

Файл `mavlang.toml` в корне проекта:

```toml
[deps]
egui = "0.28"
serde = "1"
```

`mavc build` читает его и мержит `[dependencies]` в `intRs/Cargo.toml`
(создаёт если нет, `name/edition` не затирает, существующие записи обновляет).
Если файла нет — как раньше, без зависимостей.

## GUI-заглушка (v0.2)

Встроенный мод `gui` — пока только стаб под будущий `egui`:

```rust
// source/main.mavl
fn main() {
    gui::window("hi");
}
```

транспилируется в:

```rust
println!("[gui-stub] {}", "hi"); // TODO real gui
```

LSP знает `gui` / `gui::window` (completion + hover).

## Синтаксис v0.1

```rust
// source/main.mavl
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let x: i32 = 5;
    let mut y = 10;
    y = y + add(x, 2);
    if y > 10 {
        print(y);          // -> println!("{}", y)
    } else {
        print("small");
    }
    for i in 0..3 {
        print(i);
    }
}
```

Правила:
- `fn main() {...}` обязателен
- `print(x);` и `print("hi");` — сахар над `println!`
- остальное — обычный Rust: точки с запятой, `{ }`, `// комменты`
- одна строка `.mavl` = одна строка `.rs` (поэтому ошибки маппятся точно)

## Как это работает внутри

`compiler/src/main.rs` читает все `.mavl` (`collect_sources`: `main.mavl` первым,
остальные по алфавиту), `transpile_line` меняет `print()` / `gui::window()` и клеит
шапку режима + `// <file>` маркеры. `compiler/src/main.rs` пишет `intRs/src/main.rs`,
мержит `mavlang.toml [deps]` в `intRs/Cargo.toml`, зовёт `cargo build`, копирует
`mavapp.exe/.dll` наверх и переписывает `main.rs:12` в `source/<file>:MM` по карте маркеров.

Хочешь посмотреть кухню — открой `intRs/src/main.rs` после билда.

## Roadmap

- [ ] подсветка VS Code для `.mavl`
- [x] v0.2: несколько файлов + `mod` (конкатенация, `main.mavl` первым) — done
- [x] v0.2: зависимости крейтов из `mavlang.toml` — done
- [ ] настоящий `--mem=arena` и мини-borrow checker
- [x] v0.2: `gui` обёртка-заглушка (`gui::window` → `println! [gui-stub]`) — done; `async` сахар — TODO
- [ ] датасет + дообучение мелкой модели (для тестов и тиктока)

## Для ИИ

Короткая спека для файнтюна: `LLMS.txt` в корне. Полные примеры — в `source/`.

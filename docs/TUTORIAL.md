# MavLang — пошаговое руководство (TUTORIAL)

Пройдём от установки до первой программы на MavLang. Все команды выполняются в PowerShell из корня проекта `C:\1\аап`.

---

## Шаг 0. Что нужно

На вашем компьютере уже установлены:
- **Rust toolchain** (`cargo`, `rustc`) — для сборки компилятора и программ.
- **Python 3.12** с venv `ai/.venv` — только если хотите заниматься дообучением нейросети (трек AI). Для обычной разработки на MavLang Python не нужен.

---

## Шаг 1. Установка компилятора `mavc`

Компилятор написан на Rust. Один раз установите его в PATH:

```powershell
cd C:\1\аап
cargo install --path compiler
```

После этого `mavc` доступен из любой папки. Проверьте:

```powershell
mavc --help
```

Должна напечататься справка с командами: `check`, `build`, `run`, `clean`, `lsp`, `watch`.

---

## Шаг 2. Создайте первую программу

Папка `source/` — место для ваших исходников `.mavl`. Обязателен файл `source/main.mavl` с точкой входа `fn main()`.

Создайте файл `source/main.mavl`:

```mavl
// Мой первый файл на MavLang
fn main() {
    let x: i32 = 5;
    let mut y = 10;
    y = y + x;
    print(y);
    print("привет, MavLang!");
}
```

---

## Шаг 3. Проверка и сборка

### Проверка синтаксиса (без сборки)
```powershell
mavc check
```
Ожидаемый вывод:
```
[mavc] check ok: source/main.mavl (1 files)
```

### Сборка в .exe
```powershell
mavc build
```
Ожидаемый вывод:
```
[mavc] ok (borrow, debug): mavapp.exe, mavapp.pdb -> target/debug/
```

Готовый бинарник лежит в `target/debug/mavapp.exe`.

### Сборка и запуск одной командой
```powershell
mavc run
```
Ожидаемый вывод:
```
[mavc] ok (borrow, debug): mavapp.exe, mavapp.pdb -> target/debug/
15
привет, MavLang!
```

### Релизная сборка (быстрее, меньше отладочной информации)
```powershell
mavc build --release
```
Бинарник появится в `target/release/mavapp.exe`.

---

## Шаг 4. Режимы памяти (`--mem`)

MavLang поддерживает три режима работы с памятью:

```powershell
mavc build --mem=borrow    # обычные ссылки & / &mut (по умолчанию)
mavc build --mem=manual    # unsafe, сырые указатели *mut T
mavc build --mem=arena     # заглушка под будущий bump-аллокатор
```

Пример с `manual`:
```mavl
fn main() {
    let mut arr = [10, 20, 30];
    let p: *mut i32 = arr.as_mut_ptr();
    unsafe {
        *p.offset(1) = 99;
    }
    print(arr[1]);  // выведет 99
}
```

---

## Шаг 5. Работа с файлами

MavLang может читать и писать файлы без импортов:

```mavl
fn main() {
    write_file("дневник.txt", "первая строка\n");
    append_file("дневник.txt", "вторая строка\n");
    let содержимое = read_file("дневник.txt");
    print(содержимое);
}
```

Запустите `mavc run` — программа создаст файл `дневник.txt` и распечатает его содержимое.

---

## Шаг 6. Работа со строками

```mavl
fn main() {
    let s = "  MavLang  ";
    print(trim(s));           // "MavLang"
    print(upper(s));          // "  MAVLANG  "
    print(lower(s));          // "  mavlang  "
    print(capitalize(s));     // "  mavlang  " (первый не-пробел — M, остальное как есть)
    print(len(s));            // 12
    
    let части = split("а,б,в", ",");
    print(len(части));        // 3
    print(join(части, "-"));  // а-б-в
    
    if contains("abcdef", "cd") {
        print("есть");
    }
}
```

---

## Шаг 7. Ввод с клавиатуры и аргументы

```mavl
fn main() {
    let имя = input("как вас зовут? ");
    print("привет, ");
    print(имя);
    
    let аргументы = args();
    print(len(аргументы));
    
    if len(аргументы) > 1 {
        print(arg(1));
    } else {
        print("аргументов нет");
    }
}
```

Запустите с аргументом:
```powershell
mavc run
# в консоли программы введите имя при запросе
```

Или передайте аргументы при запуске через скомпилированный exe напрямую:
```powershell
target/debug/mavapp.exe любое_слово
```

---

## Шаг 7.5. Автосборка при разработке (`watch`)

Во время разработки удобно не запускать `mavc build` вручную после каждого сохранения. Режим `watch` следит за папкой `source/` и пересобирает автоматически:

```powershell
mavc watch
```

Откройте `source/main.mavl` в редакторе, сохраните — в окне с `watch` увидите:
```
[watch] изменение в source/, пересобираю...
[mavc] ok (borrow, debug): mavapp.exe, mavapp.pdb -> target/debug/
[watch] сборка ок
```

Остановите `watch` по `Ctrl+C`.

---

## Шаг 8. Несколько файлов

MavLang склеивает все `source/*.mavl` в одну программу. `main.mavl` — первым, остальные по алфавиту.

**source/main.mavl:**
```mavl
fn main() {
    print(вспомогательная());
}
```

**source/helper.mavl:**
```mavl
fn вспомогательная() -> i32 {
    return 42;
}
```

Запустите `mavc run` — выведет `42`. Ошибки компиляции будут показывать с правильным именем файла (например, `source/helper.mavl:3`).

---

## Шаг 9. Зависимости крейтов (`mavlang.toml`)

Если нужны внешние библиотеки Rust (например, `egui` для GUI, `serde` для сериализации), создайте файл `mavlang.toml` в корне проекта:

```toml
[deps]
egui = "0.28"
serde = "1"
```

Затем `mavc build` автоматически добавит их в `intRs/Cargo.toml` в секцию `[dependencies]`. Теперь в `.mavl` можно использовать типы и функции из этих крейтов (передавая их как есть в Rust-код).

---

## Шаг 10. VS Code — подсветка и LSP

Папка `vscode-mavlang/` содержит расширение для Visual Studio Code.

**Установка:**
1. Скопируйте папку `vscode-mavlang` в `%USERPROFILE%\.vscode\extensions\mavlang-0.1.0\`.
2. Перезапустите VS Code.
3. Откройте любой `.mavl` — в правом углу появится надпись `MavLang`, текст окрасится.

**Что работает:**
- Подсветка ключевых слов (`fn`, `let`, `if`, `for` и т.д.), типов (`i32`, `String`, `Vec`), встроенных функций (`print`, `input`, `len` и др.), чисел, строк, комментариев.
- LSP-сервер `mavc lsp` — диагностика ошибок, автодополнение (по `Ctrl+Space`), подсказки при наведении (`hover`).

---

## Шаг 11. Полная очистка

Удалить все промежуточные и готовые файлы сборки:

```powershell
mavc clean
```

---

## Шаг 12. Первая полноценная программа (пример)

Создайте `source/main.mavl`:

```mavl
// Калькулятор на MavLang
fn добавить(a: i32, b: i32) -> i32 {
    return a + b;
}

fn умножить(a: i32, b: i32) -> i32 {
    return a * b;
}

fn main() {
    let x: i32 = 10;
    let y: i32 = 5;
    
    print("x + y = ");
    print(добавить(x, y));
    
    print("x * y = ");
    print(умножить(x, y));
    
    let стемка = input("введите строку для проверки: ");
    if contains(стемка, "мав") {
        print("нашли 'мав'!");
    } else {
        print("не нашли");
    }
    
    sleep(500);
    print("завершено");
}
```

Запустите:
```powershell
mavc run
```

---

## Шаг 13. Удаление всего

```powershell
mavc clean
```

Удалит `intRs/target`, `target/debug`, `target/release`.

---

## Быстрая шпаргалка команд

| Команда | Что делает |
|---------|-----------|
| `mavc check` | Проверить синтаксис без сборки |
| `mavc build` | Собрать `.exe` в `target/debug/` |
| `mavc build --release` | Собрать оптимизированный `.exe` в `target/release/` |
| `mavc run` | Собрать и запустить |
| `mavc run --release` | Собратьrelease и запустить |
| `mavc clean` | Удалить все файлы сборки |
| `mavc watch` | Автосборка при изменении `source/` |
| `mavc lsp` | Запустить LSP-сервер для редакторов |

---

## Дальше

- **docs/SPEC.md** — полная спецификация языка.
- **docs/ARCHITECTURE.md** — как устроен компилятор внутри.
- **docs/EXAMPLES.md** — коллекция готовых примеров.
- **docs/README.md** — краткий обзор и roadmap.
- **LLMS.md** — документация для нейросетей (дообучение под MavLang).

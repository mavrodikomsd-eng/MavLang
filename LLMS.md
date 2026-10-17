# MavLang — LLMS documentation (для нейросетей)

Краткая, машинноориентированная спецификация языка MavLang v0.5.  
Цель: дать ИИ достаточно информации, чтобы генерировать валидный код, который проходит `mavc build+run`.

---

## 1. Структура файла

- Расширение: `.mavl`
- Обязательная точка входа: `fn main() {...}`
- Каждый стейтмент заканчивается `;`
- Блоки `{ }` — без `;` после閉вающей скобки
- Комментарии: `// ...`
- Одна строка `.mavl` ≈ одна строка сгенерированного `.rs`

Пример минимальной программы:
```mavl
fn main() {
    let x: i32 = 5;
    print(x);
}
```

---

## 2. Ключевые слова (не использовать как имена)

`fn let mut if else while for in return struct enum impl match use pub const unsafe as is loop break continue true false borrow arena manual`

---

## 3. Типы

`i32 u32 i64 u64 f32 f64 bool char String str Vec`

Объявление переменной:
```mavl
let x: i32 = 5;
let mut y = 10;  // тип выводится
```

Объявление функции:
```mavl
fn имя(параметр: тип) -> возвращаемый_тип {
    return значение;
}
```

---

## 4. Встроенные функции (без импортов)

| Функция | Сигнатура | Возвращает | Пример |
|---------|-----------|------------|--------|
| `print(x)` | `print(any)` | — | `print(x); print("hi");` |
| `input(prompt)` | `input(String)` | `String` | `let n = input("имя? ");` |
| `len(x)` | `len(String\|Vec)` | `usize` | `len(s); len(v);` |
| `read_file(path)` | `read_file(String)` | `String` | `read_file("a.txt")` |
| `write_file(path, data)` | `write_file(String, String)` | — | `write_file("o.txt", "hi");` |
| `append_file(path, data)` | `append_file(String, String)` | — | `append_file("log.txt", "x\n");` |
| `split(s, delim)` | `split(String, String)` | `Vec<String>` | `split("a,b", ",")` |
| `lines(s)` | `lines(String)` | `Vec<String>` | `lines(read_file("a.txt"))` |
| `join(v, delim)` | `join(Vec<String>, String)` | `String` | `join(v, ",")` |
| `trim(s)` | `trim(String)` | `String` | `trim("  hi  ")` |
| `upper(s)` | `upper(String)` | `String` | `upper("abc")` → `"ABC"` |
| `lower(s)` | `lower(String)` | `String` | `lower("ABC")` → `"abc"` |
| `capitalize(s)` | `capitalize(String)` | `String` | `capitalize("hi")` → `"Hi"` |
| `to_int(s)` | `to_int(String)` | `i32` | `to_int("42")` |
| `sleep(ms)` | `sleep(i32)` | — | `sleep(100);` |
| `args()` | `args()` | `Vec<String>` | `args()` |
| `arg(i)` | `arg(i32)` | `String` | `arg(1);` (0 = имя программы) |
| `contains(s, sub)` | `contains(String, String)` | `bool` | `contains("abc", "b")` |
| `check(cond)` | `check(bool)` | — | `check(1 + 1 == 2);` |
| `env(name)` | `env(String)` | `String` | `env("PATH")` |
| `exists(path)` | `exists(String)` | `bool` | `exists("a.txt")` |
| `time_ms()` | `time_ms()` | `i64` | `time_ms()` |
| `dir(path)` | `dir(String)` | `Vec<String>` | `dir(".")` |
| `gui::window(t)` | `gui::window(String)` | — | `gui::window("hi");` (стаб) |

### Правило вывода в print()
- Скаляры (`i32`, `bool`, `String`): `print(x);` → `println!("{}", x)`
- Векторы (`Vec<String>` из `split`, `lines`, `args`, `dir`): `print(v);` → `println!("{:?}", v)`

---

## 5. Управление потоком

```mavl
if условие {
    тело;
} else {
    тело;
}

while условие {
    тело;
}

for i in 0..3 {
    тело;
}

match переменная {
    Шаблон => тело,
    _ => тело,
}

return значение;
```

---

## 6. Структуры и перечисления

```mavl
struct Точка {
    x: i32,
    y: i32,
}

enum Направление {
    Вверх,
    Вниз,
}

impl Точка {
    fn новая() -> Точка {
        return Точка { x: 0, y: 0 };
    }
}
```

Доступ к полю: `p.x`, вызов метода: `c.increment()`.

---

## 7. Режимы памяти (`--mem`)

| Режим | Флаг | Что разрешено |
|-------|------|--------------|
| borrow | `--mem=borrow` (default) | `& / &mut`, проверки rustc |
| manual | `--mem=manual` | `unsafe`, `*mut T`, `*const T` |
| arena | `--mem=arena` | заглушка (пока не работает) |

Пример unsafe:
```mavl
fn main() {
    let mut a = [10, 20, 30];
    let p: *mut i32 = a.as_mut_ptr();
    unsafe {
        *p.offset(1) = 99;
    }
    print(a[1]);
}
```

---

## 8. Ограничения (важно для ИИ)

1. **Обязателен `fn main() {...}`** — без него ошибка компиляции.
2. **Не объявляйте собственные функции с именами встроенных** (`fn print(...)`, `fn len(...)`) — транспилер не трогает `fn` с такими именами, но это может привести к неочевидному поведению.
3. **Не используйте `println!` в исходном `.mavl`** — оно пройдёт транзитом, но используйте `print()`.
4. **Точка с запятой `;` обязательна** в конце стейтментов (кроме `{ }` блоков).
5. **Несколько файлов**: все `source/*.mavl` склеиваются. `main.mavl` первым.
6. **`mavlang.toml`** в корне: `[deps] имя_крейта = "версия"` — добавляет зависимости в `intRs/Cargo.toml`.

---

## 9. Чеклист валидности (проверить перед генерацией)

- [ ] Есть `fn main() {...}`
- [ ] Каждый стейтмент заканчивается на `;` (кроме `{ }` блоков)
- [ ] Встроенные функции вызываются с правильным числом аргументов
- print() с вектором → использует `{:?}` автоматически (не нужно задумываться)
- Если используется `unsafe` — флаг `--mem=manual`
- Строковые литералы в двойных кавычках `"..."`

---

## 10. Примеры валидного кода

**Минимум:**
```mavl
fn main() {
    print("ok");
}
```

**С функциями и типами:**
```mavl
fn квадрат(n: i32) -> i32 {
    return n * n;
}

fn main() {
    let x: i32 = 5;
    print(квадрат(x));
}
```

**Со строками и файлами:**
```mavl
fn main() {
    write_file("out.txt", "привет\n");
    let с = read_file("out.txt");
    print(trim(с));
    print(len(с));
}
```

**Со встроенными функциями векторов:**
```mavl
fn main() {
    let части = split("а,б,в", ",");
    print(join(части, "-"));
    print(len(args()));
}
```

---

## 11. Частые ошибки ИИ (избегать)

- ❌ `let x = 5` без `fn main` — ошибка, нужна точка входа
- ❌ `print(x)` без `;` — может не собраться
- ❌ `os.Args`, `File::open`, `read_line` — это Rust, не MavLang. Используйте `args()`, `read_file()`, `input()`.
- ❌ capitalized именованные аргументы — MavLang использует snake_case для встроенных функций
- ❌ Забыть `mut` для изменяемой переменной — ошибка borrow-checker
- ❌ Вызывать `trim`, `upper`, `lower`, `capitalize`, `to_int`, `len` как методы (`.trim()`) — в MavLang это функции `trim(s)`, а не методы (хотя Rust-транзит позволяет и то, и другое, лучше использовать функциональный стиль)

---

## 12. Команды компилятора

```
mavc check                 # проверить синтаксис
mavc build                 # собрать target/debug/mavapp.exe
mavc build --release       # собрать target/release/mavapp.exe
mavc run                   # собрать и запустить
mavc clean                 # удалить сборки
mavc watch                 # автосборка при изменении source/
```

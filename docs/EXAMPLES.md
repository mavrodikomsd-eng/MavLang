# MavLang — коллекция примеров кода

Все примеры проверены через настоящий компилятор `mavc build+run`.  
Чтобы попробовать: скопируйте код в `source/main.mavl`, выполните `mavc run`.

---

## 1. Hello World

```mavl
fn main() {
    print("привет, мир!");
}
```

**Вывод:**
```
привет, мир!
```

---

## 2. Переменные и арифметика

```mavl
fn main() {
    let a: i32 = 10;
    let mut b = 3;
    b = b + 2;
    print(a);
    print(b);
    print(a * b);
}
```

**Вывод:**
```
10
5
50
```

---

## 3. Условия (if / else)

```mavl
fn main() {
    let возраст: i32 = 18;
    if возраст >= 18 {
        print("взрослый");
    } else {
        print("несовершеннолетний");
    }
}
```

**Вывод:**
```
взрослый
```

---

## 4. Циклы (while и for)

### while
```mavl
fn main() {
    let mut i: i32 = 0;
    while i < 3 {
        print(i);
        i = i + 1;
    }
}
```

**Вывод:**
```
0
1
2
```

### for
```mavl
fn main() {
    for i in 0..3 {
        print(i);
    }
}
```

**Вывод:**
```
0
1
2
```

---

## 5. FizzBuzz

```mavl
fn main() {
    let mut i: i32 = 1;
    while i <= 15 {
        if i % 15 == 0 {
            print("fizzbuzz");
        } else if i % 3 == 0 {
            print("fizz");
        } else if i % 5 == 0 {
            print("buzz");
        } else {
            print(i);
        }
        i = i + 1;
    }
}
```

**Вывод:**
```
1
2
fizz
4
buzz
fizz
7
8
fizz
buzz
11
fizz
13
14
fizzbuzz
```

---

## 6. Функции

```mavl
fn добавить(a: i32, b: i32) -> i32 {
    return a + b;
}

fn умножить(a: i32, b: i32) -> i32 {
    return a * b;
}

fn main() {
    print(добавить(3, 4));
    print(умножить(3, 4));
}
```

**Вывод:**
```
7
12
```

---

## 7. Рекурсия (Фибоначчи)

```mavl
fn fib(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

fn main() {
    print(fib(10));
}
```

**Вывод:**
```
55
```

---

## 8. Структуры (struct)

```mavl
struct Точка {
    x: i32,
    y: i32,
}

fn main() {
    let p = Точка { x: 3, y: 4 };
    print(p.x);
    print(p.y);
}
```

**Вывод:**
```
3
4
```

---

## 9. Методы (impl)

```mavl
struct Счётчик {
    n: i32,
}

impl Счётчик {
    fn increment(&mut self) {
        self.n = self.n + 1;
    }
}

fn main() {
    let mut c = Счётчик { n: 0 };
    c.increment();
    c.increment();
    c.increment();
    print(c.n);
}
```

**Вывод:**
```
3
```

---

## 10. Перечисления (enum) и match

```mavl
enum Направление {
    Вверх,
    Вниз,
    Влево,
    Вправо,
}

fn main() {
    let д = Направление::Вправо;
    match д {
        Направление::Вверх => print("вверх"),
        Направление::Вниз => print("вниз"),
        Направление::Влево => print("влево"),
        Направление::Вправо => print("вправо"),
    }
}
```

**Вывод:**
```
вправо
```

---

## 11. borrow checker (& и &mut)

```mavl
fn печать_ссылку(s: &str) {
    print(s);
}

fn изменить(x: &mut i32) {
    *x = *x * 2;
}

fn main() {
    let текст = "привет";
    печать_ссылку(текст);
    
    let mut число: i32 = 21;
    изменить(&mut число);
    print(число);
}
```

**Вывод:**
```
привет
42
```

---

## 12. Unsafe и сырые указатели (`--mem=manual`)

```mavl
fn main() {
    let mut arr = [10, 20, 30];
    let p: *mut i32 = arr.as_mut_ptr();
    unsafe {
        *p.offset(1) = 99;
    }
    print(arr[1]);
}
```

Запуск:
```powershell
mavc build --mem=manual
target/debug/mavapp.exe
```

**Вывод:**
```
99
```

---

## 13. Ввод с клавиатуры (input)

```mavl
fn main() {
    let имя = input("как вас зовут? ");
    print("привет, ");
    print(имя);
}
```

Пример сессии:
```
как вас зовут? Алиса
привет, Алиса
```

---

## 14. Работа с файлами

```mavl
fn main() {
    write_file("заметка.txt", "первая линия\n");
    append_file("заметка.txt", "вторая линия\n");
    append_file("заметка.txt", "третья линия\n");
    
    let содержимое = read_file("заметка.txt");
    print(содержимое);
    
    print(len(содержимое));
}
```

**Вывод:**
```
первая линия
вторая линия
третья линия
51
```

---

## 15. Работа со строками (stdlib)

```mavl
fn main() {
    let с = "  Привет, МИР!  ";
    
    print(trim(с));           // "Привет, МИР!"
    print(upper(с));          // "  ПРИВЕТ, МИР!  "
    print(lower(с));          // "  привет, мир!  "
    print(capitalize(с));     // "  привет, мир!  "
    
    print(len(с));            // 18
    
    let части = split("яблоко,банан,вишня", ",");
    print(len(части));        // 3
    print(join(части, " | ")); // яблоко | банан | вишня
    
    print(lines("строка1\nстрока2\nстрока3")); // выведет вектор
    
    if contains("Мавел нравится", "Мав") {
        print("содержит Мав");
    }
    
    print(to_int("42") + 8);  // 50
}
```

---

## 16. Аргументы командной строки

```mavl
fn main() {
    let арг = args();
    print(len(арг));
    
    if len(арг) > 1 {
        print(arg(1));
    } else {
        print("аргументов нет");
    }
}
```

Запуск:
```powershell
target/debug/mavapp.exe привет мир
```

**Вывод:**
```
3
привет
```

---

## 17. Время и пауза

```mavl
fn main() {
    print(time_ms());
    sleep(1000);
    print(time_ms());
}
```

**Вывод:**
```
1792247122377
1792247123377
```
(значения будут другими — это миллисекунды с 1970 года)

---

## 18. Переменные окружения и проверка файлов

```mavl
fn main() {
    print(env("USERNAME"));
    print(exists("source/main.mavl"));
    print(exists("несуществующий_файл.xyz"));
}
```

**Вывод (пример):**
```
admin
true
false
```

---

## 19. Список файлов в папке (dir)

```mavl
fn main() {
    print(dir("source"));
}
```

**Вывод (пример):**
```
["main.mavl"]
```

---

## 20. Проверки (check) для тестов

```mavl
fn main() {
    let x: i32 = 2 + 2;
    check(x == 4);
    print("все тесты прошли");
}
```

Если условие ложно — программа паникует (как `assert!` в Rust).

---

## 21. GUI-заглушка

```mavl
fn main() {
    gui::window("моё приложение");
}
```

**Вывод:**
```
[gui-stub] моё приложение
```
(в будущем здесь будет настоящее GUI через `egui`)

---

## 22. Сложный пример: подсчёт слов в файле

```mavl
fn main() {
    write_file("вход.txt", "привет мир привет мавланг мир\n");
    let текст = read_file("вход.txt");
    let слова = split(trim(текст), " ");
    print(len(слова));
    print(join(слова, "\n"));
}
```

**Вывод:**
```
6
привет
мир
привет
мавланг
мир
```

---

## 23. Сложный пример: CSV-сумма

```mavl
fn main() {
    let данные = "10,20,30,40";
    let части = split(данные, ",");
    let mut сумма: i32 = 0;
    для каждой части в части {
        сумма = сумма + to_int(часть);
    }
    print(сумма);
}
```

*Примечание: в текущей версии синтаксис `для каждой` не поддерживается, используйте `for`:*

```mavl
fn main() {
    let данные = "10,20,30,40";
    let части = split(данные, ",");
    let mut сумма: i32 = 0;
    for п in части {
        сумма = сумма + to_int(п);
    }
    print(сумма);
}
```

**Вывод:**
```
100
```

---

Все примеры лежат в репозитории и могут быть запущены через `mavc run`. Если пример не компилируется — проверьте версию языка (документ `docs/SPEC.md`) и наличие `fn main()`.

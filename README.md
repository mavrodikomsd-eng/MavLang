<div align="center">

# MavLang

**Свой язык программирования, который помещается в один экзешник.**

`.mavl` → Rust → нативный `.exe`. Компилятор — один бинарник **0.53 МБ**. Ни `node_modules`, ни менеджеров пакетов, ни экосистемы, которая тяжелее самого языка.

[![Release](https://img.shields.io/github/v/tag/mavrodikomsd-eng/MavLang?label=release&color=green)](https://github.com/mavrodikomsd-eng/MavLang/releases)
[![Compiler size](https://img.shields.io/badge/mavc.exe-0.53%20MB-blue)](https://github.com/mavrodikomsd-eng/MavLang/releases)
[![Stack](https://img.shields.io/badge/stack-pure%20Rust-orange)](compiler/)

</div>

---

## Зачем ещё один язык

| Больно | Что делает MavLang |
|---|---|
| Ради hello world ставишь инструменты на несколько ГБ | нужны только `rustup` и один exe |
| Ошибки сборки приходят изнутри сгенерированного кода | ошибки — **на строках твоего `.mavl`** |
| Экосистема мешает начать | нет пакетов: **24 builtin-функции вшиты в компилятор** |
| Управление памятью — либо GC, либо боль | три режима одним флагом: `--mem=auto \| manual \| arena` |

## Hello MavLang

```rust
fn main() {
    let s = "  hello world  ";
    print(upper(trim(s)));

    let v = split("a,b,c", ",");
    for i in 0..len(v) {
        print(v[i]);
    }

    write_file("out.txt", "собрано MavLang v0.5.0");
    print(read_file("out.txt"));
}
```

```powershell
mavc run                          # проверить, собрать и запустить одной командой
mavc build --release --mem=arena  # релизный exe с arena-режимом
```

## Как это работает

```
 source/main.mavl          intRs/main.rs              target/release/mavapp.exe
┌──────────────────┐     ┌─────────────────────┐     ┌──────────────────┐
│  твой код (.mavl) │ →  │  транспилер mavc     │ →  │  cargo/rustc      │ →   нативный exe
│  свои ошибки      │    │  генерирует Rust     │    │  оптимизация      │     ~200 КБ
└──────────────────┘     └─────────────────────┘     └──────────────────┘
```

Транспилер помнит, какая строка Rust из какой строки `.mavl` получена, и исправляет обратные ссылки при ошибках компиляции.

## Три режима памяти

| Флаг | Что делает | Кому |
|---|---|---|
| `--mem=auto` | mavc сам решает, где аллокации можно объединить | по умолчанию |
| `--mem=manual` | `let mut buf = bytes(n);` — памятью управляешь сам | кто пишет аллокаторы |
| `--mem=arena` | арена: всё живёт в сегменте и сбрасывается разом | парсеры, игры |

## Бенч: язык, которого нет в интернете

Модели не видели `.mavl` в обучении — знание о языке в них отсутствует. Проверка честная: не текстовое сходство, а реальный `mavc build` и запуск бинарника на 16 задачах.

| Участник | Задачи решены |
|---|---|
| Qwen 0.5B (база, без обучения) | **0 / 16** |
| Qwen 0.5B + LoRA, 15 минут на RTX 4060 | **10 / 16** |
| Claude Haiku 4.5 (zero-shot, со спекой LLMS.md) | **10 / 16** |
| Claude Haiku 5.5 (zero-shot, прогон за $0.007) | **16 / 16** |

Вывод: 252 примера в датасете, 15 минут LoRA — и крошка 0.5B держит уровень frontier zero-shot. Датасет, скрипты бенча и предикты — в [ai/](ai/), спека для ИИ — [LLMS.md](LLMS.md).

## Установка

**Вариант 1 — готовый exe (0.53 МБ):**
скачай [`mavc-windows-x64.exe`](https://github.com/mavrodikomsd-eng/MavLang/releases) из Releases, переименуй в `mavc.exe`, положи в PATH.

**Вариант 2 — из исходников:**
```powershell
git clone https://github.com/mavrodikomsd-eng/MavLang.git
cd MavLang
cargo install --path compiler
```

Требуется только Rust toolchain (`rustup`).

## Состав репо

| Папка | Что там |
|---|---|
| `compiler/` | транспилер mavc (чистый Rust) |
| `examples/` | рабочие примеры языка |
| `vscode-mavlang/` | расширение VS Code: подсветка, LSP |
| `ai/` | датасет (252 примера), LoRA-скрипты, эталоны бенча |
| `docs/` | [SPEC](docs/SPEC.md) · [REFERENCE](docs/REFERENCE.md) · [TUTORIAL](docs/TUTORIAL.md) · [ARCHITECTURE](docs/ARCHITECTURE.md) |

## Что язык не обещает

MavLang v0.5.0 — честный маленький язык: без перегрузки операторов, без макросов, только то, что вшито. Типичные ловушки (например, `res = res + &s` вместо `String + String` в цикле) задокументированы в [LLMS.md, раздел 8](LLMS.md).

## Roadmap

- [ ] GGUF Q4_K_M — сжатая модель для локальных прогонов без GPU
- [ ] многоразовые arena-сегменты в `--mem=arena`
- [ ] модули и внешние пакеты вместо вшитого stdlib

---

<div align="center">

**Если идея зацепила — расскажи технарям. Весь конвейер открыт:**

https://github.com/mavrodikomsd-eng/MavLang

</div>

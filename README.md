# MavLang (.mavl) — v0.1 работает

Пишешь `source/main.mavl` → `mavc run` → получаешь `target/debug/mavapp.exe`.
Компилятор на тру-Rust (`compiler/`), ставится в PATH через `cargo install --path compiler`.

Полная дока: `docs/README.md`. Короткая спека для ИИ: `LLMS.txt`.

```powershell
mavc check
mavc run
mavc build --release --mem=borrow
```

// mavc lsp — мини rust-analyzer для MavLang.
// stdin/stdout JSON-RPC с Content-Length. Диагностика + completion + hover.
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufReader, Read, Write};

fn read_message(reader: &mut BufReader<io::StdinLock>) -> Option<Value> {
    let mut headers = HashMap::new();
    let mut buf = Vec::new();
    // читаем заголовки до \r\n\r\n побайтово чтобы не съесть тело
    let mut last4 = [0u8; 4];
    let mut n4 = 0;
    loop {
        let mut b = [0u8; 1];
        if reader.read_exact(&mut b).is_err() {
            return None;
        }
        buf.push(b[0]);
        last4[n4 % 4] = b[0];
        n4 += 1;
        if n4 >= 4 {
            let t = [last4[n4 % 4], last4[(n4 + 1) % 4], last4[(n4 + 2) % 4], last4[(n4 + 3) % 4]];
            if t == [b'\r', b'\n', b'\r', b'\n'] {
                break;
            }
        }
        if buf.len() > 65536 {
            return None;
        }
    }
    let head = String::from_utf8_lossy(&buf).to_string();
    for line in head.lines() {
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }
    let len: usize = headers.get("content-length")?.parse().ok()?;
    if len > 16 * 1024 * 1024 {
        return None;
    }
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}

fn send(val: &Value) {
    let s = serde_json::to_string(val).unwrap();
    let out = io::stdout();
    let mut o = out.lock();
    write!(o, "Content-Length: {}\r\n\r\n{}", s.len(), s).ok();
    o.flush().ok();
}

fn diagnostics_for(uri: &str, text: &str) -> Value {
    let mut diags = vec![];
    let push = &mut |line: usize, col_s: usize, col_e: usize, msg: &str, sev: i32| {
        diags.push(json!({
            "range": {
                "start": {"line": line, "character": col_s},
                "end": {"line": line, "character": col_e}
            },
            "severity": sev,
            "source": "mavc",
            "message": msg
        }));
    };
    if !text.contains("fn main") {
        push(0, 0, 5, "нужен fn main() {...} в .mavl", 1);
    }
    if text.matches('{').count() != text.matches('}').count() {
        push(0, 0, 5, "дисбаланс { } в .mavl", 1);
    }
    for (i, line) in text.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") {
            continue;
        }
        // подсказка: print без ; — в Rust у нас всё равно statement, мягкий hint
        if t.starts_with("print(") && !t.ends_with(';') && !t.ends_with('{') && !t.ends_with('}') {
            push(i, 0, line.len().min(20), "в MavLang statement обычно с ; в конце", 4);
        }
    }
    // пробуем полный транспилер для глубоких ошибок
    match crate::parse_and_transpile(text, "borrow") {
        Ok(_) => {}
        Err(e) => {
            push(0, 0, 5, &e, 1);
        }
    }
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": uri, "diagnostics": diags}
    })
}

fn completions() -> Vec<Value> {
    let kw = [
        ("fn", "fn name() {\n\t\n}"),
        ("let", "let x = "),
        ("let mut", "let mut x = "),
        ("if", "if  {\n\t\n}"),
        ("else", "else {\n\t\n}"),
        ("while", "while  {\n\t\n}"),
        ("for", "for i in 0.. {\n\t\n}"),
        ("match", "match  {\n\t\n}"),
        ("struct", "struct Name {\n\t\n}"),
        ("print", "print();"),
        ("input", "input(\"name? \");"),
        ("len", "len();"),
        ("read_file", "read_file(\"\");"),
        ("write_file", "write_file(\"\", );"),
        ("split", "split(, \",\");"),
        ("trim", "trim();"),
        ("upper", "upper();"),
        ("lower", "lower();"),
        ("to_int", "to_int();"),
        ("sleep", "sleep(100);"),
        ("args", "args();"),
        ("arg", "arg(1);"),
        ("append_file", "append_file(\"\", );"),
        ("lines", "lines();"),
        ("join", "join(, \",\");"),
        ("contains", "contains(, \"\");"),
        ("check", "check();"),
        ("capitalize", "capitalize();"),
        ("env", "env(\"PATH\");"),
        ("exists", "exists(\"file.txt\");"),
        ("time_ms", "time_ms();"),
        ("dir", "dir(\".\");"),
        ("gui", "gui::window(\"\");"),
        ("gui::window", "gui::window(\"\");"),
        ("return", "return "),
    ];
    kw.iter()
        .map(|(l, ins)| {
            json!({
                "label": l,
                "kind": 14,
                "detail": "mavl",
                "insertText": ins,
                "insertTextFormat": 2
            })
        })
        .collect()
}

fn hover_for(word: &str) -> Option<String> {
    let m: HashMap<&str, &str> = [
        ("fn", "Функция. `fn add(a: i32) -> i32 { return a; }`"),
        ("let", "Переменная. `let x: i32 = 5;` / `let mut y = 1;`"),
        ("print", "Печать. `print(x);` → `println!(\"{}\", x)`"),
        ("input", "Ввод. `let n = input(\"name? \");` → строка из stdin"),
        ("len", "Длина. `len(v);` → `(v.len())`"),
        ("read_file", "Файл в строку. `read_file(\"f.txt\");` → `std::fs::read_to_string`"),
        ("write_file", "Строка в файл. `write_file(\"f.txt\", data);`"),
        ("split", "Разбить. `split(s, \",\");` → `Vec<String>`"),
        ("trim", "Обрезать пробелы. `trim(s);` → `String`"),
        ("upper", "Вверхний регистр. `upper(s);` → `String`"),
        ("lower", "Нижний регистр. `lower(s);` → `String`"),
        ("to_int", "В число. `to_int(s);` → `i32`"),
        ("sleep", "Пауза. `sleep(100);` → 100 мс"),
        ("args", "Аргументы. `args();` → `Vec<String>`"),
        ("arg", "Аргумент по номеру. `arg(1);` → `String` (0 — имя программы)"),
        ("append_file", "Дописать в файл. `append_file(\"f\", data);`"),
        ("lines", "По строкам. `lines(s);` → `Vec<String>`"),
        ("join", "Склеить. `join(v, \",\");` → `String`"),
        ("contains", "Подстрока. `contains(s, \"x\");` → `bool`"),
        ("check", "Проверка. `check(1 + 1 == 2);` → `assert!`"),
        ("capitalize", "С заглавной. `capitalize(\"hi\");` → `\"Hi\"`"),
        ("env", "Переменная окружения. `env(\"PATH\");` → `String`"),
        ("exists", "Проверка файла. `exists(\"a.txt\");` → `bool`"),
        ("time_ms", "Время. `time_ms();` → `i64` мс с эпохи"),
        ("dir", "Список папки. `dir(\".\");` → `Vec<String>`"),
        ("gui", "GUI-заглушка. `gui::window(\"hi\");` → `println!(\"[gui-stub] {}\", \"hi\"); // TODO real gui`"),
        ("gui::window", "GUI-заглушка. `gui::window(\"hi\");` → `println!(\"[gui-stub] {}\", \"hi\"); // TODO real gui`"),
        ("window", "GUI-заглушка. `gui::window(\"hi\");` → `println!(\"[gui-stub] {}\", \"hi\"); // TODO real gui`"),
        ("if", "Ветвление как в Rust"),
        ("for", "Цикл `for i in 0..3 { }`"),
        ("while", "Цикл `while cond { }`"),
        ("match", "Матчинг как в Rust"),
        ("struct", "Структура как в Rust"),
        ("mut", "Изменяемая переменная"),
        ("return", "Возврат из функции"),
    ]
    .into_iter()
    .collect();
    m.get(word).map(|s| s.to_string())
}

pub fn run() -> i32 {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    eprintln!("[mavc-lsp] started");
    loop {
        let msg = match read_message(&mut reader) {
            Some(m) => m,
            None => break,
        };
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        match method {
            "initialize" => {
                send(&json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "capabilities": {
                            "textDocumentSync": 1,
                            "completionProvider": {"triggerCharacters": ["p", "f", "l", "g", ":"]},
                            "hoverProvider": true
                        },
                        "serverInfo": {"name": "mavc-lsp", "version": "0.4.0"}
                    }
                }));
            }
            "initialized" => {}
            "shutdown" => {
                send(&json!({"jsonrpc": "2.0", "id": id, "result": null}));
            }
            "exit" => break,
            "textDocument/didOpen" => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let text = msg["params"]["textDocument"]["text"].as_str().unwrap_or("").to_string();
                send(&diagnostics_for(&uri, &text));
            }
            "textDocument/didChange" => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let text = msg["params"]["contentChanges"]
                    .as_array()
                    .and_then(|a| a.last())
                    .and_then(|c| c.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                send(&diagnostics_for(&uri, &text));
            }
            "textDocument/completion" => {
                send(&json!({"jsonrpc": "2.0", "id": id, "result": completions()}));
            }
            "textDocument/hover" => {
                // клиент не шлёт слово — пробуем вытащить из контекста, иначе общая подсказка
                let word = msg["params"]["context"].as_str().unwrap_or("").to_string();
                let md = hover_for(&word).unwrap_or_else(|| "MavLang: fn/let/print/gui::window/if/for/while/match/struct".into());
                send(&json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": {"contents": {"kind": "markdown", "value": md}}
                }));
            }
            _ => {
                if !id.is_null() {
                    send(&json!({"jsonrpc": "2.0", "id": id, "result": null}));
                }
            }
        }
    }
    0
}

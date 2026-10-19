// mavc — MavLang compiler v0.2 на тру Rust.
// Транспилер .mavl -> Rust (intRs) -> cargo/rustc. Без Python.
mod lsp;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

fn header(mem: &str) -> &'static str {
    match mem {
        "arena" => "// MavLang mem=arena | TODO: bump allocator wrapper\n",
        "manual" => "// MavLang mem=manual | unsafe allowed\n",
        _ => "// MavLang mem=borrow | transpiled to Rust (safe borrows)\n",
    }
}

fn project_root() -> PathBuf {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // ищем вверх папку с source/
    let mut cur = cwd.clone();
    for _ in 0..4 {
        if cur.join("source").is_dir() {
            return cur;
        }
        match cur.parent() {
            Some(p) => cur = p.to_path_buf(),
            None => break,
        }
    }
    cwd
}

fn collect_sources(root: &Path) -> Result<Vec<PathBuf>, String> {
    let src_dir = root.join("source");
    let mut others: Vec<PathBuf> = vec![];
    if let Ok(rd) = fs::read_dir(&src_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().map(|x| x == "mavl").unwrap_or(false) {
                others.push(p);
            }
        }
    }
    others.sort();
    if others.is_empty() {
        return Err("source/*.mavl не найден. Положи main.mavl в source/".into());
    }
    // main.mavl первым, остальные по алфавиту
    let mut out: Vec<PathBuf> = Vec::with_capacity(others.len());
    for p in &others {
        if p.file_name().map(|n| n == "main.mavl").unwrap_or(false) {
            out.push(p.clone());
            break;
        }
    }
    for p in others {
        if p.file_name().map(|n| n == "main.mavl").unwrap_or(false) {
            continue;
        }
        out.push(p);
    }
    Ok(out)
}

#[allow(dead_code)]
fn find_entry(root: &Path) -> Result<PathBuf, String> {
    Ok(collect_sources(root)?.remove(0))
}

fn find_closing_paren(c: &[char], open_paren_at: usize) -> Option<usize> {
    // open_paren_at — индекс '('; возвращаем индекс парной ')'
    let mut depth = 0usize;
    let mut s: Option<char> = None;
    let mut j = open_paren_at;
    while j < c.len() {
        let d = c[j];
        if let Some(q) = s {
            if d == '\\' {
                j += 2;
                continue;
            }
            if d == q {
                s = None;
            }
            j += 1;
            continue;
        }
        if d == '"' || d == '\'' {
            s = Some(d);
            j += 1;
            continue;
        }
        if d == '(' {
            depth += 1;
        } else if d == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(j);
            }
        }
        j += 1;
    }
    None
}

/// Верхнеуровневый '+' в строке (вне строк/скобок)?
fn has_top_level_plus(s: &str) -> bool {
    let c: Vec<char> = s.chars().collect();
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    let mut i = 0usize;
    while i < c.len() {
        if let Some(q) = in_str {
            if c[i] == '\\' && i + 1 < c.len() {
                i += 2;
                continue;
            }
            if c[i] == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c[i] {
            '"' | '\'' => in_str = Some(c[i]),
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '+' if depth == 0 => {
                // выбираем только бинарный: слева не оператор/скобка
                let bin = i > 0
                    && !matches!(
                        c[i - 1],
                        '+' | '=' | '(' | '[' | ',' | '-'
                    );
                if bin {
                    return true;
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Режем верхнеуровневым '+' на части (вне строк/скобок).
fn plus_parts(s: &str) -> Vec<String> {
    let c: Vec<char> = s.chars().collect();
    let mut parts: Vec<String> = vec![];
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    let mut i = 0usize;
    while i < c.len() {
        if let Some(q) = in_str {
            cur.push(c[i]);
            if c[i] == '\\' && i + 1 < c.len() {
                cur.push(c[i + 1]);
                i += 2;
                continue;
            }
            if c[i] == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c[i] {
            '"' | '\'' => {
                in_str = Some(c[i]);
                cur.push(c[i]);
                i += 1;
                continue;
            }
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '+' if depth == 0 => {
                let bin = i > 0
                    && !matches!(c[i - 1], '+' | '=' | '(' | '[' | ',' | '-');
                if bin {
                    parts.push(cur.trim().to_string());
                    cur = String::new();
                    i += 1;
                    continue;
                }
            }
            _ => {}
        }
        cur.push(c[i]);
        i += 1;
    }
    parts.push(cur.trim().to_string());
    parts
}

fn has_top_level_comma(s: &str) -> bool {    let c: Vec<char> = s.chars().collect();
    let mut depth = 0usize;
    let mut q: Option<char> = None;
    let mut i = 0;
    while i < c.len() {
        let d = c[i];
        if let Some(cur) = q {
            if d == '\\' {
                i += 2;
                continue;
            }
            if d == cur {
                q = None;
            }
            i += 1;
            continue;
        }
        if d == '"' || d == '\'' {
            q = Some(d);
        } else if d == '(' || d == '[' || d == '{' {
            depth += 1;
        } else if d == ')' || d == ']' || d == '}' {
            depth = depth.saturating_sub(1);
        } else if d == ',' && depth == 0 {
            return true;
        }
        i += 1;
    }
    false
}

/// Граница слова слева: начало строки или prev не буква/цифра/_/точка.
/// Отсекает `x.len(`, `my_len(`, `println` и т.п.
/// Исключение: `0..len(v)` — range-выражение, после `..` builtin раскрываем.
fn is_word_boundary(c: &[char], pos: usize) -> bool {
    if pos == 0 {
        return true;
    }
    let p = c[pos - 1];
    if p == '.' {
        if pos >= 2 && c[pos - 2] == '.' {
            return true; // `..len(` — диапазон, не метод
        }
        return false; // `.len(` — метод, не трогаем
    }
    !(p.is_alphanumeric() || p == '_' || p == ':')
}

/// true если позиция — имя в `fn name(` (определение функции).
/// Встроенные функции тогда не раскрываем: `fn trim(...)` проходит как есть.
fn fn_before(c: &[char], pos: usize) -> bool {
    let s: String = c[..pos].iter().collect();
    s.trim_end().ends_with("fn")
}

/// Делит `a, b` по первой запятой верхнего уровня (вне строк/скобок).
fn split_top2(s: &str) -> Option<(String, String)> {
    let c: Vec<char> = s.chars().collect();
    let mut depth = 0usize;
    let mut q: Option<char> = None;
    let mut i = 0;
    while i < c.len() {
        let d = c[i];
        if let Some(cur) = q {
            if d == '\\' {
                i += 2;
                continue;
            }
            if d == cur {
                q = None;
            }
            i += 1;
            continue;
        }
        if d == '"' || d == '\'' {
            q = Some(d);
        } else if d == '(' || d == '[' || d == '{' {
            depth += 1;
        } else if d == ')' || d == ']' || d == '}' {
            depth = depth.saturating_sub(1);
        } else if d == ',' && depth == 0 {
            return Some((
                c[..i].iter().collect::<String>(),
                c[i + 1..].iter().collect::<String>(),
            ));
        }
        i += 1;
    }
    None
}

/// Один проход. Сгенерированный Rust для повторного прогона стабилен:
/// `println!` пропускается, `.len()/.trim()/.split(` отсекаются границей слова,
// `::sleep/::args` — двоеточием.
fn transpile_once(line: &str) -> String {
    let c: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut saw_gui = false;
    let mut i = 0;
    let mut in_str: Option<char> = None;
    while i < c.len() {
        let ch = c[i];
        if let Some(q) = in_str {
            out.push(ch);
            if ch == '\\' && i + 1 < c.len() {
                out.push(c[i + 1]);
                i += 2;
                continue;
            }
            if ch == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        // комментарий // — дальше не трогаем
        if ch == '/' && i + 1 < c.len() && c[i + 1] == '/' {
            out.push_str(&c[i..].iter().collect::<String>());
            break;
        }
        if ch == '"' || ch == '\'' {
            in_str = Some(ch);
            out.push(ch);
            i += 1;
            continue;
        }
        // println! пропускаем как есть
        if starts_at(&c, i, "println!") {
            out.push_str("println!");
            i += "println!".len();
            continue;
        }
        // встроенный gui-стаб: gui::window(x) -> println!("[gui-stub] {}", x)
        if starts_at(&c, i, "gui::window(") {
            let open = i + "gui::window".len(); // индекс '('
            match find_closing_paren(&c, open) {
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    let a = arg.trim();
                    if a.is_empty() {
                        out.push_str("println!(\"[gui-stub]\")");
                    } else if has_top_level_comma(a) {
                        out.push_str(&format!("println!(\"[gui-stub] {{}}\", format!({}))", a));
                    } else {
                        out.push_str(&format!("println!(\"[gui-stub] {{}}\", {})", a));
                    }
                    saw_gui = true;
                    i = j + 1; // пропускаем ')'
                    continue;
                }
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
            }
        }
        // ---- MavLang std v0.3 (однострочные выражения, без импортов) ----
        // len(x) -> (x.len())
        if starts_at(&c, i, "len(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "len".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    out.push_str(&format!("({}.len())", arg.trim()));
                    i = j + 1;
                    continue;
                }
            }
        }
        // input(prompt) -> прочитать строку из stdin (с промптом без \n)
        if starts_at(&c, i, "input(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "input".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    let a = arg.trim();
                    let prompt = if a.is_empty() {
                        String::new()
                    } else if a.starts_with('"') || a.starts_with('\'') {
                        format!("print!({}); ", a)
                    } else {
                        format!("print!(\"{{}}\", {}); ", a)
                    };
                    out.push_str(&format!("({{ {}std::io::Write::flush(&mut std::io::stdout()).ok(); let mut __s = String::new(); std::io::stdin().read_line(&mut __s).unwrap(); __s.trim().to_string() }})", prompt));
                    i = j + 1;
                    continue;
                }
            }
        }
        // read_file(p) -> String
        if starts_at(&c, i, "read_file(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "read_file".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    out.push_str(&format!("std::fs::read_to_string({}).unwrap()", arg.trim()));
                    i = j + 1;
                    continue;
                }
            }
        }
        // write_file(p, data)
        if starts_at(&c, i, "write_file(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "write_file".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    match split_top2(&arg) {
                        Some((p, d)) => out.push_str(&format!(
                            "std::fs::write({}, {}).unwrap()",
                            p.trim(),
                            d.trim()
                        )),
                        None => out.push_str(&format!("std::fs::write({}).unwrap()", arg.trim())),
                    }
                    i = j + 1;
                    continue;
                }
            }
        }
        // env(name) -> String (переменная окружения)
        if starts_at(&c, i, "env(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "env".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    out.push_str(&format!(
                        "std::env::var({}).unwrap_or_else(|_| String::new())",
                        arg.trim()
                    ));
                    i = j + 1;
                    continue;
                }
            }
        }
        // exists(path) -> bool
        if starts_at(&c, i, "exists(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "exists".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    out.push_str(&format!("std::path::Path::new({}).exists()", arg.trim()));
                    i = j + 1;
                    continue;
                }
            }
        }
        // time_ms() -> i64 (мс с эпохи)
        if starts_at(&c, i, "time_ms()") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            out.push_str(
                "std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64",
            );
            i += "time_ms()".len();
            continue;
        }
        // dir(path) -> Vec<String> (имена файлов в папке)
        if starts_at(&c, i, "dir(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "dir".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    out.push_str(&format!(
                        "std::fs::read_dir({}).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect::<Vec<String>>()",
                        arg.trim()
                    ));
                    i = j + 1;
                    continue;
                }
            }
        }
        // split(s, delim) -> Vec<String>
        if starts_at(&c, i, "split(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "split".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    match split_top2(&arg) {
                        Some((s, d)) => out.push_str(&format!(
                            "{}.split({}).map(|x| x.to_string()).collect::<Vec<String>>()",
                            s.trim(),
                            d.trim()
                        )),
                        None => out.push_str(&format!(
                            "{}.split({}).map(|x| x.to_string()).collect::<Vec<String>>()",
                            arg.trim(),
                            "\",\""
                        )),
                    }
                    i = j + 1;
                    continue;
                }
            }
        }
        // trim(s) / upper(s) / lower(s) -> String
        let mut std_handled = false;
        for (name, method) in [
            ("trim(", "trim"),
            ("upper(", "to_uppercase"),
            ("lower(", "to_lowercase"),
        ] {
            if starts_at(&c, i, name) && is_word_boundary(&c, i) && !fn_before(&c, i) {
                let open = i + name.len() - 1;
                match find_closing_paren(&c, open) {
                    None => {
                        out.push_str(&c[i..].iter().collect::<String>());
                        i = c.len();
                        break;
                    }
                    Some(j) => {
                        let arg: String = c[open + 1..j].iter().collect();
                        out.push_str(&format!("{}.{}().to_string()", arg.trim(), method));
                        i = j + 1;
                        std_handled = true;
                        break;
                    }
                }
            }
        }
        if std_handled {
            continue;
        }
        if i >= c.len() {
            break;
        }
        // to_int(s) -> i32
        if starts_at(&c, i, "to_int(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "to_int".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    out.push_str(&format!("{}.trim().parse::<i32>().unwrap()", arg.trim()));
                    i = j + 1;
                    continue;
                }
            }
        }
        // sleep(ms)
        if starts_at(&c, i, "sleep(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "sleep".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    out.push_str(&format!(
                        "std::thread::sleep(std::time::Duration::from_millis({}))",
                        arg.trim()
                    ));
                    i = j + 1;
                    continue;
                }
            }
        }
        // args() -> Vec<String>
        if starts_at(&c, i, "args()") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            out.push_str("std::env::args().collect::<Vec<String>>()");
            i += "args()".len();
            continue;
        }
        // ---- MavLang std v0.4 ----
        // arg(i) -> i-й аргумент или "" (i=0 это имя программы)
        if starts_at(&c, i, "arg(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "arg".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    out.push_str(&format!(
                        "std::env::args().nth(({}) as usize).unwrap_or_default()",
                        arg.trim()
                    ));
                    i = j + 1;
                    continue;
                }
            }
        }
        // append_file(p, data) — дописать в конец файла
        if starts_at(&c, i, "append_file(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "append_file".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    match split_top2(&arg) {
                        Some((p, d)) => out.push_str(&format!(
                            "({{ let mut __f = std::fs::OpenOptions::new().create(true).append(true).open({}).unwrap(); use std::io::Write as _; __f.write_all({}.as_bytes()).unwrap() }})",
                            p.trim(),
                            d.trim()
                        )),
                        None => out.push_str(&format!(
                            "({{ let mut __f = std::fs::OpenOptions::new().create(true).append(true).open({}).unwrap(); use std::io::Write as _; __f.write_all(\"\".as_bytes()).unwrap() }})",
                            arg.trim()
                        )),
                    }
                    i = j + 1;
                    continue;
                }
            }
        }
        // lines(s) -> Vec<String> (по строкам)
        if starts_at(&c, i, "lines(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "lines".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    out.push_str(&format!(
                        "{}.lines().map(|x| x.to_string()).collect::<Vec<String>>()",
                        arg.trim()
                    ));
                    i = j + 1;
                    continue;
                }
            }
        }
        // join(v, delim) -> String (обратное split)
        if starts_at(&c, i, "join(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "join".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    match split_top2(&arg) {
                        Some((v, d)) => out.push_str(&format!(
                            "{}.join({})",
                            v.trim(),
                            d.trim()
                        )),
                        None => out.push_str(&format!("{}.join(\",\")", arg.trim())),
                    }
                    i = j + 1;
                    continue;
                }
            }
        }
        // contains(s, sub) -> bool (sub может быть и строковой переменной)
        if starts_at(&c, i, "contains(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "contains".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    match split_top2(&arg) {
                        Some((s, sub)) => out.push_str(&format!(
                            "{{ let __p = ({}).to_string(); ({}).contains(__p.as_str()) }}",
                            sub.trim(),
                            s.trim()
                        )),
                        None => out.push_str(&format!("({}).contains(\"\")", arg.trim())),
                    }
                    i = j + 1;
                    continue;
                }
            }
        }
        // find(s, sub) -> i32 (индекс вхождения или -1)
        if starts_at(&c, i, "find(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "find".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    match split_top2(&arg) {
                        Some((s, sub)) => out.push_str(&format!(
                            "({{ let __p = ({}).to_string(); ({}).find(__p.as_str()) }}).map(|p| p as i32).unwrap_or(-1)",
                            sub.trim(),
                            s.trim()
                        )),
                        None => out.push_str(&format!(
                            "({{ let __p = (\"\").to_string(); ({}).find(__p.as_str()) }}).map(|p| p as i32).unwrap_or(-1)",
                            arg.trim()
                        )),
                    }
                    i = j + 1;
                    continue;
                }
            }
        }
        // replace(s, old, new) -> String
        if starts_at(&c, i, "replace(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "replace".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    let (s, rest) = match split_top2(&arg) {
                        Some(v) => v,
                        None => (arg.trim().to_string(), "\"\",\"\"".to_string()),
                    };
                    let (o, n) = match split_top2(&rest) {
                        Some(v) => v,
                        None => (rest.trim().to_string(), "\"\"".to_string()),
                    };
                    out.push_str(&format!(
                        "{{ let __o = ({}).to_string(); let __n = ({}).to_string(); ({}).replace(__o.as_str(), __n.as_str()) }}",
                        o.trim(),
                        n.trim(),
                        s.trim()
                    ));
                    i = j + 1;
                    continue;
                }
            }
        }
        // check(cond) -> assert!(cond), для тестов
        if starts_at(&c, i, "check(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "check".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    out.push_str(&format!("assert!({})", arg.trim()));
                    i = j + 1;
                    continue;
                }
            }
        }
        // capitalize(s) -> String ("hello" -> "Hello")
        if starts_at(&c, i, "capitalize(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            let open = i + "capitalize".len();
            match find_closing_paren(&c, open) {
                None => {
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[open + 1..j].iter().collect();
                    out.push_str(&format!(
                        "({{ let __s: String = ({}).to_string(); let mut __it = __s.chars(); match __it.next() {{ None => String::new(), Some(f) => f.to_uppercase().collect::<String>() + __it.as_str() }} }})",
                        arg.trim()
                    ));
                    i = j + 1;
                    continue;
                }
            }
        }
        if starts_at(&c, i, "print(") && is_word_boundary(&c, i) && !fn_before(&c, i) {
            // ищем закрывающую скобку с учётом вложенности и строк
            let open = i + "print".len(); // индекс '('
            match find_closing_paren(&c, open) {
                None => {
                    // не сбалансировано — оставляем как есть
                    out.push_str(&c[i..].iter().collect::<String>());
                    break;
                }
                Some(j) => {
                    let arg: String = c[i + "print(".len()..j].iter().collect();
                    let a = arg.trim();
                    let vec_fns = ["dir(", "args()", "split(", "lines("];
                    let is_vec = vec_fns.iter().any(|f| a.starts_with(f));
                    // top-level '+' в аргументе → format!-конкатенация строк,
                    // но только если первая часть — строковый литерал
                    // (иначе это может быть числовая арифметика)
                    if has_top_level_plus(a) && a.trim_start().starts_with('"') {
                        let parts = plus_parts(a);
                        if parts.len() >= 2 {
                            let mut fmt = String::new();
                            for _ in 0..parts.len() {
                                fmt.push_str("{}");
                            }
                            out.push_str(&format!(
                                "println!(\"{}\", {})",
                                fmt,
                                parts.join(", ")
                            ));
                            i = j + 1;
                            continue;
                        }
                    }
                    if a.starts_with('"') || a.starts_with('\'') {
                        out.push_str(&format!("println!({})", a));
                    } else if is_vec {
                        out.push_str(&format!("println!(\"{{:?}}\", {})", a));
                    } else {
                        out.push_str(&format!("println!(\"{{}}\", {})", a));
                    }
                    i = j + 1; // пропускаем ')'
                    continue;
                }
            }
        }
        out.push(ch);
        i += 1;
    }
    if saw_gui {
        out.push_str(" // TODO real gui");
    }
    out
}

/// `let x: Type = "литерал";` и `let [mut] x = "литерал";` → добавляет
/// `.to_string()`. Пусть Руст-тип будет String — иначе конкатенации
/// (`res = res + s`) ломаются: &str + &String не собирается.
fn fix_typed_string_let(line: &str) -> String {
    if !line.trim_start().starts_with("let ") {
        return line.to_string();
    }
    let eq = match line.find('=') {
        Some(i) => i,
        None => return line.to_string(),
    };
    let rrhs = line[eq + 1..].trim_start();
    if !rrhs.starts_with('"') {
        return line.to_string();
    }
    // ищем закрывающую кавычку литерала
    let mut end = None;
    let mut chars = rrhs.char_indices().skip(1);
    while let Some((i, ch)) = chars.next() {
        match ch {
            '\\' => {
                chars.next();
            }
            '"' => {
                end = Some(i);
                break;
            }
            _ => {}
        }
    }
    let close = match end {
        Some(i) => i,
        None => return line.to_string(),
    };
    let tail = &rrhs[close + 1..];
    if !(tail.trim_start().starts_with(';') || tail.trim().is_empty()) {
        return line.to_string(); // после литерала что-то ещё — не трогаем
    }
    let rhs_start = line[eq + 1..].len() - rrhs.len() + eq + 1;
    format!(
        "{}{}.to_string(){}",
        &line[..rhs_start],
        &rrhs[..=close],
        tail
    )
}

/// Строка целиком: крутим проходы пока вложенные вызовы
/// (типа `print(trim(s))`) полностью не раскроются. Сходится за 2-3 прохода.
fn transpile_line(line: &str) -> String {
    let mut cur = transpile_once(&fix_typed_string_let(line));
    for _ in 0..8 {
        let next = transpile_once(&cur);
        if next == cur {
            break;
        }
        cur = next;
    }
    cur
}

const TY_STR: u8 = 1;
const TY_INT: u8 = 2;
const TY_VEC: u8 = 3;

/// builtin'ы, возвращающие String (для вывода типов и авто-`&`)
const STR_FNS: [&str; 8] = [
    "read_file",
    "input",
    "upper",
    "lower",
    "trim",
    "capitalize",
    "join",
    "replace",
];

/// Урезает `//`-комментарий (не внутри строковых литералов).
fn strip_line_comment(line: &str) -> String {
    let c: Vec<char> = line.chars().collect();
    let mut in_str: Option<char> = None;
    let mut i = 0usize;
    while i < c.len() {
        if let Some(q) = in_str {
            if c[i] == '\\' && i + 1 < c.len() {
                i += 2;
                continue;
            }
            if c[i] == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if c[i] == '"' || c[i] == '\'' {
            in_str = Some(c[i]);
            i += 1;
            continue;
        }
        if c[i] == '/' && i + 1 < c.len() && c[i + 1] == '/' {
            let s: String = c[..i].iter().collect();
            return s;
        }
        i += 1;
    }
    line.to_string()
}

/// Позиция верхнеуровневого '=' (вне строк и скобок) — иначе None.
fn top_level_eq(line: &str) -> Option<usize> {
    let c: Vec<char> = line.chars().collect();
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    let mut i = 0usize;
    while i < c.len() {
        if let Some(q) = in_str {
            if c[i] == '\\' && i + 1 < c.len() {
                i += 2;
                continue;
            }
            if c[i] == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if c[i] == '"' || c[i] == '\'' {
            in_str = Some(c[i]);
            i += 1;
            continue;
        }
        if c[i] == '/' && i + 1 < c.len() && c[i + 1] == '/' {
            return None;
        }
        match c[i] {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '=' if depth == 0 => {
                if i + 1 < c.len() && c[i + 1] == '=' {
                    i += 2;
                    continue;
                }
                return Some(i);
            }
            '>' if depth == 0 => {
                return None;
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Учит типы переменных из строки `let x = ...` в исходнике .mavl.
fn learn_types(line: &str, vars: &mut std::collections::HashMap<String, u8>) {
    let l = strip_line_comment(line);
    let eq = match top_level_eq(&l) {
        Some(i) => i,
        None => return,
    };
    let prefix = l[..eq].trim().to_string();
    let mut words = prefix.split_whitespace();
    let mut it = words.clone();
    if it.next() == Some("let") {
        words.next(); // let
        let second = words.next().unwrap_or("");
        let (name, _) = match second {
            "mut" => {
                let third = words.next().unwrap_or("");
                (third, 0)
            }
            tok => (tok, 0),
        };
        let name = match name.find(':') {
            Some(p) => &name[..p],
            None => name,
        };
        set_type(vars, name, rhs_type(l[eq + 1..].trim()));
    }
}

fn set_type(vars: &mut std::collections::HashMap<String, u8>, name: &str, t: u8) {
    if !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_alphanumeric() || ch == '_')
    {
        vars.insert(name.to_string(), t);
    }
}

fn rhs_type(rhs: &str) -> u8 {
    let r = rhs.trim().trim_end_matches(';').trim();
    if r.is_empty() {
        return 0;
    }
    if r.starts_with('"') || r.starts_with('\'') {
        return TY_STR;
    }
    if !r.is_empty() && r.chars().all(|ch| ch.is_ascii_digit() || ch == '-' || ch == '.') {
        return TY_INT;
    }
    for (name, t) in [
        ("to_int(", TY_INT),
        ("len(", TY_INT),
        ("time_ms()", TY_INT),
        ("split(", TY_VEC),
        ("lines(", TY_VEC),
        ("args()", TY_VEC),
        ("dir(", TY_VEC),
    ] {
        if r.starts_with(name) {
            return t;
        }
    }
    for name in STR_FNS {
        if r.starts_with(name) && r[name.len()..].starts_with('(') {
            return TY_STR;
        }
    }
    if r.starts_with("String::new()") {
        return TY_STR;
    }
    0
}

/// `args().get(K)` → `arg(K)` (модели любят get; arg проще и безопаснее).
fn fix_args_get(line: &str) -> String {
    let mut cur = line.to_string();
    loop {
        match cur.find("args().get(") {
            None => return cur,
            Some(a) => {
                let c: Vec<char> = cur.chars().collect();
                let open = a + "args().get".len();
                match find_closing_paren(&c, open) {
                    None => return cur,
                    Some(j) => {
                        let inner: String = c[open + 1..j].iter().collect();
                        let mut rebuilt = String::new();
                        let a_chars: Vec<char> = cur.chars().collect();
                        rebuilt.extend(a_chars[..a].iter());
                        rebuilt.push_str("arg(");
                        rebuilt.push_str(inner.trim());
                        rebuilt.push(')');
                        rebuilt.extend(c[j + 1..].iter());
                        cur = rebuilt;
                    }
                }
            }
        }
    }
}

/// `args(N)` → `arg(N)` (частая ошибка ИИ: путаница args()/arg(i)).
/// `args()` без аргументов не трогаем. Однопроходный, без зацикливания.
fn fix_args_n(line: &str) -> String {
    let hay: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < hay.len() {
        if starts_at(&hay, i, "args(") {
            let open = i + 4;
            if let Some(j) = find_closing_paren(&hay, open) {
                let inner: String = hay[open + 1..j].iter().collect();
                if inner.trim().is_empty() {
                    out.push_str("args()");
                    i = j + 1;
                    continue;
                }
                let inner = fix_args_n(&inner);
                out.push_str("arg(");
                out.push_str(&inner);
                out.push(')');
                i = j + 1;
                continue;
            }
        }
        out.push(hay[i]);
        i += 1;
    }
    out
}

/// `for x in v` → `for x in v.iter()` когда v — не диапазон `0..N`.
fn fix_for_in(line: &str) -> String {
    let s = line.trim_start();
    let (lead, rest) = match line.find(s) {
        Some(n) => (&line[..n], s),
        None => ("", line),
    };
    if !(rest.starts_with("for ") || rest.starts_with("for\t")) {
        return line.to_string();
    }
    // X = "for " ... " in " ... до '{' (вне строк)
    let after_for = rest[4..].trim();
    let in_pos = match after_for.find(" in ") {
        Some(p) => p,
        None => return line.to_string(),
    };
    let var = after_for[..in_pos].trim();
    if var.is_empty() || var.split_whitespace().count() != 1 {
        return line.to_string();
    }
    let tail = after_for[in_pos + 4..].trim_end();
    // до '{'
    let after: Vec<char> = tail.chars().collect();
    let mut body = String::new();
    let mut trailing = String::new();
    let mut in_str: Option<char> = None;
    let mut i = 0usize;
    while i < after.len() {
        if let Some(q) = in_str {
            body.push(after[i]);
            if after[i] == '\\' && i + 1 < after.len() {
                body.push(after[i + 1]);
                i += 2;
                continue;
            }
            if after[i] == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if after[i] == '"' || after[i] == '\'' {
            in_str = Some(after[i]);
            body.push(after[i]);
            i += 1;
            continue;
        }
        if after[i] == '{' {
            trailing = after[i..].iter().collect::<String>();
            break;
        }
        body.push(after[i]);
        i += 1;
    }
    let expr = body.trim();
    if expr.is_empty()
        || expr.contains("0..")
        || expr.contains(';')
        || expr.contains('=')
        || expr.starts_with("fn ")
    {
        return line.to_string();
    }
    // только простое слово или сбалансированный вызов; иначе не трогаем
    let ok = {
        let fc: Vec<char> = expr.chars().collect();
        (fc.first().map(|ch| ch.is_alphabetic() || *ch == '_') == Some(true))
            && fc.iter().filter(|ch| **ch == '(').count()
                == fc.iter().filter(|ch| **ch == ')').count()
            && (fc.iter().filter(|ch| **ch == '(').count() > 0 || expr.chars().all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '.' || ch == '(' || ch == ')'))
    };
    if !ok {
        return line.to_string();
    }
    format!("{}for {} in {}.iter() {}", lead, var, expr, trailing)
}

/// Вставляет `&` перед строковыми правыми операндами `+`:
/// `a = a + s` → `a = a + &s` (иначе String moves и E0382).
/// Работает когда левый операнд — строковый, правый — строковая
/// переменная или строковый builtin (trim(...), upper(...) и т.п.).
fn fix_string_concat(line: &str, vars: &std::collections::HashMap<String, u8>) -> String {
    let c: Vec<char> = line.chars().collect();

    // helper: слово, заканчивающееся на позиции idx-1 (сканируем назад)
    let word_back = |mut k: usize| -> usize {
        while k > 0 && (c[k - 1].is_alphanumeric() || c[k - 1] == '_') {
            k -= 1;
        }
        k
    };
    // helper: слово, начинающееся с позиции start
    let word_fwd = |mut k: usize| -> usize {
        while k < c.len() && (c[k].is_alphanumeric() || c[k] == '_') {
            k += 1;
        }
        k
    };
    let is_str_fn_call = |name: &str, pos_after_word: usize| -> bool {
        pos_after_word < c.len()
            && c[pos_after_word] == '('
            && STR_FNS.iter().any(|f| *f == name)
    };

    let mut inserts: Vec<usize> = vec![];
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    let mut i = 0usize;
    while i < c.len() {
        if let Some(q) = in_str {
            if c[i] == '\\' && i + 1 < c.len() {
                i += 2;
                continue;
            }
            if c[i] == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if c[i] == '"' || c[i] == '\'' {
            in_str = Some(c[i]);
            i += 1;
            continue;
        }
        if c[i] == '/' && i + 1 < c.len() && c[i + 1] == '/' {
            break; // комментарий — дальше не анализируем
        }
        match c[i] {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '+' => {
                // не унарный/составной: ++, += или старт строки
                let unary = i == 0
                    || matches!(c.get(i - 1), Some('+') | Some('=') | Some('(') | Some('[') | Some(',') | Some('{') | Some('-') | Some('*') | Some('>') | Some('<'));
                if unary {
                    i += 1;
                    continue;
                }
                // ---- левый операнд ----
                let mut k = i;
                while k > 0 && c[k - 1] == ' ' {
                    k -= 1;
                }
                if k == 0 {
                    i += 1;
                    continue;
                }
                // слово слева от '+'
                let wl = word_back(k);
                if wl == k {
                    i += 1;
                    continue;
                }
                let left_word: String = c[wl..k].iter().collect();
                // перед словом ')' → левый операнд вызов: найдём имя функции
                let left_is_str = if wl > 0 && c[wl - 1] == ')' {
                    // ищем (' под уровнем с учётом вложенности
                    let mut bal = 1i32;
                    let mut p = wl - 1;
                    while p > 0 && bal > 0 {
                        p -= 1;
                        if c[p] == ')' {
                            bal += 1;
                        }
                        if c[p] == '(' {
                            bal -= 1;
                            if bal == 0 {
                                break;
                            }
                        }
                    }
                    if c.get(p) == Some(&'(') {
                        let fw = word_back(p);
                        if fw < p {
                            let name: String = c[fw..p].iter().collect();
                            STR_FNS.iter().any(|f| *f == name)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    matches!(vars.get(&left_word), Some(&TY_STR))
                };
                if !left_is_str {
                    i += 1;
                    continue;
                }
                // ---- правый операнд ----
                let mut m = i + 1;
                while m < c.len() && c[m] == ' ' {
                    m += 1;
                }
                if m >= c.len() {
                    i += 1;
                    continue;
                }
                if c[m] == '&' || c[m] == '"' || c[m] == '\'' {
                    i += 1;
                    continue;
                }
                let wr = word_fwd(m);
                if wr == m {
                    i += 1;
                    continue;
                }
                let right_word: String = c[m..wr].iter().collect();
                let right_is_str = matches!(vars.get(&right_word), Some(&TY_STR))
                    || is_str_fn_call(&right_word, wr);
                if right_is_str {
                    inserts.push(m);
                }
                i += 1;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    if inserts.is_empty() {
        return line.to_string();
    }
    let mut out = String::new();
    let mut set: std::collections::HashSet<usize> = inserts.into_iter().collect();
    for (idx, ch) in c.iter().enumerate() {
        if set.remove(&idx) {
            out.push('&');
        }
        out.push(*ch);
    }
    out
}
/// Builtin-сахар над крейтом rand:
/// `rand()` → `rand::random::<f64>()`
/// `rand_int(a, b)` → `(rand::thread_rng().gen_range((a)..(b)))`
/// Границы слов соблюдаются: `brand(` не трогаем.
fn fix_rand_builtins(line: &str) -> String {
    let hay: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    while i < hay.len() {
        // rand_int(a, b) — сначала, чтобы rand() не съел его префикс
        if starts_at(&hay, i, "rand_int(") && (i == 0 || !is_word(hay[i - 1])) {
            let open = i + "rand_int".len();
            if let Some(j) = find_closing_paren(&hay, open) {
                let inner: String = hay[open + 1..j].iter().collect();
                // один top-level аргумент → до запятой, второй → после
                let mut depth = 0i32;
                let mut split: Option<usize> = None;
                for (k, ch) in inner.char_indices() {
                    match ch {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        ',' if depth == 0 && split.is_none() => {
                            split = Some(k);
                        }
                        _ => {}
                    }
                }
                match split {
                    Some(cpos) => {
                        let a = inner[..cpos].trim();
                        let b = inner[cpos + 1..].trim();
                        let a = fix_rand_builtins(a);
                        let b = fix_rand_builtins(&b);
                        out.push_str(&format!(
                            "(rand::Rng::gen_range(&mut rand::thread_rng(), ({})..({})))",
                            a, b
                        ));
                        i = j + 1;
                        continue;
                    }
                    None => {
                        // один аргумент — это не наш вызов, пропускаем как есть
                    }
                }
            }
        }
        if starts_at(&hay, i, "rand(") && (i == 0 || !is_word(hay[i - 1])) {
            let open = i + "rand".len();
            if let Some(j) = find_closing_paren(&hay, open) {
                let inner: String = hay[open + 1..j].iter().collect();
                if inner.trim().is_empty() {
                    out.push_str("rand::random::<f64>()");
                    i = j + 1;
                    continue;
                }
            }
        }
        out.push(hay[i]);
        i += 1;
    }
    out
}

/// Полный препроцесс строку .mavl перед transpile_line.
fn prep_line(line: &str, vars: &mut std::collections::HashMap<String, u8>) -> String {
    let l = fix_args_get(line);
    let l = fix_args_n(&l);
    let l = fix_rand_builtins(&l);
    learn_types(&l, vars);
    let l = fix_for_in(&l);
    fix_string_concat(&l, vars)
}

fn starts_at(hay: &[char], pos: usize, needle: &str) -> bool {
    let n: Vec<char> = needle.chars().collect();
    if pos + n.len() > hay.len() {
        return false;
    }
    hay[pos..pos + n.len()] == n[..]
}

pub(crate) fn parse_and_transpile(src: &str, mem: &str) -> Result<String, String> {
    if !src.contains("fn main") {
        return Err("нужен fn main() {...} в .mavl".into());
    }
    if src.matches('{').count() != src.matches('}').count() {
        return Err("дисбаланс { } в .mavl".into());
    }
    let mut out: Vec<String> = Vec::new();
    let mut vars: std::collections::HashMap<String, u8> = std::collections::HashMap::new();
    for line in src.lines() {
        out.push(transpile_line(&prep_line(line, &mut vars)));
    }
    Ok(format!("{}{}\n", header(mem), out.join("\n")))
}

/// Загружает все source/*.mavl: main.mavl первым, остальные по алфавиту.
fn load_sources(root: &Path) -> Result<Vec<(String, String)>, String> {
    let paths = collect_sources(root)?;
    let mut out = Vec::with_capacity(paths.len());
    for p in paths {
        let name = p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or("main.mavl".into());
        let content = fs::read_to_string(&p).unwrap_or_default();
        out.push((name, content));
    }
    Ok(out)
}

/// Конкатенирует все .mavl в один Rust: шапка + `// <file>` + строки.
/// Возвращает (код, карта[(имя, marker_line, nlines)]). marker_line — 1-based
/// номер строки `// <file>` в сгенерённом main.rs.
fn combine_and_transpile(
    sources: &[(String, String)],
    mem: &str,
) -> Result<(String, Vec<(String, usize, usize)>), String> {
    let combined: String = sources
        .iter()
        .map(|(_, c)| c.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if !combined.contains("fn main") {
        return Err("нужен fn main() {...} в .mavl".into());
    }
    if combined.matches('{').count() != combined.matches('}').count() {
        return Err("дисбаланс { } в .mavl".into());
    }
    let mut out: Vec<String> = vec![header(mem).trim_end().to_string()];
    let mut map: Vec<(String, usize, usize)> = Vec::with_capacity(sources.len());
    // текущая строка = out.len() (1-based номер следующей пуш-строки)
    for (name, content) in sources {
        let marker_line = out.len() + 1; // куда встанет `// name`
        out.push(format!("// {}", name));
        let mut n = 0usize;
        let mut vars: std::collections::HashMap<String, u8> = std::collections::HashMap::new();
        for line in content.lines() {
            out.push(transpile_line(&prep_line(line, &mut vars)));
            n += 1;
        }
        map.push((name.clone(), marker_line, n));
    }
    Ok((format!("{}\n", out.join("\n")), map))
}

fn map_rust_line_to_mavl(rs_line: i64, map: &[(String, usize, usize)]) -> (String, i64) {
    for (name, marker, n) in map {
        let m = *marker as i64;
        let start = m + 1; // первая строка исходника
        let end = m + *n as i64; // последняя строка исходника
        if rs_line == m {
            return (name.clone(), 1);
        }
        if rs_line >= start && rs_line <= end {
            return (name.clone(), rs_line - m);
        }
    }
    // шапка или хвост — относим к первому файлу
    if let Some((name, _, _)) = map.first() {
        return (name.clone(), 1);
    }
    ("main.mavl".into(), rs_line.max(1))
}

fn map_error_multi(err: &str, map: &[(String, usize, usize)]) -> String {
    // cargo пишет ...main.rs:12:5 — маппим в source/<file>:MM по карте маркеров
    let mut res = String::new();
    let mut i = 0;
    let b = err.as_bytes();
    while i < err.len() {
        if err[i..].starts_with("main.rs:") {
            let mut j = i + "main.rs:".len();
            let mut num = String::new();
            while j < b.len() && b[j].is_ascii_digit() {
                num.push(b[j] as char);
                j += 1;
            }
            match num.parse::<i64>() {
                Ok(n) => {
                    let (file, m) = map_rust_line_to_mavl(n, map);
                    res.push_str(&format!("source/{}:{}", file, m));
                }
                Err(_) => {
                    res.push_str("source/main.mavl:");
                    res.push_str(&num);
                }
            }
            i = j;
            continue;
        }
        let ch = err[i..].chars().next().unwrap();
        res.push(ch);
        i += ch.len_utf8();
    }
    res
}

/// Парсит mavlang.toml, секция [deps] (или [dependencies]): `name = "ver"`.
/// Без внешних TOML-крейтов, только ручной разбор строк.
fn parse_mavlang_toml_deps(text: &str) -> Vec<(String, String)> {
    let mut section = String::new();
    let mut deps: Vec<(String, String)> = vec![];
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if line.starts_with('[') {
            if let Some(end) = line.find(']') {
                section = line[1..end].trim().to_lowercase();
                continue;
            }
        }
        if section != "deps" && section != "dependencies" {
            continue;
        }
        // отрезаем инлайн-коммент '#' вне кавычек
        let mut code = String::new();
        let mut q: Option<char> = None;
        for (idx, ch) in raw.chars().enumerate() {
            if let Some(cur) = q {
                code.push(ch);
                if ch == cur {
                    // грубо: не считаем экраны, для версий достаточно
                    q = None;
                }
                let _ = idx;
                continue;
            }
            if ch == '"' || ch == '\'' {
                q = Some(ch);
                code.push(ch);
            } else if ch == '#' {
                break;
            } else {
                code.push(ch);
            }
        }
        let Some((k, v)) = code.split_once('=') else {
            continue;
        };
        let name = k.trim().trim_matches(|c| c == '"' || c == '\'').trim().to_string();
        let ver = v.trim().trim().to_string();
        if name.is_empty() || ver.is_empty() {
            continue;
        }
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            continue;
        }
        // последний выигрывает
        if let Some(slot) = deps.iter_mut().find(|(n, _)| n == &name) {
            slot.1 = ver;
        } else {
            deps.push((name, ver));
        }
    }
    deps
}

/// Читает mavlang.toml в корне и мержит [dependencies] в intRs/Cargo.toml.
/// Не затирает name/edition, создаёт файлы/секции при отсутствии.
fn sync_deps(root: &Path, extra: &[(String, String)]) {
    let mtoml = root.join("mavlang.toml");
    let mut deps: Vec<(String, String)> = Vec::new();
    if let Ok(text) = fs::read_to_string(&mtoml) {
        deps = parse_mavlang_toml_deps(&text);
    }
    for (k, v) in extra {
        if !deps.iter().any(|(n, _)| n == k) {
            // версия без кавычек → TOML прочитает как число; оборачиваем
            let vv = v.trim();
            let vv = if vv.starts_with('"') || vv.starts_with('\'') || vv.starts_with('{') {
                vv.to_string()
            } else {
                format!("\"{}\"", vv)
            };
            deps.push((k.clone(), vv));
        }
    }
    if deps.is_empty() {
        return;
    }
    let intrs = root.join("intRs");
    fs::create_dir_all(intrs.join("src")).ok();
    let toml_path = intrs.join("Cargo.toml");
    let mut cargo = fs::read_to_string(&toml_path).unwrap_or_else(|_| {
        "[package]\nname = \"mavapp\"\nversion = \"0.1.0\"\nedition = \"2021\"\n".to_string()
    });
    if !cargo.ends_with('\n') {
        cargo.push('\n');
    }
    // разбиваем на строки для точечного мержа внутри [dependencies]
    let mut lines: Vec<String> = cargo.lines().map(|s| s.to_string()).collect();
    // найти границы [dependencies]
    let mut dep_start: Option<usize> = None;
    let mut dep_end: usize = lines.len();
    for (idx, ln) in lines.iter().enumerate() {
        let t = ln.trim().to_lowercase();
        if t.starts_with('[') && t.ends_with(']') {
            if dep_start.is_some() {
                dep_end = idx;
                break;
            }
            if t == "[dependencies]" {
                dep_start = Some(idx);
            }
        }
    }
    match dep_start {
        None => {
            lines.push("[dependencies]".to_string());
            for (k, v) in &deps {
                lines.push(format!("{} = {}", k, v));
            }
        }
        Some(s) => {
            for (k, v) in &deps {
                let mut found: Option<usize> = None;
                for idx in (s + 1)..dep_end {
                    let t = lines[idx].trim();
                    if t.is_empty() || t.starts_with('#') {
                        continue;
                    }
                    // ключ до '=' без кавычек
                    if let Some((lk, _)) = t.split_once('=') {
                        if lk.trim().trim_matches(|c| c == '"' || c == '\'') == k {
                            found = Some(idx);
                            break;
                        }
                    }
                }
                match found {
                    Some(idx) => lines[idx] = format!("{} = {}", k, v),
                    None => {
                        lines.insert(dep_end, format!("{} = {}", k, v));
                        dep_end += 1;
                    }
                }
            }
        }
    }
    let new_cargo = lines.join("\n") + "\n";
    fs::write(&toml_path, new_cargo).ok();
    let names: Vec<String> = deps.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
    println!("[mavc] deps: {}", names.join(", "));
}

#[allow(dead_code)]
fn map_error(err: &str, header_lines: i64, mavl_name: &str) -> String {
    // cargo пишет ...main.rs:12:5 — маппим номер строки обратно в source/*.mavl
    let mut res = String::new();
    let mut i = 0;
    let b = err.as_bytes();
    while i < err.len() {
        if err[i..].starts_with("main.rs:") {
            res.push_str(&format!("source/{}:", mavl_name));
            // парсим число после main.rs:
            let mut j = i + "main.rs:".len();
            let mut num = String::new();
            while j < b.len() && b[j].is_ascii_digit() {
                num.push(b[j] as char);
                j += 1;
            }
            if let Ok(n) = num.parse::<i64>() {
                let m = (n - header_lines).max(1);
                res.push_str(&m.to_string());
            } else {
                res.push_str(&num);
            }
            i = j;
            continue;
        }
        // копируем по одному char (utf8-safe)
        let ch = err[i..].chars().next().unwrap();
        res.push(ch);
        i += ch.len_utf8();
    }
    res
}

/// Мavl-подсказки к частым ошибкам rustc.
fn hints_for(err: &str) -> Vec<&'static str> {
    let mut h: Vec<&'static str> = vec![];
    if err.contains("E0308") {
        h.push("E0308 несовпадение типов: проверь let и аргументы; строковые литералы в let x: String mavc оборачивает .to_string() сам.");
    }
    if err.contains("E0382") {
        h.push("E0382 строка перемещена: конкатенацию в цикле пиши a = a + &s (или просто a + s — mavc вставит & сам).");
    }
    if err.contains("E0425") {
        h.push("E0425 имя не найдено: builtin'ы — arg(i), len(x), split(s, d), join(v, d), replace(s, o, n), find(s, x), to_int(s).");
    }
    if err.contains("E0499") {
        h.push("E0499 повторное заимствование: держи одну изменяемую ссылку.");
    }
    h
}

fn ensure_cargo_project(root: &Path, rs_code: &str) {
    let intrs = root.join("intRs");
    fs::create_dir_all(intrs.join("src")).ok();
    let toml = intrs.join("Cargo.toml");
    if !toml.exists() {
        fs::write(&toml, "[package]\nname = \"mavapp\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
            .ok();
    }
    fs::write(intrs.join("src").join("main.rs"), rs_code).ok();
}

fn do_build(root: &Path, release: bool, mem: &str) -> i32 {
    let sources = match load_sources(root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let (rs_code, line_map) = match combine_and_transpile(&sources, mem) {
        Ok(v) => v,
        Err(e) => {
            println!("MavLang error in source/*.mavl: {}", e);
            return 1;
        }
    };
    ensure_cargo_project(root, &rs_code);
    // авто-deps: builtin rand()/rand_int() тянут крейт rand сами
    let extra: Vec<(String, String)> = if rs_code.contains("rand::") {
        vec![("rand".to_string(), "0.8".to_string())]
    } else {
        vec![]
    };
    sync_deps(root, &extra);
    let intrs_toml = root.join("intRs").join("Cargo.toml");
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(&intrs_toml);
    if release {
        cmd.arg("--release");
    }
    let out = cmd.output().expect("не найден cargo в PATH");
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        println!("[mavc] ошибка компиляции MavLang (mem={}):", mem);
        println!("{}", map_error_multi(&stderr, &line_map));
        for h in hints_for(&stderr) {
            println!("[mavc] подсказка: {}", h);
        }
        return out.status.code().unwrap_or(1);
    }
    let profile = if release { "release" } else { "debug" };
    let src_bin = root.join("intRs").join("target").join(profile);
    let dst_bin = root.join("target").join(profile);
    fs::create_dir_all(&dst_bin).ok();
    let mut copied: Vec<String> = vec![];
    if let Ok(rd) = fs::read_dir(&src_bin) {
        for e in rd.flatten() {
            let p = e.path();
            if !p.is_file() {
                continue;
            }
            let name = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            if !name.starts_with("mavapp") {
                continue;
            }
            let ext = p.extension().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            // exe, dll, pdb + бинарь без расширения; мусор .d/.rlib/.rmeta пропускаем
            let ok = name == "mavapp"
                || name == "mavapp.exe"
                || ext == "dll"
                || ext == "pdb"
                || ext == "exe"
                || (ext.is_empty() && name == "mavapp");
            if ok {
                if fs::copy(&p, dst_bin.join(&name)).is_ok() {
                    copied.push(name);
                }
            }
        }
    }
    println!(
        "[mavc] ok ({}, {}): {} -> target/{}/",
        mem,
        profile,
        if copied.is_empty() {
            "собрано".to_string()
        } else {
            copied.join(", ")
        },
        profile
    );
    0
}

fn do_run(root: &Path, release: bool, mem: &str) -> i32 {
    let rc = do_build(root, release, mem);
    if rc != 0 {
        return rc;
    }
    let profile = if release { "release" } else { "debug" };
    let mut exe = root.join("target").join(profile).join("mavapp.exe");
    if !exe.exists() {
        exe = root.join("target").join(profile).join("mavapp");
    }
    match Command::new(&exe).status() {
        Ok(s) => s.code().unwrap_or(0),
        Err(e) => {
            eprintln!("не могу запустить {}: {}", exe.display(), e);
            1
        }
    }
}

fn do_clean(root: &Path) -> i32 {
    for d in [
        root.join("intRs").join("target"),
        root.join("target").join("debug"),
        root.join("target").join("release"),
    ] {
        if d.exists() {
            fs::remove_dir_all(&d).ok();
        }
    }
    println!("[mavc] cleaned");
    0
}

fn do_check(root: &Path, mem: &str) -> i32 {
    let sources = match load_sources(root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    match combine_and_transpile(&sources, mem) {
        Ok(_) => {
            let names: Vec<String> =
                sources.iter().map(|(n, _)| format!("source/{}", n)).collect();
            println!("[mavc] check ok: {} ({} files)", names.join(", "), names.len());
            0
        }
        Err(e) => {
            println!("MavLang error in source/*.mavl: {}", e);
            1
        }
    }
}

fn do_watch(root: &Path, release: bool, mem: &str) -> i32 {
    println!("[watch] watching source/ ... (Ctrl+C чтобы остановить)");
    let mut last: Option<SystemTime> = None;
    loop {
        let mut newest: Option<SystemTime> = None;
        if let Ok(rd) = fs::read_dir(root.join("source")) {
            for e in rd.flatten() {
                if let Ok(md) = e.metadata() {
                    if let Ok(mt) = md.modified() {
                        newest = Some(match newest {
                            None => mt,
                            Some(n) => if mt > n { mt } else { n },
                        });
                    }
                }
            }
        }
        if let Some(mt) = newest {
            if last.map_or(true, |l| mt > l) {
                last = Some(mt);
                println!("[watch] изменение в source/, пересобираю...");
                let rc = do_build(root, release, mem);
                if rc == 0 {
                    println!("[watch] сборка ок");
                } else {
                    println!("[watch] ошибка сборки (rc={})", rc);
                }
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn print_help() {
    println!("mavc — MavLang compiler v0.2 (Rust)");
    println!("  mavc check [--mem=borrow|arena|manual]");
    println!("  mavc build [--release] [--mem=...]");
    println!("  mavc run [--release] [--mem=...]");
    println!("  mavc clean");
    println!("  mavc lsp   (мини rust-analyzer: диагностика + completion + hover по stdio)");
    println!("  mavc watch [--release] [--mem=...]  (автосборка при изменении source/)");
    println!("  mavlang.toml: [deps] egui=\"0.28\" -> merge в intRs/Cargo.toml");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }
    let root = project_root();
    let cmd = args[1].as_str();
    let mut release = false;
    let mut mem = "borrow".to_string();
    for a in &args[2..] {
        if a == "--release" {
            release = true;
        } else if a.starts_with("--mem=") {
            mem = a["--mem=".len()..].to_string();
        } else if a == "--mem" {
            // ignore, ждём --mem=x
        } else if ["borrow", "arena", "manual"].contains(&a.as_str()) {
            mem = a.clone();
        }
    }
    if !["borrow", "arena", "manual"].contains(&mem.as_str()) {
        eprintln!("--mem должен быть borrow|arena|manual");
        std::process::exit(1);
    }
    let rc = match cmd {
        "check" => do_check(&root, &mem),
        "build" => do_build(&root, release, &mem),
        "run" => do_run(&root, release, &mem),
        "clean" => do_clean(&root),
        "lsp" => lsp::run(),
        "watch" => do_watch(&root, release, &mem),
        _ => {
            print_help();
            1
        }
    };
    std::process::exit(rc);
}

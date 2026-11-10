//! Source clean-up before lexing: the header of exported class and form
//! files (`VERSION`, `BEGIN ... END`) and conditional compilation
//! (`#If`, `#ElseIf`, `#Else`, `#End If`, `#Const`). Lines that are
//! dropped become empty so line numbers stay the same.

use std::collections::HashMap;

/// The constants a 64-bit Windows host defines, as macros written for one
/// expect.
fn builtin_constants() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    for (k, v) in [
        ("vba7", -1.0),
        ("vba6", -1.0),
        ("win64", -1.0),
        ("win32", -1.0),
        ("win16", 0.0),
        ("mac", 0.0),
    ] {
        m.insert(k.to_string(), v);
    }
    m
}

/// Split into lines (any line ending), drop the file header and resolve
/// conditional compilation.
pub fn preprocess(src: &str) -> Vec<String> {
    let text = src.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
    strip_header(&mut lines);
    conditional(&mut lines);
    lines
}

fn first_word(line: &str) -> String {
    line.trim_start()
        .split(|c: char| c.is_whitespace())
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// `VERSION 1.0 CLASS` / `VERSION 5.00` and the `BEGIN ... END` block
/// after it (nested blocks of forms included).
fn strip_header(lines: &mut [String]) {
    let Some(first) = lines.iter().position(|l| !l.trim().is_empty()) else {
        return;
    };
    if first_word(&lines[first]) != "version" {
        return;
    }
    let mut depth = 0i32;
    let mut started = false;
    for line in lines.iter_mut().skip(first) {
        let w = first_word(line);
        let trimmed = line.trim().to_ascii_lowercase();
        if w == "attribute" && !started {
            break;
        }
        if w == "begin" {
            depth += 1;
            started = true;
        } else if trimmed == "end" {
            depth -= 1;
        }
        line.clear();
        if started && depth <= 0 {
            break;
        }
    }
}

fn conditional(lines: &mut [String]) {
    let mut consts = builtin_constants();
    // For each open #If: (this branch is active, a branch was taken, the
    // enclosing code is active).
    let mut stack: Vec<(bool, bool, bool)> = Vec::new();
    let active = |stack: &[(bool, bool, bool)]| stack.last().is_none_or(|s| s.0 && s.2);
    for line in lines.iter_mut() {
        let t = line.trim_start();
        if !t.starts_with('#') {
            if !active(&stack) {
                line.clear();
            }
            continue;
        }
        let body = t[1..].trim();
        let lower = body.to_ascii_lowercase();
        let outer = active(&stack);
        if lower.starts_with("if ") || lower.starts_with("if(") {
            let cond = strip_then(&body[2..]);
            let v = outer && eval(&cond, &consts) != 0.0;
            stack.push((v, v, outer));
        } else if lower.starts_with("elseif") {
            let cond = strip_then(&body[6..]);
            if let Some(top) = stack.last_mut() {
                let v = top.2 && !top.1 && eval(&cond, &consts) != 0.0;
                top.0 = v;
                top.1 |= v;
            }
        } else if lower == "else" || lower.starts_with("else ") || lower.starts_with("else'") {
            if let Some(top) = stack.last_mut() {
                top.0 = top.2 && !top.1;
                top.1 = true;
            }
        } else if lower.starts_with("end if") || lower.starts_with("endif") {
            stack.pop();
        } else if lower.starts_with("const ") {
            if outer {
                let rest = &body[6..];
                if let Some((name, value)) = rest.split_once('=') {
                    let v = eval(value, &consts);
                    consts.insert(name.trim().to_ascii_lowercase(), v);
                }
            }
        } else {
            // Not a directive (a date literal at the start of a line, say).
            if !outer {
                line.clear();
            }
            continue;
        }
        line.clear();
    }
}

fn strip_then(s: &str) -> String {
    let t = s.trim();
    let lower = t.to_ascii_lowercase();
    match lower.rfind("then") {
        Some(i) if lower[i..].trim() == "then" => t[..i].to_string(),
        _ => t.to_string(),
    }
}

/// Evaluate a conditional-compilation expression: numbers, True/False,
/// constants, Not, And, Or, Xor, comparisons and parentheses. Unknown
/// names are 0.
fn eval(src: &str, consts: &HashMap<String, f64>) -> f64 {
    let toks = tokenize(src);
    let mut p = 0;
    or_expr(&toks, &mut p, consts)
}

fn tokenize(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c == '\'' {
            break;
        }
        if c.is_whitespace() {
            i += 1;
        } else if c.is_alphanumeric() || c == '_' || c == '.' {
            let st = i;
            while i < cs.len() && (cs[i].is_alphanumeric() || cs[i] == '_' || cs[i] == '.') {
                i += 1;
            }
            out.push(cs[st..i].iter().collect::<String>().to_ascii_lowercase());
        } else if c == '"' {
            let st = i;
            i += 1;
            while i < cs.len() && cs[i] != '"' {
                i += 1;
            }
            i += 1;
            out.push(cs[st..i.min(cs.len())].iter().collect());
        } else {
            let two: String = cs[i..(i + 2).min(cs.len())].iter().collect();
            if ["<>", "<=", ">="].contains(&two.as_str()) {
                out.push(two);
                i += 2;
            } else {
                out.push(c.to_string());
                i += 1;
            }
        }
    }
    out
}

fn or_expr(t: &[String], p: &mut usize, c: &HashMap<String, f64>) -> f64 {
    let mut v = and_expr(t, p, c);
    while *p < t.len() && (t[*p] == "or" || t[*p] == "xor") {
        let op = t[*p].clone();
        *p += 1;
        let r = and_expr(t, p, c);
        let (a, b) = (v as i64, r as i64);
        v = if op == "or" {
            (a | b) as f64
        } else {
            (a ^ b) as f64
        };
    }
    v
}

fn and_expr(t: &[String], p: &mut usize, c: &HashMap<String, f64>) -> f64 {
    let mut v = not_expr(t, p, c);
    while *p < t.len() && t[*p] == "and" {
        *p += 1;
        let r = not_expr(t, p, c);
        v = ((v as i64) & (r as i64)) as f64;
    }
    v
}

fn not_expr(t: &[String], p: &mut usize, c: &HashMap<String, f64>) -> f64 {
    if *p < t.len() && t[*p] == "not" {
        *p += 1;
        return !(not_expr(t, p, c) as i64) as f64;
    }
    cmp_expr(t, p, c)
}

fn cmp_expr(t: &[String], p: &mut usize, c: &HashMap<String, f64>) -> f64 {
    let a = atom(t, p, c);
    if *p < t.len() {
        let op = t[*p].clone();
        let r = match op.as_str() {
            "=" | "<>" | "<" | ">" | "<=" | ">=" => {
                *p += 1;
                let b = atom(t, p, c);
                Some(match op.as_str() {
                    "=" => a == b,
                    "<>" => a != b,
                    "<" => a < b,
                    ">" => a > b,
                    "<=" => a <= b,
                    _ => a >= b,
                })
            }
            _ => None,
        };
        if let Some(r) = r {
            return if r { -1.0 } else { 0.0 };
        }
    }
    a
}

fn atom(t: &[String], p: &mut usize, c: &HashMap<String, f64>) -> f64 {
    let Some(tok) = t.get(*p).cloned() else {
        return 0.0;
    };
    *p += 1;
    match tok.as_str() {
        "(" => {
            let v = or_expr(t, p, c);
            if t.get(*p).map(String::as_str) == Some(")") {
                *p += 1;
            }
            v
        }
        "-" => -atom(t, p, c),
        "true" => -1.0,
        "false" => 0.0,
        _ => tok
            .parse::<f64>()
            .ok()
            .or_else(|| c.get(&tok).copied())
            .unwrap_or(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditional_compilation_keeps_the_64_bit_branch() {
        let src = "#If VBA7 Then\nA\n#Else\nB\n#End If\n#If Mac Then\nC\n#ElseIf Win64 And Not Mac Then\nD\n#End If\nE";
        let l = preprocess(src);
        assert_eq!(l, vec!["", "A", "", "", "", "", "", "", "D", "", "E"]);
    }

    #[test]
    fn const_directives_feed_later_conditions() {
        let src = "#Const DEBUG_ON = 1\n#If DEBUG_ON = 1 Then\nX\n#End If";
        assert_eq!(preprocess(src), vec!["", "", "X", ""]);
    }

    #[test]
    fn exported_form_headers_are_blanked() {
        let src = "VERSION 5.00\nBegin {C62A} UserForm1\n   Caption = \"x\"\n   Begin Forms.Label.1 L\n   End\nEnd\nAttribute VB_Name = \"UserForm1\"\nSub A()";
        let l = preprocess(src);
        assert!(l[..6].iter().all(String::is_empty));
        assert_eq!(l[6], "Attribute VB_Name = \"UserForm1\"");
        let cls =
            "VERSION 1.0 CLASS\nBEGIN\n  MultiUse = -1  'True\nEND\nAttribute VB_Name = \"C\"";
        let l = preprocess(cls);
        assert!(l[..4].iter().all(String::is_empty));
    }
}

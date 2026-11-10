//! Script debugging: breakpoints and stepping in the Scripts panel.
//!
//! The script is instrumented before it runs: in front of every statement
//! that starts a line, a check `__bp(N)&&__pause(N,e=>eval(e));` goes on
//! that same line (so line numbers stay). [`instrument`] finds those lines
//! with a small tokenizer and keeps only places where the check cannot
//! change what the script means; the result must still parse. On the
//! desktop the script runs on a worker thread that waits at a pause for
//! Continue, Step, Stop or an expression to evaluate in the paused scope
//! (the arrow function's direct `eval` sees the local variables).

/// One token of the tokenizer: enough to tell statements apart.
#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Word(String),
    /// Numbers, strings, templates, regular expressions.
    Literal,
    Punct(String),
}

#[derive(Debug, Clone)]
struct Token {
    tok: Tok,
    /// 1-based line and the byte offset where the token starts.
    line: u32,
    offset: usize,
    first_on_line: bool,
}

/// The bracket a token sits in.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Ctx {
    Block,
    Object,
    Class,
    Switch,
    Paren,
    Bracket,
    Template,
}

const PUNCTS: [&str; 52] = [
    ">>>=", "...", "===", "!==", "**=", "<<=", ">>=", ">>>", "&&=", "||=", "??=", "=>", "==", "!=",
    "<=", ">=", "&&", "||", "??", "?.", "++", "--", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=",
    "<<", ">>", "**", "{", "}", "(", ")", "[", "]", ";", ",", "<", ">", "+", "-", "*", "%", "&",
    "|", "^", "!", "~",
];

/// Keywords after which a `/` starts a regular expression.
const REGEX_AFTER: [&str; 15] = [
    "return",
    "typeof",
    "instanceof",
    "in",
    "of",
    "new",
    "delete",
    "void",
    "throw",
    "case",
    "do",
    "else",
    "yield",
    "await",
    "extends",
];

/// Words that end an expression (a statement may follow on the next line).
const VALUE_WORDS: [&str; 7] = [
    "this", "true", "false", "null", "super", "break", "continue",
];

/// Words that cannot start a statement here, or that continue the one
/// before.
const NOT_START: [&str; 9] = [
    "else",
    "catch",
    "finally",
    "in",
    "of",
    "instanceof",
    "case",
    "default",
    "while",
];

fn tokenize(src: &str) -> Option<Vec<Token>> {
    let b = src.as_bytes();
    let mut out: Vec<Token> = Vec::new();
    let mut i = 0usize;
    let mut line = 1u32;
    let mut line_has_token = false;
    // Brace depth inside each open template substitution.
    let mut templates: Vec<u32> = Vec::new();
    let regex_ok = |out: &Vec<Token>| match out.last().map(|t| &t.tok) {
        None => true,
        Some(Tok::Literal) => false,
        Some(Tok::Word(w)) => REGEX_AFTER.contains(&w.as_str()),
        Some(Tok::Punct(p)) => !matches!(p.as_str(), ")" | "]" | "}"),
    };
    let push = |out: &mut Vec<Token>, tok: Tok, line: u32, offset: usize, first: &mut bool| {
        out.push(Token {
            tok,
            line,
            offset,
            first_on_line: !*first,
        });
        *first = true;
    };
    // Scan a template's text from `i` (after the backtick or a closing
    // brace of a substitution); returns the index after it and whether a
    // substitution opened.
    fn template_text(b: &[u8], mut i: usize, line: &mut u32) -> Option<(usize, bool)> {
        while i < b.len() {
            match b[i] {
                b'\\' => i += 2,
                b'`' => return Some((i + 1, false)),
                b'$' if b.get(i + 1) == Some(&b'{') => return Some((i + 2, true)),
                b'\n' => {
                    *line += 1;
                    i += 1;
                }
                _ => i += 1,
            }
        }
        None
    }
    while i < b.len() {
        let c = b[i];
        if c == b'\n' {
            line += 1;
            line_has_token = false;
            i += 1;
            continue;
        }
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'*') {
            i += 2;
            loop {
                if i + 1 >= b.len() {
                    return None;
                }
                if b[i] == b'*' && b[i + 1] == b'/' {
                    i += 2;
                    break;
                }
                if b[i] == b'\n' {
                    line += 1;
                }
                i += 1;
            }
            continue;
        }
        let start = i;
        let start_line = line;
        if c == b'"' || c == b'\'' {
            i += 1;
            while i < b.len() && b[i] != c {
                if b[i] == b'\\' {
                    i += 1;
                }
                if b.get(i) == Some(&b'\n') {
                    line += 1;
                }
                i += 1;
            }
            if i >= b.len() {
                return None;
            }
            i += 1;
            push(
                &mut out,
                Tok::Literal,
                start_line,
                start,
                &mut line_has_token,
            );
            continue;
        }
        if c == b'`' {
            let (next, open) = template_text(b, i + 1, &mut line)?;
            i = next;
            push(
                &mut out,
                Tok::Literal,
                start_line,
                start,
                &mut line_has_token,
            );
            if open {
                templates.push(0);
                push(
                    &mut out,
                    Tok::Punct("${".into()),
                    line,
                    i - 2,
                    &mut line_has_token,
                );
            }
            continue;
        }
        if c == b'}' && templates.last() == Some(&0) {
            templates.pop();
            push(
                &mut out,
                Tok::Punct("}$".into()),
                line,
                i,
                &mut line_has_token,
            );
            let (next, open) = template_text(b, i + 1, &mut line)?;
            i = next;
            if open {
                templates.push(0);
                push(
                    &mut out,
                    Tok::Punct("${".into()),
                    line,
                    i - 2,
                    &mut line_has_token,
                );
            }
            continue;
        }
        if c.is_ascii_digit() || (c == b'.' && b.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'.' || b[i] == b'_') {
                i += 1;
            }
            push(
                &mut out,
                Tok::Literal,
                start_line,
                start,
                &mut line_has_token,
            );
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' || c == b'$' || c == b'\\' || c >= 0x80 {
            while i < b.len()
                && (b[i].is_ascii_alphanumeric()
                    || b[i] == b'_'
                    || b[i] == b'$'
                    || b[i] == b'\\'
                    || b[i] >= 0x80)
            {
                i += 1;
            }
            let w = String::from_utf8_lossy(&b[start..i]).into_owned();
            push(
                &mut out,
                Tok::Word(w),
                start_line,
                start,
                &mut line_has_token,
            );
            continue;
        }
        if c == b'/' && regex_ok(&out) {
            i += 1;
            let mut class = false;
            while i < b.len() {
                match b[i] {
                    b'\\' => i += 1,
                    b'[' => class = true,
                    b']' => class = false,
                    b'/' if !class => break,
                    b'\n' => return None,
                    _ => {}
                }
                i += 1;
            }
            if i >= b.len() {
                return None;
            }
            i += 1;
            while i < b.len() && b[i].is_ascii_alphabetic() {
                i += 1;
            }
            push(
                &mut out,
                Tok::Literal,
                start_line,
                start,
                &mut line_has_token,
            );
            continue;
        }
        let rest = &src[i..];
        let p = PUNCTS
            .iter()
            .find(|p| rest.starts_with(**p))
            .map(|p| p.to_string())
            .unwrap_or_else(|| (c as char).to_string());
        i += p.len().max(1);
        if let Some(depth) = templates.last_mut() {
            if p == "{" {
                *depth += 1;
            } else if p == "}" {
                *depth = depth.saturating_sub(1);
            }
        }
        push(
            &mut out,
            Tok::Punct(p),
            start_line,
            start,
            &mut line_has_token,
        );
    }
    Some(out)
}

/// The byte offsets and line numbers where a pause check can go.
fn statement_starts(tokens: &[Token]) -> Vec<(usize, u32)> {
    let mut stack: Vec<Ctx> = Vec::new();
    let mut pending_class = false;
    let mut pending_switch = false;
    let mut out = Vec::new();
    let mut last_line = 0u32;
    // For each open parenthesis: whether it holds the head of an `if`,
    // `for`, `while` or `with` (the statement it guards follows).
    let mut parens: Vec<bool> = Vec::new();
    let mut control_head_closed = false;
    for (k, t) in tokens.iter().enumerate() {
        let prev = k.checked_sub(1).and_then(|j| tokens.get(j)).map(|t| &t.tok);
        let top = stack.last().copied();
        let in_block = matches!(top, None | Some(Ctx::Block));
        let starts_ok = match &t.tok {
            Tok::Word(w) => !NOT_START.contains(&w.as_str()),
            Tok::Punct(p) => p == "{",
            Tok::Literal => false,
        };
        let prev_ok = match prev {
            None => true,
            Some(Tok::Literal) => true,
            Some(Tok::Punct(p)) => {
                matches!(p.as_str(), ";" | "{" | "}" | "]") || (p == ")" && !control_head_closed)
            }
            Some(Tok::Word(w)) => {
                VALUE_WORDS.contains(&w.as_str()) || w == "return" || !is_keyword(w)
            }
        };
        // A `{` closing back to a block or the top level: the previous
        // `{` must have been a block's, not an object's.
        let prev_brace_ok = match prev {
            Some(Tok::Punct(p)) if p == "{" => in_block,
            _ => true,
        };
        if t.first_on_line
            && t.line != last_line
            && in_block
            && starts_ok
            && prev_ok
            && prev_brace_ok
        {
            out.push((t.offset, t.line));
            last_line = t.line;
        }
        match &t.tok {
            Tok::Word(w) if w == "class" => pending_class = true,
            Tok::Word(w) if w == "switch" => pending_switch = true,
            Tok::Punct(p) => match p.as_str() {
                "(" => {
                    stack.push(Ctx::Paren);
                    parens.push(matches!(
                        prev,
                        Some(Tok::Word(w)) if matches!(w.as_str(), "if" | "for" | "while" | "with")
                    ));
                }
                "[" => stack.push(Ctx::Bracket),
                "${" => stack.push(Ctx::Template),
                "{" => {
                    let ctx = if pending_class {
                        pending_class = false;
                        Ctx::Class
                    } else if pending_switch && matches!(prev, Some(Tok::Punct(p)) if p == ")") {
                        pending_switch = false;
                        Ctx::Switch
                    } else {
                        match prev {
                            None => Ctx::Block,
                            Some(Tok::Punct(p)) => match p.as_str() {
                                ";" | "{" | "}" | ")" | "=>" => {
                                    if p == "{" && !matches!(top, None | Some(Ctx::Block)) {
                                        Ctx::Object
                                    } else {
                                        Ctx::Block
                                    }
                                }
                                _ => Ctx::Object,
                            },
                            Some(Tok::Word(w)) => {
                                if matches!(w.as_str(), "else" | "try" | "finally" | "do") {
                                    Ctx::Block
                                } else if is_keyword(w) && !VALUE_WORDS.contains(&w.as_str()) {
                                    Ctx::Object
                                } else {
                                    // `a\n{` starts a block by ASI only
                                    // at the start of a line; treat as an
                                    // object to stay safe.
                                    Ctx::Object
                                }
                            }
                            Some(Tok::Literal) => Ctx::Object,
                        }
                    };
                    stack.push(ctx);
                }
                ")" => {
                    stack.pop();
                    control_head_closed = parens.pop().unwrap_or(false);
                }
                "]" | "}" | "}$" => {
                    stack.pop();
                }
                _ => {}
            },
            _ => {}
        }
    }
    out
}

fn is_keyword(w: &str) -> bool {
    matches!(
        w,
        "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "export"
            | "extends"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
            | "let"
            | "static"
            | "await"
            | "of"
            | "true"
            | "false"
            | "null"
    )
}

/// The check that goes in front of a statement on line `n`.
fn check(n: u32) -> String {
    format!("__bp({n})&&__pause({n},e=>eval(e));")
}

fn parses(src: &str) -> bool {
    let mut ctx = boa_engine::Context::default();
    boa_engine::Script::parse(boa_engine::Source::from_bytes(src), None, &mut ctx).is_ok()
}

fn insert(src: &str, places: &[(usize, u32)]) -> String {
    let mut out = String::with_capacity(src.len() + places.len() * 40);
    let mut at = 0;
    for (offset, line) in places {
        out.push_str(&src[at..*offset]);
        out.push_str(&check(*line));
        at = *offset;
    }
    out.push_str(&src[at..]);
    out
}

/// The script with pause checks, and the lines that have one (where a
/// breakpoint can stop). A script that does not parse comes back as it is
/// with no lines.
pub fn instrument(src: &str) -> (String, Vec<u32>) {
    if !parses(src) {
        return (src.to_string(), Vec::new());
    }
    let Some(tokens) = tokenize(src) else {
        return (src.to_string(), Vec::new());
    };
    let mut places = statement_starts(&tokens);
    let mut out = insert(src, &places);
    if !parses(&out) {
        // Keep only the places that parse one by one, then all together.
        places.retain(|p| parses(&insert(src, std::slice::from_ref(p))));
        out = insert(src, &places);
        if !parses(&out) {
            return (src.to_string(), Vec::new());
        }
    }
    let lines = places.iter().map(|p| p.1).collect();
    (out, lines)
}

/// The line a breakpoint set on `line` stops at: that line when it has a
/// check, else the next one that has.
pub fn effective_line(lines: &[u32], line: u32) -> Option<u32> {
    lines.iter().copied().find(|l| *l >= line)
}

impl crate::app::App {
    /// Scripts panel, Debug: run the panel's script with its breakpoints
    /// (desktop; the browser has no worker threads).
    pub fn start_script_debug(&mut self) {
        if self.script_debug.is_some() || crate::files::WEB {
            return;
        }
        let src = self.script_source.clone();
        let bps: Vec<u32> = self.breakpoints.iter().copied().collect();
        match crate::scripting::start_debug(self, &src, &bps) {
            Ok(run) => {
                self.script_debug = Some(run);
                self.debug_paused = None;
            }
            Err(e) => self.scripts_output.push(format!("Error: {e}")),
        }
    }

    /// Continue, Step, Stop or evaluate in the paused script.
    pub fn debug_control(&mut self, ctl: crate::scripting::DebugCtl) {
        let Some(run) = &self.script_debug else {
            return;
        };
        if let crate::scripting::DebugCtl::Eval(e) = &ctl {
            self.scripts_output.push(format!("> {e}"));
        } else {
            self.debug_paused = None;
        }
        let _ = run.ctl.send(ctl);
    }

    /// A click in the editor's margin: set or clear a breakpoint, also in
    /// a running script.
    pub fn toggle_breakpoint(&mut self, line: u32) {
        if !self.breakpoints.remove(&line) {
            self.breakpoints.insert(line);
        }
        if let Some(run) = &self.script_debug {
            let set = self
                .breakpoints
                .iter()
                .filter_map(|l| effective_line(&run.lines, *l))
                .collect();
            if let Ok(mut b) = run.breakpoints.lock() {
                *b = set;
            }
        }
    }

    /// Each frame: take what the debugged script reports; apply its
    /// changes when it ends. Whether it still runs.
    pub fn debug_tick(&mut self) -> bool {
        let Some(run) = &self.script_debug else {
            return false;
        };
        while let Ok(ev) = run.events.try_recv() {
            match ev {
                crate::scripting::DebugEvent::Paused { line, output } => {
                    self.scripts_output.extend(output);
                    self.debug_paused = Some(line);
                }
                crate::scripting::DebugEvent::Value(v) => self.scripts_output.push(v),
            }
        }
        if !run.is_finished() {
            return true;
        }
        if let Some(run) = self.script_debug.take() {
            let (output, error) = crate::scripting::finish_debug(self, run);
            self.scripts_output.extend(output);
            if let Some(e) = error {
                self.scripts_output.push(format!("Error: {e}"));
            }
        }
        self.debug_paused = None;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(src: &str) -> Vec<u32> {
        instrument(src).1
    }

    #[test]
    fn statements_that_start_lines_get_checks() {
        let src = "const a = 1;\nlet b = a +\n  2;\nif (b > 2)\n  print(b);\nfor (let i = 0; i < 2; i++) {\n  print(i);\n}\nfunction f(x) {\n  return x * 2;\n}\nprint(f(3))\nprint(4)\n";
        // Line 3 continues line 2; line 5 is the body of the `if` on line 4
        // (a check there would run the print unconditionally).
        assert_eq!(lines(src), vec![1, 2, 4, 6, 7, 9, 10, 12, 13]);
        let (out, _) = instrument(src);
        assert!(out.contains("__bp(7)&&__pause(7,e=>eval(e));print(i);"));
        // The same number of lines: errors keep their line numbers.
        assert_eq!(out.lines().count(), src.lines().count());
    }

    #[test]
    fn objects_classes_switches_and_templates_get_none_inside() {
        let src = "const o = {\n  a: 1,\n  b() {\n    return 2;\n  },\n};\nclass C {\n  m() {\n    return 3;\n  }\n}\nswitch (o.a) {\n  case 1:\n    print(1);\n}\nconst t = `x ${o.a\n} y`;\nconst r = /}{/g;\nprint(t)\n";
        assert_eq!(lines(src), vec![1, 4, 7, 9, 12, 16, 18, 19]);
    }

    #[test]
    fn calls_and_operators_on_the_next_line_are_left_alone() {
        // `(`, `[`, `.`, template and operator lines continue the line
        // before; else, catch and finally follow a block; a do-while's
        // while too.
        let src = "let x = print\n(1)\nlet y = [1, 2]\n[0]\nlet z = 'a'\n  .length\ntry {\n  z = 1\n}\ncatch (e) {\n}\ndo {\n  z++\n}\nwhile (z < 3)\n";
        assert_eq!(lines(src), vec![1, 3, 5, 7, 8, 12, 13]);
    }

    #[test]
    fn instrumented_scripts_run_the_same() {
        let src = "let s = 0;\nfor (const v of [1, 2, 3]) {\n  s += v\n}\nconst o = { k: s,\n  f() { return this.k } };\nclass K { get v() { return o.f() * 2 } }\nconst label = `${s}-${new K().v}`;\nlabel\n";
        let run = |code: &str, bp: &str| {
            let mut ctx = boa_engine::Context::default();
            ctx.eval(boa_engine::Source::from_bytes(bp))
                .expect("helpers");
            ctx.eval(boa_engine::Source::from_bytes(code))
                .expect("runs")
                .display()
                .to_string()
        };
        let (out, l) = instrument(src);
        assert!(!l.is_empty());
        let helpers = "function __bp(n) { return false; } function __pause() {}";
        assert_eq!(run(src, helpers), run(&out, helpers));
        assert_eq!(run(&out, helpers), "\"6-12\"");
    }

    #[test]
    fn broken_or_odd_input_is_left_as_it_is() {
        for src in ["let = ;", "'open", "/* open", "a /", "`x ${", "\u{1F600}"] {
            let (out, l) = instrument(src);
            assert_eq!(out, src);
            assert!(l.is_empty(), "{src}");
        }
        assert_eq!(effective_line(&[2, 5, 9], 3), Some(5));
        assert_eq!(effective_line(&[2, 5, 9], 10), None);
    }

    fn wait_paused(app: &mut crate::app::App) -> Option<u32> {
        for _ in 0..400 {
            app.debug_tick();
            if app.debug_paused.is_some() || app.script_debug.is_none() {
                return app.debug_paused;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        None
    }

    fn wait_done(app: &mut crate::app::App) {
        for _ in 0..400 {
            if !app.debug_tick() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("the script did not end");
    }

    fn shapes(app: &crate::app::App) -> usize {
        app.doc().pages[0]
            .layers
            .iter()
            .map(|l| l.shapes.len())
            .sum()
    }

    const LOOP: &str = "let n = 0;\nfor (let i = 0; i < 3; i++) {\n  n += i;\n  ActiveLayer.CreateRectangle(i * 10, 0, 5, 5);\n}\nprint('n=' + n);\n";

    #[test]
    fn breakpoints_pause_and_show_the_local_scope() {
        use crate::scripting::DebugCtl;
        let mut app = crate::app::App::headless();
        let before = shapes(&app);
        app.script_source = LOOP.into();
        app.toggle_breakpoint(4);
        app.start_script_debug();
        assert_eq!(wait_paused(&mut app), Some(4));
        // The loop's own variables are in reach.
        app.debug_control(DebugCtl::Eval("n + ':' + i".into()));
        for _ in 0..200 {
            app.debug_tick();
            if app.scripts_output.last().is_some_and(|l| l.contains(':'))
                && !app
                    .scripts_output
                    .last()
                    .is_some_and(|l| l.starts_with('>'))
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(
            app.scripts_output.last().map(String::as_str),
            Some("\"0:0\"")
        );
        // Continue stops at the same breakpoint in the next turn.
        app.debug_control(DebugCtl::Continue);
        assert_eq!(wait_paused(&mut app), Some(4));
        // Step goes to the next statement.
        app.debug_control(DebugCtl::Step);
        assert_eq!(wait_paused(&mut app), Some(3));
        // Without the breakpoint the script runs to its end.
        app.toggle_breakpoint(4);
        app.debug_control(DebugCtl::Continue);
        wait_done(&mut app);
        assert_eq!(shapes(&app), before + 3);
        assert!(
            app.scripts_output.iter().any(|l| l == "n=3"),
            "{:?}",
            app.scripts_output
        );
        // One undo step.
        app.undo();
        assert_eq!(shapes(&app), before);
    }

    #[test]
    fn stop_ends_the_script_keeping_what_ran() {
        use crate::scripting::DebugCtl;
        let mut app = crate::app::App::headless();
        let before = shapes(&app);
        app.script_source = LOOP.into();
        // A breakpoint on a line without a statement stops at the next one.
        app.toggle_breakpoint(5);
        app.start_script_debug();
        assert_eq!(wait_paused(&mut app), Some(6));
        app.debug_control(DebugCtl::Stop);
        wait_done(&mut app);
        assert_eq!(shapes(&app), before + 3);
        assert!(
            app.scripts_output.iter().any(|l| l.contains("stopped")),
            "{:?}",
            app.scripts_output
        );
        assert!(!app.scripts_output.iter().any(|l| l == "n=3"));
        // Dropping a paused run (the panel closed) stops it too.
        app.toggle_breakpoint(5);
        app.toggle_breakpoint(1);
        app.start_script_debug();
        assert_eq!(wait_paused(&mut app), Some(1));
        let run = app.script_debug.take().expect("run");
        let crate::scripting::DebugRun { ctl, events, .. } = run;
        drop(ctl);
        drop(events);
    }
}

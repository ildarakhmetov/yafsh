use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

use crate::builtins::system::exec_word;
use crate::loops;
use crate::tokenizer;
use crate::types::{ControlFlow, LoopType, SkipTarget, State, Value, Word};

// ========== PATH lookup ==========

/// Check if a file exists and is executable.
fn is_executable(path: &str) -> bool {
    if let Ok(meta) = fs::metadata(path) {
        meta.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

/// Find a command in PATH, return its absolute path if found.
fn find_in_path(cmd: &str) -> Option<String> {
    // Absolute path
    if cmd.starts_with('/') {
        return if is_executable(cmd) {
            Some(cmd.to_string())
        } else {
            None
        };
    }

    // Relative path with /
    if cmd.contains('/') {
        if let Ok(cwd) = std::env::current_dir() {
            let abs = cwd.join(cmd);
            let abs_str = abs.to_string_lossy().to_string();
            return if is_executable(&abs_str) {
                Some(abs_str)
            } else {
                None
            };
        }
        return None;
    }

    // Search PATH
    let path_var = std::env::var("PATH").ok()?;
    for dir in path_var.split(':') {
        let full = format!("{}/{}", dir, cmd);
        if is_executable(&full) {
            return Some(full);
        }
    }
    None
}

// ========== Glob expansion ==========

/// Check if a string contains glob characters.
fn has_glob_chars(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

/// Simple glob matching: `*` matches any sequence, `?` matches one char.
fn glob_matches(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    glob_match_helper(&pat, &txt, 0, 0)
}

fn glob_match_helper(pat: &[char], txt: &[char], pi: usize, ti: usize) -> bool {
    if pi == pat.len() {
        return ti == txt.len();
    }
    match pat[pi] {
        '*' => {
            // Try matching * with 0, 1, 2, ... chars
            for skip in 0..=(txt.len() - ti) {
                if glob_match_helper(pat, txt, pi + 1, ti + skip) {
                    return true;
                }
            }
            false
        }
        '?' => {
            if ti < txt.len() {
                glob_match_helper(pat, txt, pi + 1, ti + 1)
            } else {
                false
            }
        }
        c => {
            if ti < txt.len() && txt[ti] == c {
                glob_match_helper(pat, txt, pi + 1, ti + 1)
            } else {
                false
            }
        }
    }
}

/// Expand a glob pattern to matching file paths.
fn expand_glob(pattern: &str) -> Vec<String> {
    let (dir, file_pattern) = match pattern.rsplit_once('/') {
        Some((d, f)) => (d.to_string(), f),
        None => (".".to_string(), pattern),
    };

    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut matches: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| glob_matches(file_pattern, name))
        .collect();

    matches.sort();

    if dir == "." {
        matches
    } else {
        matches
            .into_iter()
            .map(|f| format!("{}/{}", dir, f))
            .collect()
    }
}

// ========== Trace helpers ==========

// ANSI color codes
const C_RESET: &str = "\x1b[0m";
const C_BOLD: &str = "\x1b[1m";
const C_DIM: &str = "\x1b[2m";
const C_RED: &str = "\x1b[31m";
const C_GREEN: &str = "\x1b[32m";
const C_YELLOW: &str = "\x1b[33m";
const C_MAGENTA: &str = "\x1b[35m";
const C_CYAN: &str = "\x1b[36m";

/// Format a single value for trace display (compact, no colors).
fn trace_fmt_value(val: &Value) -> String {
    match val {
        Value::Str(s) => format!("\"{}\"", s),
        Value::Int(n) => format!("{}", n),
        Value::Output(s) => {
            let line_count = s.lines().count();
            if line_count <= 1 {
                let trimmed = s.trim_end();
                if trimmed.len() > 30 {
                    format!("<<{}...>>", &trimmed[..27])
                } else {
                    format!("<<{}>>", trimmed)
                }
            } else {
                format!("<<output {} lines>>", line_count)
            }
        }
    }
}

/// Format a single value for trace display with colors.
fn trace_fmt_value_colored(val: &Value) -> String {
    match val {
        Value::Str(s) => format!("{C_YELLOW}\"{}\"{C_RESET}", s),
        Value::Int(n) => format!("{C_CYAN}{}{C_RESET}", n),
        Value::Output(s) => {
            let line_count = s.lines().count();
            if line_count <= 1 {
                let trimmed = s.trim_end();
                if trimmed.len() > 30 {
                    format!("{C_MAGENTA}<<{C_RESET}{}...{C_MAGENTA}>>{C_RESET}", &trimmed[..27])
                } else {
                    format!("{C_MAGENTA}<<{C_RESET}{}{C_MAGENTA}>>{C_RESET}", trimmed)
                }
            } else {
                format!("{C_MAGENTA}<<output {} lines>>{C_RESET}", line_count)
            }
        }
    }
}

/// Format the stack contents for trace display with colors.
fn trace_fmt_stack(stack: &[Value]) -> String {
    if stack.is_empty() {
        format!("{C_DIM}(empty){C_RESET}")
    } else {
        stack
            .iter()
            .map(trace_fmt_value_colored)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Describe the diff between stack states before and after a token execution.
fn trace_describe_diff(before: &[Value], after: &[Value]) -> String {
    // Find common prefix length
    let common = before
        .iter()
        .zip(after.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let popped = &before[common..];
    let pushed = &after[common..];

    let mut parts = Vec::new();

    if !popped.is_empty() {
        let items: Vec<String> = popped.iter().rev().map(trace_fmt_value).collect();
        parts.push(format!("{C_RED}pop{C_RESET} {}", items.join(", ")));
    }

    if !pushed.is_empty() {
        let items: Vec<String> = pushed.iter().map(trace_fmt_value).collect();
        parts.push(format!("{C_GREEN}push{C_RESET} {}", items.join(", ")));
    }

    if parts.is_empty() {
        format!("{C_DIM}(no stack change){C_RESET}")
    } else {
        parts.join("; ")
    }
}

/// Print a trace line for a token evaluation step.
///
/// Verbosity levels:
///   1 = push/pop description only
///   2 = push/pop + stack state
///   3 = push/pop + doc string + stack state
fn trace_print_step(
    level: u8,
    step: usize,
    token: &str,
    is_quoted: bool,
    before: &[Value],
    after: &[Value],
    doc: Option<&str>,
) {
    let display_token = if is_quoted {
        format!("{C_YELLOW}\"{}\"", token)
    } else {
        format!("{C_BOLD}{}", token)
    };

    let desc = trace_describe_diff(before, after);

    eprintln!(
        "  {C_DIM}Step {}{C_RESET} {:<20}{C_RESET} \u{2192} {}",
        step, display_token, desc,
    );
    if level >= 3 {
        if let Some(doc) = doc {
            eprintln!(
                "  {C_DIM}{:>28} {}{C_RESET}",
                "", doc
            );
        }
    }
    if level >= 2 {
        let stack_display = trace_fmt_stack(after);
        eprintln!(
            "  {C_DIM}{:>28} Stack:{C_RESET} {}",
            "", stack_display
        );
    }
    let _ = std::io::stderr().flush();
}

/// Look up the doc string for a token from the dictionary.
fn trace_lookup_doc<'a>(state: &'a State, token: &str, is_quoted: bool) -> Option<&'a str> {
    if is_quoted {
        return None;
    }
    match state.dict.get(token)? {
        Word::Builtin(_, Some(doc)) => Some(doc),
        Word::Defined(_) => Some("(user-defined word)"),
        _ => None,
    }
}

// ========== Token evaluation ==========

/// Handle word definition collection (: name ... ;).
fn handle_word_definition(state: &mut State, token: &str) -> Result<(), String> {
    if let Some(ref name) = state.defining.clone() {
        if name == "UNNAMED" {
            // This token is the word name
            state.defining = Some(token.to_string());
        } else if token == ";" {
            // End definition
            let name = name.clone();
            let body = std::mem::take(&mut state.def_body);
            state.dict.insert(name, Word::Defined(body));
            state.defining = None;
        } else {
            // Accumulate token into body
            state.def_body.push(token.to_string());
        }
    }
    Ok(())
}

/// Handle control flow skipping (if/else/then nesting).
fn handle_control_flow_skipping(
    state: &mut State,
    token: &str,
    target: SkipTarget,
    depth: usize,
) -> Result<(), String> {
    if token == "if" {
        // Nested if: increase depth
        state.control_flow = ControlFlow::Skipping {
            target,
            depth: depth + 1,
        };
    } else if token == "then" {
        if depth == 0 {
            state.control_flow = ControlFlow::Normal;
        } else {
            state.control_flow = ControlFlow::Skipping {
                target,
                depth: depth - 1,
            };
        }
    } else if token == "else" {
        match (&target, depth) {
            (SkipTarget::Else, 0) => {
                // We were skipping the if-branch, now execute else-branch
                state.control_flow = ControlFlow::Normal;
            }
            (SkipTarget::Then, 0) => {
                // We executed the if-branch, keep skipping past else
                // (stay in Skipping state)
            }
            _ => {
                // Nested else, ignore
            }
        }
    }
    Ok(())
}

/// Handle control flow keywords. Returns Ok(true) if handled, Ok(false) if not a keyword.
fn handle_control_flow_keywords(state: &mut State, token: &str) -> Result<bool, String> {
    if token == "if" {
        // Pop condition from stack
        match state.stack.pop() {
            Some(Value::Int(0)) => {
                // False: skip to else or then
                state.control_flow = ControlFlow::Skipping {
                    target: SkipTarget::Else,
                    depth: 0,
                };
            }
            Some(Value::Int(_)) => {
                // True: continue normally
                state.control_flow = ControlFlow::Normal;
            }
            Some(_) => return Err("if: requires integer on stack".into()),
            None => return Err("if: stack underflow".into()),
        }
        Ok(true)
    } else if token == "else" {
        // We executed the if-branch, now skip to then
        state.control_flow = ControlFlow::Skipping {
            target: SkipTarget::Then,
            depth: 0,
        };
        Ok(true)
    } else if token == "then" {
        // End of if/then block
        state.control_flow = ControlFlow::Normal;
        Ok(true)
    } else if token == ":" {
        // Start word definition
        state.defining = Some("UNNAMED".to_string());
        Ok(true)
    } else if token == "begin" {
        // Start begin...until or begin...while...repeat loop
        state.collecting_loop = Some((LoopType::BeginUntil, Vec::new(), 0));
        Ok(true)
    } else if token == "do" {
        // Start do...loop or do...+loop
        state.collecting_loop = Some((LoopType::DoLoop, Vec::new(), 0));
        Ok(true)
    } else if token == "each" {
        // Start each...then - pop Output from stack
        match state.stack.pop() {
            Some(Value::Output(content)) => {
                state.collecting_each = Some((content, Vec::new()));
                Ok(true)
            }
            Some(_) => Err("each: requires Output on stack".into()),
            None => Err("each: stack underflow".into()),
        }
    } else if token == "until" {
        Err("until: no matching begin".into())
    } else if token == "repeat" {
        Err("repeat: no matching begin".into())
    } else if token == "loop" {
        Err("loop: no matching do".into())
    } else if token == "+loop" {
        Err("+loop: no matching do".into())
    } else {
        Ok(false)
    }
}

/// Handle execution of a single token (integers, dictionary lookup, PATH lookup, globs).
fn handle_token_execution(state: &mut State, token: &str, is_quoted: bool) -> Result<(), String> {
    // Integer?
    if !is_quoted && tokenizer::is_int(token) {
        let n: i64 = token.parse().unwrap();
        state.stack.push(Value::Int(n));
        return Ok(());
    }

    // Dictionary lookup (only for unquoted tokens)
    if !is_quoted {
        if let Some(word) = state.dict.get(token).cloned() {
            match word {
                Word::Builtin(f, _) => {
                    return f(state);
                }
                Word::Defined(tokens) => {
                    // Execute defined word: each token is unquoted
                    for t in &tokens {
                        eval_token(state, t, false)?;
                    }
                    return Ok(());
                }
                Word::ShellCmd(cmd) => {
                    state.stack.push(Value::Str(cmd));
                    return exec_word(state);
                }
            }
        }
    }

    // Quoted string: push as literal
    if is_quoted {
        state.stack.push(Value::Str(token.to_string()));
        return Ok(());
    }

    // Unquoted: try PATH lookup
    if let Some(full_path) = find_in_path(token) {
        state.stack.push(Value::Str(full_path));
        return exec_word(state);
    }

    // Glob expansion
    if has_glob_chars(token) {
        let matches = expand_glob(token);
        if !matches.is_empty() {
            for m in matches {
                state.stack.push(Value::Str(m));
            }
            return Ok(());
        }
    }

    // Otherwise: push as string literal
    state.stack.push(Value::Str(token.to_string()));
    Ok(())
}

/// Evaluate a single token within the current interpreter state.
pub fn eval_token(state: &mut State, token: &str, is_quoted: bool) -> Result<(), String> {
    // 1. Are we collecting an each...then body?
    if state.collecting_each.is_some() {
        return loops::handle_each_collection(state, token);
    }

    // 2. Are we collecting a loop body?
    if state.collecting_loop.is_some() {
        return loops::handle_loop_collection(state, token);
    }

    // 3. Are we defining a word?
    if state.defining.is_some() {
        return handle_word_definition(state, token);
    }

    // 4. Are we skipping (control flow)?
    if let ControlFlow::Skipping { ref target, depth } = state.control_flow.clone() {
        return handle_control_flow_skipping(state, token, target.clone(), depth);
    }

    // Trace: snapshot stack before execution
    let trace_level = state.trace;
    let stack_before = if trace_level > 0 {
        Some(state.stack.clone())
    } else {
        None
    };

    // 5. Is it a control flow keyword?
    if !is_quoted && handle_control_flow_keywords(state, token)? {
        if let Some(before) = stack_before {
            state.trace_step += 1;
            let doc = trace_lookup_doc(state, token, is_quoted);
            trace_print_step(trace_level, state.trace_step, token, is_quoted, &before, &state.stack, doc);
        }
        return Ok(());
    }

    // 6. Execute normally
    let result = handle_token_execution(state, token, is_quoted);

    // Trace: print step after execution
    if let Some(before) = stack_before {
        state.trace_step += 1;
        let doc = trace_lookup_doc(state, token, is_quoted);
        trace_print_step(trace_level, state.trace_step, token, is_quoted, &before, &state.stack, doc);
    }

    result
}

/// Evaluate a full line of input.
pub fn eval_line(state: &mut State, line: &str) -> Result<(), String> {
    // Reset trace step counter for each new line
    state.trace_step = 0;

    let tokens = tokenizer::tokenize(line);

    // Handle special `: name` prefix -- consume name early
    if tokens.len() >= 2 && tokens[0].text == ":" && !tokens[0].quoted {
        state.defining = Some(tokens[1].text.clone());
        state.def_body.clear();
        for token in &tokens[2..] {
            eval_token(state, &token.text, token.quoted)?;
        }
        return Ok(());
    }

    // Normal evaluation
    for token in &tokens {
        eval_token(state, &token.text, token.quoted)?;
    }
    Ok(())
}

use std::io::Write;
use std::process::{Command, Stdio};

use crate::types::{State, Value};

/// `exec` ( args... cmd -- output ) Execute shell command with arguments from stack.
///
/// Stack layout: top is the command, below it are arguments and optional depth limit.
/// - `Output` values on the stack are concatenated and piped as stdin.
/// - `Str` and `Int` values are collected as command arguments.
/// - An `Int` immediately after the command name acts as a depth limit.
pub fn exec_word(state: &mut State) -> Result<(), String> {
    // Pop the command name
    let cmd = match state.stack.pop() {
        Some(Value::Str(s)) => s,
        Some(other) => {
            state.stack.push(other);
            return Err("exec: top of stack must be a string (command name)".into());
        }
        None => return Err("exec: stack underflow".into()),
    };

    // Check for optional depth limit (Int immediately below command)
    let depth_limit = match state.stack.last() {
        Some(Value::Int(n)) => {
            let n = *n;
            state.stack.pop();
            Some(n as usize)
        }
        _ => None,
    };

    // Collect arguments (Str/Int) and stdin data (Output) from stack
    let mut cmd_args: Vec<String> = Vec::new();
    let mut stdin_parts: Vec<String> = Vec::new();
    let mut remaining: Vec<Value> = Vec::new();
    let mut count = 0usize;

    // Drain from top of stack (which is the end of the vec)
    while let Some(val) = state.stack.pop() {
        match val {
            Value::Str(s) => {
                if depth_limit.is_some_and(|limit| count >= limit) {
                    remaining.push(Value::Str(s));
                    // Collect remaining items
                    while let Some(v) = state.stack.pop() {
                        remaining.push(v);
                    }
                    break;
                }
                cmd_args.push(s);
                count += 1;
            }
            Value::Int(n) => {
                if depth_limit.is_some_and(|limit| count >= limit) {
                    remaining.push(Value::Int(n));
                    while let Some(v) = state.stack.pop() {
                        remaining.push(v);
                    }
                    break;
                }
                cmd_args.push(n.to_string());
                count += 1;
            }
            Value::Output(s) => {
                stdin_parts.push(s);
            }
        }
    }

    // Remaining items go back on the stack (they were popped in reverse)
    for val in remaining.into_iter().rev() {
        state.stack.push(val);
    }

    // Args were collected top-to-bottom, but should be bottom-to-top for command
    cmd_args.reverse();

    // Concatenate stdin data
    let stdin_data: String = stdin_parts.into_iter().rev().collect();
    let has_stdin = !stdin_data.is_empty();

    // Execute
    let result = if has_stdin {
        // Spawn with piped stdin
        let child = Command::new(&cmd)
            .args(&cmd_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn();

        match child {
            Ok(mut child) => {
                // Write stdin data
                if let Some(mut stdin) = child.stdin.take() {
                    let data = stdin_data;
                    // Write in a thread to avoid deadlock
                    std::thread::spawn(move || {
                        let _ = stdin.write_all(data.as_bytes());
                    });
                }
                child
                    .wait_with_output()
                    .map_err(|e| format!("exec: {}", e))
            }
            Err(e) => Err(format!("exec: {}: {}", cmd, e)),
        }
    } else {
        // Simple execution without stdin
        Command::new(&cmd)
            .args(&cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| format!("exec: {}: {}", cmd, e))
    };

    match result {
        Ok(output) => {
            state.last_exit_code = output.status.code().unwrap_or(128);
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            state.stack.push(Value::Output(stdout));
            Ok(())
        }
        Err(e) => {
            state.last_exit_code = 127;
            Err(e)
        }
    }
}

/// `?` ( -- code ) Push exit code of last command.
pub fn exit_code(state: &mut State) -> Result<(), String> {
    state.stack.push(Value::Int(state.last_exit_code as i64));
    Ok(())
}

/// `cd` ( path -- ) Change directory.
pub fn cd(state: &mut State) -> Result<(), String> {
    let val = state.stack.pop().ok_or("cd: stack underflow")?;
    match val {
        Value::Str(path) => {
            let expanded = expand_tilde(&path);
            std::env::set_current_dir(&expanded)
                .map_err(|e| format!("cd: {}: {}", expanded, e))
        }
        _ => Err("cd: requires string".into()),
    }
}

/// Expand `~` to $HOME at the start of a path.
fn expand_tilde(path: &str) -> String {
    if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, &path[1..]);
        }
    }
    path.to_string()
}

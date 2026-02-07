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

// ========== Environment variables ==========

/// `getenv` ( key -- value ) Get environment variable (empty string if unset).
pub fn getenv(state: &mut State) -> Result<(), String> {
    let val = state.stack.pop().ok_or("getenv: stack underflow")?;
    match val {
        Value::Str(key) => {
            let value = std::env::var(&key).unwrap_or_default();
            state.stack.push(Value::Str(value));
            Ok(())
        }
        other => {
            state.stack.push(other);
            Err("getenv: requires string".into())
        }
    }
}

/// `setenv` ( value key -- ) Set environment variable.
pub fn setenv(state: &mut State) -> Result<(), String> {
    if state.stack.len() < 2 {
        return Err("setenv: stack underflow".into());
    }
    let key = state.stack.pop().unwrap();
    let value = state.stack.pop().unwrap();
    match (value, key) {
        (Value::Str(v), Value::Str(k)) => {
            std::env::set_var(&k, &v);
            Ok(())
        }
        (v, k) => {
            state.stack.push(v);
            state.stack.push(k);
            Err("setenv: requires two strings (value key)".into())
        }
    }
}

/// `unsetenv` ( key -- ) Unset environment variable.
pub fn unsetenv(state: &mut State) -> Result<(), String> {
    let val = state.stack.pop().ok_or("unsetenv: stack underflow")?;
    match val {
        Value::Str(key) => {
            std::env::remove_var(&key);
            Ok(())
        }
        other => {
            state.stack.push(other);
            Err("unsetenv: requires string".into())
        }
    }
}

/// `env-append` ( value key -- ) Append value to colon-separated env var.
pub fn env_append(state: &mut State) -> Result<(), String> {
    if state.stack.len() < 2 {
        return Err("env-append: stack underflow".into());
    }
    let key = state.stack.pop().unwrap();
    let value = state.stack.pop().unwrap();
    match (value, key) {
        (Value::Str(v), Value::Str(k)) => {
            let new_value = match std::env::var(&k) {
                Ok(existing) => format!("{}:{}", existing, v),
                Err(_) => v,
            };
            std::env::set_var(&k, &new_value);
            Ok(())
        }
        (v, k) => {
            state.stack.push(v);
            state.stack.push(k);
            Err("env-append: requires two strings (value key)".into())
        }
    }
}

/// `env-prepend` ( value key -- ) Prepend value to colon-separated env var.
pub fn env_prepend(state: &mut State) -> Result<(), String> {
    if state.stack.len() < 2 {
        return Err("env-prepend: stack underflow".into());
    }
    let key = state.stack.pop().unwrap();
    let value = state.stack.pop().unwrap();
    match (value, key) {
        (Value::Str(v), Value::Str(k)) => {
            let new_value = match std::env::var(&k) {
                Ok(existing) => format!("{}:{}", v, existing),
                Err(_) => v,
            };
            std::env::set_var(&k, &new_value);
            Ok(())
        }
        (v, k) => {
            state.stack.push(v);
            state.stack.push(k);
            Err("env-prepend: requires two strings (value key)".into())
        }
    }
}

/// `env` ( -- vars... ) Push all environment variables onto stack.
pub fn env_all(state: &mut State) -> Result<(), String> {
    let mut vars: Vec<String> = std::env::vars()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    vars.sort();
    for entry in vars {
        state.stack.push(Value::Str(entry));
    }
    Ok(())
}

// ========== Directory navigation ==========

/// `pushd` ( path -- ) Push current directory and change to path.
pub fn pushd(state: &mut State) -> Result<(), String> {
    let val = state.stack.pop().ok_or("pushd: stack underflow")?;
    match val {
        Value::Str(path) => {
            let current = std::env::current_dir()
                .map_err(|e| format!("pushd: {}", e))?
                .to_string_lossy()
                .to_string();
            let expanded = expand_tilde(&path);
            std::env::set_current_dir(&expanded)
                .map_err(|e| format!("pushd: {}: {}", expanded, e))?;
            state.dir_stack.push(current);
            Ok(())
        }
        other => {
            state.stack.push(other);
            Err("pushd: requires string".into())
        }
    }
}

/// `popd` ( -- ) Pop directory from stack and change to it.
pub fn popd(state: &mut State) -> Result<(), String> {
    let dir = state.dir_stack.pop().ok_or("popd: directory stack empty")?;
    std::env::set_current_dir(&dir)
        .map_err(|e| format!("popd: {}: {}", dir, e))
}

/// Expand `~` to $HOME at the start of a path.
fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix('~') {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, rest);
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins;

    fn new_state() -> State {
        let mut s = State::new();
        builtins::register_builtins(&mut s);
        s
    }

    #[test]
    fn test_exec_echo() {
        let mut s = new_state();
        s.stack.push(Value::Str("hello".into()));
        s.stack.push(Value::Str("/bin/echo".into()));
        exec_word(&mut s).unwrap();
        assert_eq!(s.last_exit_code, 0);
        match &s.stack[0] {
            Value::Output(out) => assert_eq!(out.trim(), "hello"),
            other => panic!("expected Output, got {:?}", other),
        }
    }

    #[test]
    fn test_exec_with_stdin() {
        let mut s = new_state();
        s.stack.push(Value::Output("hello world\n".into()));
        s.stack.push(Value::Str("-c".into()));
        s.stack.push(Value::Str("/usr/bin/wc".into()));
        exec_word(&mut s).unwrap();
        assert_eq!(s.last_exit_code, 0);
        // wc -c counts bytes: "hello world\n" = 12
        match &s.stack[0] {
            Value::Output(out) => {
                let n: i64 = out.trim().parse().unwrap();
                assert_eq!(n, 12);
            }
            other => panic!("expected Output, got {:?}", other),
        }
    }

    #[test]
    fn test_exec_not_found() {
        let mut s = new_state();
        s.stack.push(Value::Str("/nonexistent/binary".into()));
        assert!(exec_word(&mut s).is_err());
        assert_eq!(s.last_exit_code, 127);
    }

    #[test]
    fn test_exec_underflow() {
        let mut s = new_state();
        assert!(exec_word(&mut s).is_err());
    }

    #[test]
    fn test_exit_code() {
        let mut s = new_state();
        s.last_exit_code = 42;
        exit_code(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(42)]);
    }

    #[test]
    fn test_exit_code_after_failure() {
        let mut s = new_state();
        s.stack.push(Value::Str("/bin/false".into()));
        exec_word(&mut s).unwrap();
        s.stack.clear();
        exit_code(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_cd_underflow() {
        let mut s = new_state();
        assert!(cd(&mut s).is_err());
    }

    #[test]
    fn test_cd_bad_type() {
        let mut s = new_state();
        s.stack.push(Value::Int(42));
        assert!(cd(&mut s).is_err());
    }

    #[test]
    fn test_expand_tilde() {
        let home = std::env::var("HOME").unwrap();
        assert_eq!(expand_tilde("~/foo"), format!("{}/foo", home));
        assert_eq!(expand_tilde("/abs/path"), "/abs/path");
        assert_eq!(expand_tilde("relative"), "relative");
    }

    #[test]
    fn test_exec_depth_limit() {
        let mut s = new_state();
        // Stack: "extra" "hello" then depth=1 then cmd
        s.stack.push(Value::Str("extra".into()));
        s.stack.push(Value::Str("hello".into()));
        s.stack.push(Value::Int(1)); // depth limit: take only 1 arg
        s.stack.push(Value::Str("/bin/echo".into()));
        exec_word(&mut s).unwrap();
        // "extra" should remain on stack, only "hello" consumed
        assert_eq!(s.stack.len(), 2); // remaining "extra" + Output
        assert_eq!(s.stack[0], Value::Str("extra".into()));
        match &s.stack[1] {
            Value::Output(out) => assert_eq!(out.trim(), "hello"),
            other => panic!("expected Output, got {:?}", other),
        }
    }

    // ===== Environment variable tests =====

    #[test]
    fn test_getenv_existing() {
        let mut s = new_state();
        // HOME is always set
        s.stack.push(Value::Str("HOME".into()));
        getenv(&mut s).unwrap();
        assert_eq!(s.stack.len(), 1);
        match &s.stack[0] {
            Value::Str(v) => assert!(!v.is_empty()),
            other => panic!("expected Str, got {:?}", other),
        }
    }

    #[test]
    fn test_getenv_nonexistent() {
        let mut s = new_state();
        s.stack.push(Value::Str("YAFSH_TEST_NONEXISTENT_VAR".into()));
        getenv(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("".into())]);
    }

    #[test]
    fn test_getenv_underflow() {
        let mut s = new_state();
        assert!(getenv(&mut s).is_err());
    }

    #[test]
    fn test_getenv_wrong_type() {
        let mut s = new_state();
        s.stack.push(Value::Int(42));
        assert!(getenv(&mut s).is_err());
    }

    #[test]
    fn test_setenv_and_getenv() {
        let mut s = new_state();
        s.stack.push(Value::Str("test_value_42".into()));
        s.stack.push(Value::Str("YAFSH_TEST_SET".into()));
        setenv(&mut s).unwrap();
        assert!(s.stack.is_empty());

        s.stack.push(Value::Str("YAFSH_TEST_SET".into()));
        getenv(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("test_value_42".into())]);

        // Cleanup
        std::env::remove_var("YAFSH_TEST_SET");
    }

    #[test]
    fn test_setenv_underflow() {
        let mut s = new_state();
        s.stack.push(Value::Str("value".into()));
        assert!(setenv(&mut s).is_err());
    }

    #[test]
    fn test_unsetenv() {
        let mut s = new_state();
        std::env::set_var("YAFSH_TEST_UNSET", "temp");
        s.stack.push(Value::Str("YAFSH_TEST_UNSET".into()));
        unsetenv(&mut s).unwrap();
        assert!(std::env::var("YAFSH_TEST_UNSET").is_err());
    }

    #[test]
    fn test_unsetenv_underflow() {
        let mut s = new_state();
        assert!(unsetenv(&mut s).is_err());
    }

    #[test]
    fn test_env_append() {
        let mut s = new_state();
        std::env::set_var("YAFSH_TEST_APPEND", "a");
        s.stack.push(Value::Str("b".into()));
        s.stack.push(Value::Str("YAFSH_TEST_APPEND".into()));
        env_append(&mut s).unwrap();
        assert_eq!(std::env::var("YAFSH_TEST_APPEND").unwrap(), "a:b");
        std::env::remove_var("YAFSH_TEST_APPEND");
    }

    #[test]
    fn test_env_prepend() {
        let mut s = new_state();
        std::env::set_var("YAFSH_TEST_PREPEND", "a");
        s.stack.push(Value::Str("b".into()));
        s.stack.push(Value::Str("YAFSH_TEST_PREPEND".into()));
        env_prepend(&mut s).unwrap();
        assert_eq!(std::env::var("YAFSH_TEST_PREPEND").unwrap(), "b:a");
        std::env::remove_var("YAFSH_TEST_PREPEND");
    }

    #[test]
    fn test_env_all() {
        let mut s = new_state();
        env_all(&mut s).unwrap();
        // Should push at least one env var
        assert!(!s.stack.is_empty());
        // All entries should be Str and contain '='
        for val in &s.stack {
            match val {
                Value::Str(entry) => assert!(entry.contains('=')),
                other => panic!("expected Str, got {:?}", other),
            }
        }
    }

    // ===== pushd/popd tests =====

    #[test]
    fn test_pushd_popd_round_trip() {
        let mut s = new_state();
        let original = std::env::current_dir().unwrap();

        s.stack.push(Value::Str("/tmp".into()));
        pushd(&mut s).unwrap();
        assert_eq!(
            std::env::current_dir().unwrap().to_string_lossy(),
            "/tmp"
        );
        assert_eq!(s.dir_stack.len(), 1);

        popd(&mut s).unwrap();
        assert_eq!(std::env::current_dir().unwrap(), original);
        assert!(s.dir_stack.is_empty());
    }

    #[test]
    fn test_pushd_underflow() {
        let mut s = new_state();
        assert!(pushd(&mut s).is_err());
    }

    #[test]
    fn test_pushd_wrong_type() {
        let mut s = new_state();
        s.stack.push(Value::Int(42));
        assert!(pushd(&mut s).is_err());
    }

    #[test]
    fn test_popd_empty_stack() {
        let mut s = new_state();
        assert!(popd(&mut s).is_err());
    }
}

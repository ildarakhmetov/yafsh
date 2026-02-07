use std::process::Command;

use crate::types::{State, Value, Word};

/// `words` ( -- ) List all available words in the dictionary.
pub fn words(state: &mut State) -> Result<(), String> {
    let mut names: Vec<&String> = state.dict.keys().collect();
    names.sort();
    for name in &names {
        print!("{} ", name);
    }
    println!();
    Ok(())
}

/// `help` ( -- ) Show comprehensive help information.
pub fn help(_state: &mut State) -> Result<(), String> {
    println!("Forth Shell - Available Commands");
    println!();
    println!("Stack Operations:");
    println!("  dup swap drop over rot    - manipulate stack");
    println!("  .s                        - show stack contents");
    println!();
    println!("Printing:");
    println!("  .                         - print top of stack");
    println!("  type                      - print without newline");
    println!();
    println!("Arithmetic:");
    println!("  + - * / mod /mod */       - math operations");
    println!("  = < > <= >= <>            - comparisons");
    println!();
    println!("Boolean Logic:");
    println!("  and or not xor            - boolean operations");
    println!();
    println!("String Operations:");
    println!("  concat                    - concatenate two strings");
    println!();
    println!("Control Flow:");
    println!("  if ... then               - conditional");
    println!("  if ... else ... then      - conditional with else");
    println!();
    println!("Loops:");
    println!("  begin ... until           - loop until condition is true");
    println!("  begin ... while ... repeat - loop while condition is true");
    println!("  start limit do ... loop   - counted loop (step 1)");
    println!("  start limit do ... +loop  - counted loop (step from stack)");
    println!("  output each ... then      - iterate over output lines");
    println!("  i j                       - loop indices");
    println!();
    println!("Word Definition:");
    println!("  : name ... ;              - define new word");
    println!();
    println!("Type Conversions:");
    println!("  >output >string           - convert between types");
    println!();
    println!("File I/O:");
    println!("  >file >>file              - write/append output to file");
    println!();
    println!("Environment:");
    println!("  getenv setenv unsetenv    - environment variables");
    println!();
    println!("Directory:");
    println!("  cd pushd popd             - directory navigation");
    println!();
    println!("Help System:");
    println!("  words                     - list all words");
    println!("  \"word\" see                - show word definition");
    println!("  help                      - show this help");
    println!();
    println!("Type 'words' to see all available commands");
    Ok(())
}

/// `see` ( name -- ) Show the definition or documentation for a word.
///
/// Pops a string from the stack and looks it up in the dictionary.
pub fn see(state: &mut State) -> Result<(), String> {
    let val = state.stack.pop().ok_or("see: stack underflow")?;
    let name = match val {
        Value::Str(s) => s,
        other => {
            state.stack.push(other);
            return Err("see: requires string (word name)".into());
        }
    };

    match state.dict.get(&name) {
        Some(Word::Builtin(_, Some(doc))) => {
            println!("{}: {}", name, doc);
        }
        Some(Word::Builtin(_, None)) => {
            println!("{} is a builtin function", name);
        }
        Some(Word::Defined(tokens)) => {
            print!(": {} ", name);
            for t in tokens {
                print!("{} ", t);
            }
            println!(";");
        }
        Some(Word::ShellCmd(cmd)) => {
            println!("{} is a shell command: {}", name, cmd);
        }
        None => {
            println!("{} is not defined", name);
        }
    }
    Ok(())
}

// ========== Prompt helper builtins ==========

/// Helper: get the stack to inspect for prompt helpers.
/// During prompt evaluation, uses the saved original stack; otherwise uses the current stack.
fn prompt_stack(state: &State) -> &[Value] {
    state
        .prompt_eval_original_stack
        .as_deref()
        .unwrap_or(&state.stack)
}

/// Count inputs (Str/Int) vs outputs (Output) on a stack slice.
fn count_stack(stack: &[Value]) -> (usize, usize) {
    let mut inputs = 0;
    let mut outputs = 0;
    for val in stack {
        match val {
            Value::Str(_) | Value::Int(_) => inputs += 1,
            Value::Output(_) => outputs += 1,
        }
    }
    (inputs, outputs)
}

/// `$stack` ( -- str ) Push formatted `[n:m]` stack indicator.
pub fn dollar_stack(state: &mut State) -> Result<(), String> {
    let stack = prompt_stack(state);
    let (inputs, outputs) = count_stack(stack);
    let total = inputs + outputs;

    let indicator = if total == 0 {
        String::new()
    } else if outputs == 0 {
        format!("[{}]", inputs)
    } else if inputs == 0 {
        format!("[:{}]", outputs)
    } else {
        format!("[{}:{}]", inputs, outputs)
    };
    state.stack.push(Value::Str(indicator));
    Ok(())
}

/// `$in` ( -- int ) Push count of input items on the stack.
pub fn dollar_in(state: &mut State) -> Result<(), String> {
    let stack = prompt_stack(state);
    let (inputs, _) = count_stack(stack);
    state.stack.push(Value::Int(inputs as i64));
    Ok(())
}

/// `$out` ( -- int ) Push count of output items on the stack.
pub fn dollar_out(state: &mut State) -> Result<(), String> {
    let stack = prompt_stack(state);
    let (_, outputs) = count_stack(stack);
    state.stack.push(Value::Int(outputs as i64));
    Ok(())
}

/// `$gitbranch` ( -- str ) Push current git branch name (empty if not in a git repo).
pub fn dollar_gitbranch(state: &mut State) -> Result<(), String> {
    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();
    state.stack.push(Value::Str(branch));
    Ok(())
}

/// `$cwd` ( -- str ) Push the current working directory.
pub fn dollar_cwd(state: &mut State) -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "?".to_string());
    state.stack.push(Value::Str(cwd));
    Ok(())
}

/// `$basename` ( -- str ) Push the basename of the current working directory.
pub fn dollar_basename(state: &mut State) -> Result<(), String> {
    let basename = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
        .unwrap_or_else(|| "?".to_string());
    state.stack.push(Value::Str(basename));
    Ok(())
}

/// `$hostname` ( -- str ) Push the system hostname.
pub fn dollar_hostname(state: &mut State) -> Result<(), String> {
    let hostname = Command::new("hostname")
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    state.stack.push(Value::Str(hostname));
    Ok(())
}

/// `$username` ( -- str ) Push the current username.
pub fn dollar_username(state: &mut State) -> Result<(), String> {
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    state.stack.push(Value::Str(username));
    Ok(())
}

/// `$exitcode` ( -- str ) Push the last exit code as a string.
pub fn dollar_exitcode(state: &mut State) -> Result<(), String> {
    state
        .stack
        .push(Value::Str(state.last_exit_code.to_string()));
    Ok(())
}

/// `$time` ( -- str ) Push current time as HH:MM.
pub fn dollar_time(state: &mut State) -> Result<(), String> {
    let time_str = Command::new("date")
        .arg("+%H:%M")
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|| "??:??".to_string());
    state.stack.push(Value::Str(time_str));
    Ok(())
}

/// `trace` ( level -- ) Set trace verbosity level.
///
/// Accepts a string or integer:
///   "off" or 0 -- disable tracing
///   "on"  or 2 -- normal tracing (push/pop + stack)
///   1          -- minimal (push/pop only)
///   3          -- verbose (push/pop + doc strings + stack)
pub fn trace_mode(state: &mut State) -> Result<(), String> {
    let val = state.stack.pop().ok_or("trace: stack underflow")?;
    match val {
        Value::Str(s) => match s.as_str() {
            "on" => {
                state.trace = 2;
                eprintln!("Trace mode ON (level 2)");
                Ok(())
            }
            "off" => {
                state.trace = 0;
                eprintln!("Trace mode OFF");
                Ok(())
            }
            _ => Err("trace: expected \"on\", \"off\", or 0-3".into()),
        },
        Value::Int(n) if (0..=3).contains(&n) => {
            state.trace = n as u8;
            if n == 0 {
                eprintln!("Trace mode OFF");
            } else {
                eprintln!("Trace mode level {}", n);
            }
            Ok(())
        }
        Value::Int(_) => Err("trace: level must be 0-3".into()),
        other => {
            state.stack.push(other);
            Err("trace: expected string or integer (0-3)".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins;
    use crate::types::Value;

    fn new_state() -> State {
        let mut s = State::new();
        builtins::register_builtins(&mut s);
        s
    }

    #[test]
    fn test_words_runs_without_error() {
        let mut s = new_state();
        words(&mut s).unwrap();
        // Stack should be unmodified
        assert!(s.stack.is_empty());
    }

    #[test]
    fn test_help_runs_without_error() {
        let mut s = new_state();
        help(&mut s).unwrap();
        assert!(s.stack.is_empty());
    }

    #[test]
    fn test_see_builtin_with_doc() {
        let mut s = new_state();
        s.stack.push(Value::Str("dup".into()));
        see(&mut s).unwrap();
        assert!(s.stack.is_empty());
    }

    #[test]
    fn test_see_defined_word() {
        let mut s = new_state();
        s.dict.insert(
            "greet".to_string(),
            Word::Defined(vec!["\"hello\"".to_string()]),
        );
        s.stack.push(Value::Str("greet".into()));
        see(&mut s).unwrap();
        assert!(s.stack.is_empty());
    }

    #[test]
    fn test_see_undefined() {
        let mut s = new_state();
        s.stack.push(Value::Str("nonexistent".into()));
        see(&mut s).unwrap(); // Should not error, just print "not defined"
        assert!(s.stack.is_empty());
    }

    #[test]
    fn test_see_underflow() {
        let mut s = new_state();
        assert!(see(&mut s).is_err());
    }

    #[test]
    fn test_see_wrong_type() {
        let mut s = new_state();
        s.stack.push(Value::Int(42));
        assert!(see(&mut s).is_err());
    }

    // ===== Prompt helper tests =====

    #[test]
    fn test_dollar_stack_empty() {
        let mut s = new_state();
        dollar_stack(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("".into())]);
    }

    #[test]
    fn test_dollar_stack_inputs_only() {
        let mut s = new_state();
        s.stack.push(Value::Int(1));
        s.stack.push(Value::Str("x".into()));
        dollar_stack(&mut s).unwrap();
        // 2 inputs, then the result string
        assert_eq!(s.stack.len(), 3);
        assert_eq!(s.stack[2], Value::Str("[2]".into()));
    }

    #[test]
    fn test_dollar_stack_outputs_only() {
        let mut s = new_state();
        s.stack.push(Value::Output("data".into()));
        dollar_stack(&mut s).unwrap();
        assert_eq!(s.stack.len(), 2);
        assert_eq!(s.stack[1], Value::Str("[:1]".into()));
    }

    #[test]
    fn test_dollar_stack_mixed() {
        let mut s = new_state();
        s.stack.push(Value::Int(1));
        s.stack.push(Value::Output("data".into()));
        dollar_stack(&mut s).unwrap();
        assert_eq!(s.stack.len(), 3);
        assert_eq!(s.stack[2], Value::Str("[1:1]".into()));
    }

    #[test]
    fn test_dollar_in() {
        let mut s = new_state();
        s.stack.push(Value::Int(1));
        s.stack.push(Value::Str("x".into()));
        s.stack.push(Value::Output("data".into()));
        dollar_in(&mut s).unwrap();
        assert_eq!(s.stack.len(), 4);
        assert_eq!(s.stack[3], Value::Int(2));
    }

    #[test]
    fn test_dollar_out() {
        let mut s = new_state();
        s.stack.push(Value::Output("data".into()));
        dollar_out(&mut s).unwrap();
        assert_eq!(s.stack.len(), 2);
        assert_eq!(s.stack[1], Value::Int(1));
    }

    #[test]
    fn test_dollar_cwd() {
        let mut s = new_state();
        dollar_cwd(&mut s).unwrap();
        assert_eq!(s.stack.len(), 1);
        match &s.stack[0] {
            Value::Str(v) => assert!(!v.is_empty()),
            other => panic!("expected Str, got {:?}", other),
        }
    }

    #[test]
    fn test_dollar_basename() {
        let mut s = new_state();
        dollar_basename(&mut s).unwrap();
        assert_eq!(s.stack.len(), 1);
        match &s.stack[0] {
            Value::Str(v) => assert!(!v.is_empty()),
            other => panic!("expected Str, got {:?}", other),
        }
    }

    #[test]
    fn test_dollar_username() {
        let mut s = new_state();
        dollar_username(&mut s).unwrap();
        assert_eq!(s.stack.len(), 1);
        match &s.stack[0] {
            Value::Str(v) => assert!(!v.is_empty()),
            other => panic!("expected Str, got {:?}", other),
        }
    }

    #[test]
    fn test_dollar_exitcode() {
        let mut s = new_state();
        s.last_exit_code = 42;
        dollar_exitcode(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("42".into())]);
    }

    #[test]
    fn test_dollar_exitcode_zero() {
        let mut s = new_state();
        dollar_exitcode(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("0".into())]);
    }

    #[test]
    fn test_dollar_gitbranch() {
        let mut s = new_state();
        // Just verify it runs without error (may or may not be in a git repo)
        dollar_gitbranch(&mut s).unwrap();
        assert_eq!(s.stack.len(), 1);
    }

    #[test]
    fn test_dollar_hostname() {
        let mut s = new_state();
        dollar_hostname(&mut s).unwrap();
        assert_eq!(s.stack.len(), 1);
        match &s.stack[0] {
            Value::Str(v) => assert!(!v.is_empty()),
            other => panic!("expected Str, got {:?}", other),
        }
    }

    #[test]
    fn test_dollar_time() {
        let mut s = new_state();
        dollar_time(&mut s).unwrap();
        assert_eq!(s.stack.len(), 1);
        match &s.stack[0] {
            Value::Str(v) => {
                // Should match HH:MM pattern
                assert!(v.contains(':'), "time should contain colon: {}", v);
            }
            other => panic!("expected Str, got {:?}", other),
        }
    }

    #[test]
    fn test_dollar_stack_uses_original_during_prompt_eval() {
        let mut s = new_state();
        // Simulate prompt evaluation: real stack has 3 items
        s.prompt_eval_original_stack = Some(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Output("x".into()),
        ]);
        // Current stack is empty (cleared for prompt eval)
        dollar_stack(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("[2:1]".into())]);
    }

    #[test]
    fn test_dollar_in_uses_original_during_prompt_eval() {
        let mut s = new_state();
        s.prompt_eval_original_stack = Some(vec![Value::Int(1), Value::Str("x".into())]);
        dollar_in(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(2)]);
    }

    #[test]
    fn test_dollar_out_uses_original_during_prompt_eval() {
        let mut s = new_state();
        s.prompt_eval_original_stack = Some(vec![Value::Output("data".into())]);
        dollar_out(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }
}

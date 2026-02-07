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
}

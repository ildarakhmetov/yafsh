use std::fs::OpenOptions;
use std::io::Write;

use crate::types::{State, Value};
#[cfg(test)]
use crate::builtins;


/// `.` ( a -- ) Print and remove top item with newline.
pub fn dot(state: &mut State) -> Result<(), String> {
    let val = state.stack.pop().ok_or(".: stack underflow")?;
    println!("{}", val);
    Ok(())
}

/// `type` ( a -- ) Print and remove top item without newline.
pub fn type_word(state: &mut State) -> Result<(), String> {
    let val = state.stack.pop().ok_or("type: stack underflow")?;
    print!("{}", val);
    Ok(())
}

/// `.s` ( -- ) Display entire stack without modifying it.
pub fn dot_s(state: &mut State) -> Result<(), String> {
    print!("<{}> ", state.stack.len());
    for val in &state.stack {
        match val {
            Value::Str(s) => print!("\"{}\" ", s),
            Value::Int(n) => print!("{} ", n),
            Value::Output(s) => print!("«{}» ", s.trim_end()),
        }
    }
    println!();
    Ok(())
}

/// `>output` ( string -- output ) Convert Str to Output for piping.
pub fn to_output(state: &mut State) -> Result<(), String> {
    let val = state.stack.pop().ok_or(">output: stack underflow")?;
    match val {
        Value::Str(s) => {
            state.stack.push(Value::Output(s));
            Ok(())
        }
        Value::Output(_) => {
            // Already an output, push back
            state.stack.push(val);
            Ok(())
        }
        Value::Int(_) => Err(">output: requires string".into()),
    }
}

/// `>string` ( output/int -- string ) Convert Output or Int to Str.
pub fn to_string_word(state: &mut State) -> Result<(), String> {
    let val = state.stack.pop().ok_or(">string: stack underflow")?;
    match val {
        Value::Output(s) => {
            state.stack.push(Value::Str(s));
            Ok(())
        }
        Value::Int(n) => {
            state.stack.push(Value::Str(n.to_string()));
            Ok(())
        }
        Value::Str(_) => {
            // Already a string, push back
            state.stack.push(val);
            Ok(())
        }
    }
}

// ========== File I/O ==========

/// `>file` ( content filename -- ) Write output to file (create/truncate).
pub fn write_file(state: &mut State) -> Result<(), String> {
    if state.stack.len() < 2 {
        return Err(">file: stack underflow".into());
    }
    let filename = state.stack.pop().unwrap();
    let content = state.stack.pop().unwrap();
    match (content, filename) {
        (Value::Output(data), Value::Str(path)) => {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .map_err(|e| format!(">file: {}: {}", path, e))?;
            file.write_all(data.as_bytes())
                .map_err(|e| format!(">file: {}: {}", path, e))
        }
        (c, f) => {
            state.stack.push(c);
            state.stack.push(f);
            Err(">file: requires output and filename string".into())
        }
    }
}

/// `>>file` ( content filename -- ) Append output to file (create if needed).
pub fn append_file(state: &mut State) -> Result<(), String> {
    if state.stack.len() < 2 {
        return Err(">>file: stack underflow".into());
    }
    let filename = state.stack.pop().unwrap();
    let content = state.stack.pop().unwrap();
    match (content, filename) {
        (Value::Output(data), Value::Str(path)) => {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)
                .map_err(|e| format!(">>file: {}: {}", path, e))?;
            file.write_all(data.as_bytes())
                .map_err(|e| format!(">>file: {}: {}", path, e))
        }
        (c, f) => {
            state.stack.push(c);
            state.stack.push(f);
            Err(">>file: requires output and filename string".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with(vals: Vec<Value>) -> State {
        let mut s = State::new();
        builtins::register_builtins(&mut s);
        s.stack = vals;
        s
    }

    // dot and type_word print to stdout -- we test they pop correctly
    #[test]
    fn test_dot_pops() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(2)]);
        dot(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_dot_underflow() {
        let mut s = state_with(vec![]);
        assert!(dot(&mut s).is_err());
    }

    #[test]
    fn test_type_word_pops() {
        let mut s = state_with(vec![Value::Str("hi".into())]);
        type_word(&mut s).unwrap();
        assert!(s.stack.is_empty());
    }

    #[test]
    fn test_type_word_underflow() {
        let mut s = state_with(vec![]);
        assert!(type_word(&mut s).is_err());
    }

    #[test]
    fn test_dot_s_preserves_stack() {
        let mut s = state_with(vec![Value::Int(1), Value::Str("x".into())]);
        dot_s(&mut s).unwrap();
        assert_eq!(s.stack.len(), 2); // unchanged
    }

    #[test]
    fn test_to_output_from_str() {
        let mut s = state_with(vec![Value::Str("data".into())]);
        to_output(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Output("data".into())]);
    }

    #[test]
    fn test_to_output_already_output() {
        let mut s = state_with(vec![Value::Output("data".into())]);
        to_output(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Output("data".into())]);
    }

    #[test]
    fn test_to_output_from_int_fails() {
        let mut s = state_with(vec![Value::Int(42)]);
        assert!(to_output(&mut s).is_err());
    }

    #[test]
    fn test_to_output_underflow() {
        let mut s = state_with(vec![]);
        assert!(to_output(&mut s).is_err());
    }

    #[test]
    fn test_to_string_from_output() {
        let mut s = state_with(vec![Value::Output("data".into())]);
        to_string_word(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("data".into())]);
    }

    #[test]
    fn test_to_string_from_int() {
        let mut s = state_with(vec![Value::Int(42)]);
        to_string_word(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("42".into())]);
    }

    #[test]
    fn test_to_string_already_str() {
        let mut s = state_with(vec![Value::Str("hi".into())]);
        to_string_word(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("hi".into())]);
    }

    #[test]
    fn test_to_string_underflow() {
        let mut s = state_with(vec![]);
        assert!(to_string_word(&mut s).is_err());
    }

    // ===== File I/O tests =====

    #[test]
    fn test_write_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("yafsh_test_write.txt");
        let path_str = path.to_string_lossy().to_string();

        let mut s = state_with(vec![
            Value::Output("hello file\n".into()),
            Value::Str(path_str.clone()),
        ]);
        write_file(&mut s).unwrap();
        assert!(s.stack.is_empty());

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "hello file\n");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_write_file_truncates() {
        let dir = std::env::temp_dir();
        let path = dir.join("yafsh_test_truncate.txt");
        let path_str = path.to_string_lossy().to_string();

        // Write initial content
        std::fs::write(&path, "old content").unwrap();

        let mut s = state_with(vec![
            Value::Output("new".into()),
            Value::Str(path_str.clone()),
        ]);
        write_file(&mut s).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "new");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_append_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("yafsh_test_append.txt");
        let path_str = path.to_string_lossy().to_string();

        std::fs::write(&path, "first\n").unwrap();

        let mut s = state_with(vec![
            Value::Output("second\n".into()),
            Value::Str(path_str.clone()),
        ]);
        append_file(&mut s).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "first\nsecond\n");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_write_file_underflow() {
        let mut s = state_with(vec![Value::Str("file.txt".into())]);
        assert!(write_file(&mut s).is_err());
    }

    #[test]
    fn test_write_file_wrong_types() {
        let mut s = state_with(vec![
            Value::Str("not output".into()),
            Value::Str("file.txt".into()),
        ]);
        assert!(write_file(&mut s).is_err());
        // Values should be restored
        assert_eq!(s.stack.len(), 2);
    }

    #[test]
    fn test_append_file_underflow() {
        let mut s = state_with(vec![Value::Str("file.txt".into())]);
        assert!(append_file(&mut s).is_err());
    }
}

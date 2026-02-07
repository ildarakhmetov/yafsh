use crate::types::{State, Value};

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

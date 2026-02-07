use crate::types::State;

/// `dup` ( a -- a a ) Duplicate top item.
pub fn dup(state: &mut State) -> Result<(), String> {
    let top = state.stack.last().ok_or("dup: stack underflow")?.clone();
    state.stack.push(top);
    Ok(())
}

/// `swap` ( a b -- b a ) Swap top two items.
pub fn swap(state: &mut State) -> Result<(), String> {
    let len = state.stack.len();
    if len < 2 {
        return Err("swap: stack underflow".into());
    }
    state.stack.swap(len - 1, len - 2);
    Ok(())
}

/// `drop` ( a -- ) Remove top item.
pub fn drop_word(state: &mut State) -> Result<(), String> {
    state.stack.pop().ok_or("drop: stack underflow")?;
    Ok(())
}

/// `clear` ( ... -- ) Clear entire stack.
pub fn clear(state: &mut State) -> Result<(), String> {
    state.stack.clear();
    Ok(())
}

/// `over` ( a b -- a b a ) Copy second item to top.
pub fn over(state: &mut State) -> Result<(), String> {
    let len = state.stack.len();
    if len < 2 {
        return Err("over: stack underflow".into());
    }
    let val = state.stack[len - 2].clone();
    state.stack.push(val);
    Ok(())
}

/// `rot` ( a b c -- b c a ) Rotate top three items.
pub fn rot(state: &mut State) -> Result<(), String> {
    let len = state.stack.len();
    if len < 3 {
        return Err("rot: stack underflow".into());
    }
    // Remove third-from-top and push to top
    let val = state.stack.remove(len - 3);
    state.stack.push(val);
    Ok(())
}

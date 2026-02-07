use crate::types::State;
#[cfg(test)]
use crate::types::Value;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with(vals: Vec<Value>) -> State {
        let mut s = State::new();
        s.stack = vals;
        s
    }

    #[test]
    fn test_dup() {
        let mut s = state_with(vec![Value::Int(5)]);
        dup(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(5), Value::Int(5)]);
    }

    #[test]
    fn test_dup_underflow() {
        let mut s = state_with(vec![]);
        assert!(dup(&mut s).is_err());
    }

    #[test]
    fn test_swap() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(2)]);
        swap(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(2), Value::Int(1)]);
    }

    #[test]
    fn test_swap_underflow() {
        let mut s = state_with(vec![Value::Int(1)]);
        assert!(swap(&mut s).is_err());
    }

    #[test]
    fn test_drop() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(2)]);
        drop_word(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_drop_underflow() {
        let mut s = state_with(vec![]);
        assert!(drop_word(&mut s).is_err());
    }

    #[test]
    fn test_clear() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        clear(&mut s).unwrap();
        assert!(s.stack.is_empty());
    }

    #[test]
    fn test_clear_empty() {
        let mut s = state_with(vec![]);
        clear(&mut s).unwrap();
        assert!(s.stack.is_empty());
    }

    #[test]
    fn test_over() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(2)]);
        over(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1), Value::Int(2), Value::Int(1)]);
    }

    #[test]
    fn test_over_underflow() {
        let mut s = state_with(vec![Value::Int(1)]);
        assert!(over(&mut s).is_err());
    }

    #[test]
    fn test_rot() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        rot(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(2), Value::Int(3), Value::Int(1)]);
    }

    #[test]
    fn test_rot_underflow() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(2)]);
        assert!(rot(&mut s).is_err());
    }

    #[test]
    fn test_dup_preserves_type() {
        let mut s = state_with(vec![Value::Str("hello".into())]);
        dup(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("hello".into()), Value::Str("hello".into())]);
    }

    #[test]
    fn test_swap_mixed_types() {
        let mut s = state_with(vec![Value::Str("a".into()), Value::Int(1)]);
        swap(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1), Value::Str("a".into())]);
    }
}

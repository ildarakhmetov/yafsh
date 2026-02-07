use crate::types::{LoopInfo, State, Value};

// ========== Helpers ==========

/// Pop two integers from the stack: top = b, second = a.
fn pop_two_ints(state: &mut State, op: &str) -> Result<(i64, i64), String> {
    if state.stack.len() < 2 {
        return Err(format!("{}: stack underflow", op));
    }
    let b = match state.stack.pop().unwrap() {
        Value::Int(n) => n,
        other => {
            state.stack.push(other);
            return Err(format!("{}: requires two integers", op));
        }
    };
    let a = match state.stack.pop().unwrap() {
        Value::Int(n) => n,
        other => {
            state.stack.push(other);
            state.stack.push(Value::Int(b));
            return Err(format!("{}: requires two integers", op));
        }
    };
    Ok((a, b))
}

// ========== Arithmetic ==========

/// `+` ( a b -- a+b ) Add two integers.
pub fn add(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, "+")?;
    state.stack.push(Value::Int(a + b));
    Ok(())
}

/// `-` ( a b -- a-b ) Subtract b from a.
pub fn sub(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, "-")?;
    state.stack.push(Value::Int(a - b));
    Ok(())
}

/// `*` ( a b -- a*b ) Multiply two integers.
pub fn mul(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, "*")?;
    state.stack.push(Value::Int(a * b));
    Ok(())
}

/// `/` ( a b -- a/b ) Divide a by b.
pub fn div(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, "/")?;
    if b == 0 {
        return Err("/: division by zero".into());
    }
    state.stack.push(Value::Int(a / b));
    Ok(())
}

/// `mod` ( a b -- a%b ) Remainder of a divided by b.
pub fn mod_op(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, "mod")?;
    if b == 0 {
        return Err("mod: division by zero".into());
    }
    state.stack.push(Value::Int(a % b));
    Ok(())
}

/// `/mod` ( a b -- quotient remainder ) Quotient and remainder.
pub fn divmod(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, "/mod")?;
    if b == 0 {
        return Err("/mod: division by zero".into());
    }
    state.stack.push(Value::Int(a / b));
    state.stack.push(Value::Int(a % b));
    Ok(())
}

/// `*/` ( a b c -- (a*b)/c ) Multiply then divide.
pub fn muldiv(state: &mut State) -> Result<(), String> {
    if state.stack.len() < 3 {
        return Err("*/: stack underflow".into());
    }
    let c = match state.stack.pop().unwrap() {
        Value::Int(n) => n,
        other => {
            state.stack.push(other);
            return Err("*/: requires three integers".into());
        }
    };
    let b = match state.stack.pop().unwrap() {
        Value::Int(n) => n,
        other => {
            state.stack.push(other);
            state.stack.push(Value::Int(c));
            return Err("*/: requires three integers".into());
        }
    };
    let a = match state.stack.pop().unwrap() {
        Value::Int(n) => n,
        other => {
            state.stack.push(other);
            state.stack.push(Value::Int(b));
            state.stack.push(Value::Int(c));
            return Err("*/: requires three integers".into());
        }
    };
    if c == 0 {
        return Err("*/: division by zero".into());
    }
    state.stack.push(Value::Int((a * b) / c));
    Ok(())
}

// ========== Comparisons ==========

/// `=` ( a b -- flag ) Test equality. Works on Int and Str.
pub fn eq(state: &mut State) -> Result<(), String> {
    if state.stack.len() < 2 {
        return Err("=: stack underflow".into());
    }
    let b = state.stack.pop().unwrap();
    let a = state.stack.pop().unwrap();
    let result = match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        _ => {
            state.stack.push(a);
            state.stack.push(b);
            return Err("=: requires two values of the same type".into());
        }
    };
    state.stack.push(Value::Int(if result { 1 } else { 0 }));
    Ok(())
}

/// `<>` ( a b -- flag ) Test inequality. Works on Int and Str.
pub fn neq(state: &mut State) -> Result<(), String> {
    if state.stack.len() < 2 {
        return Err("<>: stack underflow".into());
    }
    let b = state.stack.pop().unwrap();
    let a = state.stack.pop().unwrap();
    let result = match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => x != y,
        (Value::Str(x), Value::Str(y)) => x != y,
        _ => {
            state.stack.push(a);
            state.stack.push(b);
            return Err("<>: requires two values of the same type".into());
        }
    };
    state.stack.push(Value::Int(if result { 1 } else { 0 }));
    Ok(())
}

/// `>` ( a b -- flag ) Test greater than (integers only).
pub fn gt(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, ">")?;
    state.stack.push(Value::Int(if a > b { 1 } else { 0 }));
    Ok(())
}

/// `<` ( a b -- flag ) Test less than (integers only).
pub fn lt(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, "<")?;
    state.stack.push(Value::Int(if a < b { 1 } else { 0 }));
    Ok(())
}

/// `>=` ( a b -- flag ) Test greater than or equal (integers only).
pub fn gte(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, ">=")?;
    state.stack.push(Value::Int(if a >= b { 1 } else { 0 }));
    Ok(())
}

/// `<=` ( a b -- flag ) Test less than or equal (integers only).
pub fn lte(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, "<=")?;
    state.stack.push(Value::Int(if a <= b { 1 } else { 0 }));
    Ok(())
}

// ========== Boolean logic ==========

/// `and` ( a b -- flag ) Boolean AND (0=false, non-zero=true).
pub fn bool_and(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, "and")?;
    let result = if a != 0 && b != 0 { 1 } else { 0 };
    state.stack.push(Value::Int(result));
    Ok(())
}

/// `or` ( a b -- flag ) Boolean OR (0=false, non-zero=true).
pub fn bool_or(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, "or")?;
    let result = if a != 0 || b != 0 { 1 } else { 0 };
    state.stack.push(Value::Int(result));
    Ok(())
}

/// `not` ( a -- flag ) Boolean NOT (0=false, non-zero=true).
pub fn bool_not(state: &mut State) -> Result<(), String> {
    let val = state.stack.pop().ok_or("not: stack underflow")?;
    match val {
        Value::Int(a) => {
            state.stack.push(Value::Int(if a == 0 { 1 } else { 0 }));
            Ok(())
        }
        other => {
            state.stack.push(other);
            Err("not: requires integer".into())
        }
    }
}

/// `xor` ( a b -- flag ) Boolean XOR (0=false, non-zero=true).
pub fn bool_xor(state: &mut State) -> Result<(), String> {
    let (a, b) = pop_two_ints(state, "xor")?;
    let result = match (a != 0, b != 0) {
        (true, false) | (false, true) => 1,
        _ => 0,
    };
    state.stack.push(Value::Int(result));
    Ok(())
}

// ========== String operations ==========

/// `concat` ( a b -- a+b ) Concatenate two strings.
pub fn concat(state: &mut State) -> Result<(), String> {
    if state.stack.len() < 2 {
        return Err("concat: stack underflow".into());
    }
    let b = state.stack.pop().unwrap();
    let a = state.stack.pop().unwrap();
    match (a, b) {
        (Value::Str(sa), Value::Str(sb)) => {
            state.stack.push(Value::Str(sa + &sb));
            Ok(())
        }
        (a, b) => {
            state.stack.push(a);
            state.stack.push(b);
            Err("concat: requires two strings".into())
        }
    }
}

// ========== Loop index words ==========

/// `i` ( -- index ) Push current (innermost) loop index.
pub fn loop_i(state: &mut State) -> Result<(), String> {
    match state.loop_stack.last() {
        Some(LoopInfo::DoCountedLoop { current, .. })
        | Some(LoopInfo::DoPlusCountedLoop { current, .. }) => {
            state.stack.push(Value::Int(*current));
            Ok(())
        }
        Some(LoopInfo::BeginUntilLoop) | Some(LoopInfo::BeginWhileLoop) => {
            Err("i: loop index not available (not a counted loop)".into())
        }
        None => Err("i: not inside a loop".into()),
    }
}

/// `j` ( -- index ) Push outer loop index (for nested loops).
pub fn loop_j(state: &mut State) -> Result<(), String> {
    let len = state.loop_stack.len();
    if len < 2 {
        return Err("j: not inside a nested loop".into());
    }
    match &state.loop_stack[len - 2] {
        LoopInfo::DoCountedLoop { current, .. }
        | LoopInfo::DoPlusCountedLoop { current, .. } => {
            state.stack.push(Value::Int(*current));
            Ok(())
        }
        LoopInfo::BeginUntilLoop | LoopInfo::BeginWhileLoop => {
            Err("j: outer loop index not available (not a counted loop)".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    fn state_with(vals: Vec<Value>) -> State {
        let mut s = State::new();
        s.stack = vals;
        s
    }

    // ===== Arithmetic =====

    #[test]
    fn test_add() {
        let mut s = state_with(vec![Value::Int(3), Value::Int(7)]);
        add(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(10)]);
    }

    #[test]
    fn test_add_negative() {
        let mut s = state_with(vec![Value::Int(-3), Value::Int(7)]);
        add(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(4)]);
    }

    #[test]
    fn test_add_underflow() {
        let mut s = state_with(vec![Value::Int(1)]);
        assert!(add(&mut s).is_err());
    }

    #[test]
    fn test_sub() {
        let mut s = state_with(vec![Value::Int(10), Value::Int(3)]);
        sub(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(7)]);
    }

    #[test]
    fn test_mul() {
        let mut s = state_with(vec![Value::Int(6), Value::Int(7)]);
        mul(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(42)]);
    }

    #[test]
    fn test_div() {
        let mut s = state_with(vec![Value::Int(15), Value::Int(3)]);
        div(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(5)]);
    }

    #[test]
    fn test_div_by_zero() {
        let mut s = state_with(vec![Value::Int(10), Value::Int(0)]);
        assert!(div(&mut s).is_err());
    }

    #[test]
    fn test_mod_op() {
        let mut s = state_with(vec![Value::Int(10), Value::Int(3)]);
        mod_op(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_mod_by_zero() {
        let mut s = state_with(vec![Value::Int(10), Value::Int(0)]);
        assert!(mod_op(&mut s).is_err());
    }

    #[test]
    fn test_divmod() {
        let mut s = state_with(vec![Value::Int(10), Value::Int(3)]);
        divmod(&mut s).unwrap();
        // quotient=3, remainder=1; quotient pushed first, remainder on top
        assert_eq!(s.stack, vec![Value::Int(3), Value::Int(1)]);
    }

    #[test]
    fn test_divmod_by_zero() {
        let mut s = state_with(vec![Value::Int(10), Value::Int(0)]);
        assert!(divmod(&mut s).is_err());
    }

    #[test]
    fn test_muldiv() {
        // (2 * 6) / 4 = 3
        let mut s = state_with(vec![Value::Int(2), Value::Int(6), Value::Int(4)]);
        muldiv(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(3)]);
    }

    #[test]
    fn test_muldiv_by_zero() {
        let mut s = state_with(vec![Value::Int(2), Value::Int(6), Value::Int(0)]);
        assert!(muldiv(&mut s).is_err());
    }

    #[test]
    fn test_muldiv_underflow() {
        let mut s = state_with(vec![Value::Int(2), Value::Int(6)]);
        assert!(muldiv(&mut s).is_err());
    }

    // ===== Comparisons =====

    #[test]
    fn test_eq_true() {
        let mut s = state_with(vec![Value::Int(5), Value::Int(5)]);
        eq(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_eq_false() {
        let mut s = state_with(vec![Value::Int(5), Value::Int(7)]);
        eq(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_eq_strings() {
        let mut s = state_with(vec![Value::Str("hello".into()), Value::Str("hello".into())]);
        eq(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_eq_strings_not_equal() {
        let mut s = state_with(vec![Value::Str("hello".into()), Value::Str("world".into())]);
        eq(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_eq_mixed_types() {
        let mut s = state_with(vec![Value::Int(1), Value::Str("1".into())]);
        assert!(eq(&mut s).is_err());
    }

    #[test]
    fn test_eq_underflow() {
        let mut s = state_with(vec![Value::Int(1)]);
        assert!(eq(&mut s).is_err());
    }

    #[test]
    fn test_neq_true() {
        let mut s = state_with(vec![Value::Int(5), Value::Int(7)]);
        neq(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_neq_false() {
        let mut s = state_with(vec![Value::Int(5), Value::Int(5)]);
        neq(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_neq_strings() {
        let mut s = state_with(vec![Value::Str("hello".into()), Value::Str("world".into())]);
        neq(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_gt_true() {
        let mut s = state_with(vec![Value::Int(5), Value::Int(3)]);
        gt(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_gt_false() {
        let mut s = state_with(vec![Value::Int(3), Value::Int(5)]);
        gt(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_lt_true() {
        let mut s = state_with(vec![Value::Int(3), Value::Int(5)]);
        lt(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_lt_false() {
        let mut s = state_with(vec![Value::Int(5), Value::Int(3)]);
        lt(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_gte_equal() {
        let mut s = state_with(vec![Value::Int(5), Value::Int(5)]);
        gte(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_gte_greater() {
        let mut s = state_with(vec![Value::Int(7), Value::Int(5)]);
        gte(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_gte_less() {
        let mut s = state_with(vec![Value::Int(3), Value::Int(5)]);
        gte(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_lte_equal() {
        let mut s = state_with(vec![Value::Int(5), Value::Int(5)]);
        lte(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_lte_less() {
        let mut s = state_with(vec![Value::Int(3), Value::Int(7)]);
        lte(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_lte_greater() {
        let mut s = state_with(vec![Value::Int(7), Value::Int(3)]);
        lte(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    // ===== Boolean =====

    #[test]
    fn test_and_both_true() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(1)]);
        bool_and(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_and_one_false() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(0)]);
        bool_and(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_and_both_false() {
        let mut s = state_with(vec![Value::Int(0), Value::Int(0)]);
        bool_and(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_or_one_true() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(0)]);
        bool_or(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_or_both_false() {
        let mut s = state_with(vec![Value::Int(0), Value::Int(0)]);
        bool_or(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_not_false_to_true() {
        let mut s = state_with(vec![Value::Int(0)]);
        bool_not(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_not_true_to_false() {
        let mut s = state_with(vec![Value::Int(1)]);
        bool_not(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_not_nonzero_truthy() {
        let mut s = state_with(vec![Value::Int(42)]);
        bool_not(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_not_underflow() {
        let mut s = state_with(vec![]);
        assert!(bool_not(&mut s).is_err());
    }

    #[test]
    fn test_xor_different() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(0)]);
        bool_xor(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(1)]);
    }

    #[test]
    fn test_xor_same() {
        let mut s = state_with(vec![Value::Int(1), Value::Int(1)]);
        bool_xor(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    #[test]
    fn test_xor_both_false() {
        let mut s = state_with(vec![Value::Int(0), Value::Int(0)]);
        bool_xor(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Int(0)]);
    }

    // ===== String =====

    #[test]
    fn test_concat() {
        let mut s = state_with(vec![Value::Str("hello ".into()), Value::Str("world".into())]);
        concat(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("hello world".into())]);
    }

    #[test]
    fn test_concat_empty() {
        let mut s = state_with(vec![Value::Str("hello".into()), Value::Str("".into())]);
        concat(&mut s).unwrap();
        assert_eq!(s.stack, vec![Value::Str("hello".into())]);
    }

    #[test]
    fn test_concat_wrong_type() {
        let mut s = state_with(vec![Value::Str("hello".into()), Value::Int(42)]);
        assert!(concat(&mut s).is_err());
    }

    #[test]
    fn test_concat_underflow() {
        let mut s = state_with(vec![Value::Str("hello".into())]);
        assert!(concat(&mut s).is_err());
    }

    // ===== Type error tests =====

    #[test]
    fn test_add_wrong_type() {
        let mut s = state_with(vec![Value::Str("a".into()), Value::Int(1)]);
        assert!(add(&mut s).is_err());
    }

    #[test]
    fn test_gt_wrong_type() {
        let mut s = state_with(vec![Value::Str("a".into()), Value::Int(1)]);
        assert!(gt(&mut s).is_err());
    }

    #[test]
    fn test_and_wrong_type() {
        let mut s = state_with(vec![Value::Str("a".into()), Value::Int(1)]);
        assert!(bool_and(&mut s).is_err());
    }
}

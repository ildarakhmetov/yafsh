use yafsh::builtins;
use yafsh::eval;
use yafsh::types::{State, Value};

/// Create a fresh state with all builtins registered.
fn new_state() -> State {
    let mut s = State::new();
    builtins::register_builtins(&mut s);
    s
}

/// Helper: eval a line and return the resulting stack.
fn eval(line: &str) -> Vec<Value> {
    let mut s = new_state();
    eval::eval_line(&mut s, line).unwrap();
    s.stack
}

/// Helper: eval multiple lines sequentially.
fn eval_lines(lines: &[&str]) -> State {
    let mut s = new_state();
    for line in lines {
        eval::eval_line(&mut s, line).unwrap();
    }
    s
}

// ========== Basic value pushing ==========

#[test]
fn push_integer() {
    assert_eq!(eval("42"), vec![Value::Int(42)]);
}

#[test]
fn push_negative_integer() {
    assert_eq!(eval("-7"), vec![Value::Int(-7)]);
}

#[test]
fn push_quoted_string() {
    assert_eq!(eval("\"hello world\""), vec![Value::Str("hello world".into())]);
}

#[test]
fn push_unquoted_not_in_path() {
    // "xyznotacommand" won't be in PATH or dict, should be pushed as Str
    assert_eq!(eval("xyznotacommand"), vec![Value::Str("xyznotacommand".into())]);
}

#[test]
fn push_multiple_values() {
    assert_eq!(
        eval("1 2 3"),
        vec![Value::Int(1), Value::Int(2), Value::Int(3)]
    );
}

#[test]
fn push_mixed_types() {
    assert_eq!(
        eval("\"hello\" 42"),
        vec![Value::Str("hello".into()), Value::Int(42)]
    );
}

// ========== Stack operations via eval ==========

#[test]
fn eval_dup() {
    assert_eq!(eval("5 dup"), vec![Value::Int(5), Value::Int(5)]);
}

#[test]
fn eval_swap() {
    assert_eq!(eval("1 2 swap"), vec![Value::Int(2), Value::Int(1)]);
}

#[test]
fn eval_drop() {
    assert_eq!(eval("1 2 drop"), vec![Value::Int(1)]);
}

#[test]
fn eval_clear() {
    assert_eq!(eval("1 2 3 clear"), vec![]);
}

#[test]
fn eval_over() {
    assert_eq!(
        eval("1 2 over"),
        vec![Value::Int(1), Value::Int(2), Value::Int(1)]
    );
}

#[test]
fn eval_rot() {
    assert_eq!(
        eval("1 2 3 rot"),
        vec![Value::Int(2), Value::Int(3), Value::Int(1)]
    );
}

// ========== Shell execution ==========

#[test]
fn eval_echo() {
    let stack = eval("hello /bin/echo");
    assert_eq!(stack.len(), 1);
    match &stack[0] {
        Value::Output(s) => assert_eq!(s.trim(), "hello"),
        other => panic!("expected Output, got {:?}", other),
    }
}

#[test]
fn eval_echo_multiple_args() {
    let stack = eval("hello world /bin/echo");
    assert_eq!(stack.len(), 1);
    match &stack[0] {
        Value::Output(s) => assert_eq!(s.trim(), "hello world"),
        other => panic!("expected Output, got {:?}", other),
    }
}

#[test]
fn eval_path_lookup() {
    // "echo" should be found in PATH
    let stack = eval("hello echo");
    assert_eq!(stack.len(), 1);
    match &stack[0] {
        Value::Output(s) => assert_eq!(s.trim(), "hello"),
        other => panic!("expected Output, got {:?}", other),
    }
}

// ========== Auto-piping ==========

#[test]
fn eval_auto_pipe() {
    // echo produces Output, then wc -c counts its bytes via stdin
    let s = eval_lines(&["hello echo", "\"-c\" wc"]);
    assert_eq!(s.stack.len(), 1);
    match &s.stack[0] {
        Value::Output(out) => {
            let n: i64 = out.trim().parse().unwrap();
            assert_eq!(n, 6); // "hello\n" = 6 bytes
        }
        other => panic!("expected Output, got {:?}", other),
    }
}

// ========== Depth control ==========

#[test]
fn eval_depth_control() {
    // "extra" "hello" 1 echo -> echo only takes 1 arg, "extra" stays
    let stack = eval("extra hello 1 echo");
    assert_eq!(stack.len(), 2);
    assert_eq!(stack[0], Value::Str("extra".into()));
    match &stack[1] {
        Value::Output(s) => assert_eq!(s.trim(), "hello"),
        other => panic!("expected Output, got {:?}", other),
    }
}

// ========== Word definitions ==========

#[test]
fn eval_word_definition() {
    let s = eval_lines(&[": greet \"hello\" ;", "greet"]);
    assert_eq!(s.stack, vec![Value::Str("hello".into())]);
}

#[test]
fn eval_word_definition_with_builtins() {
    let s = eval_lines(&[": dup2 dup dup ;", "5 dup2"]);
    assert_eq!(
        s.stack,
        vec![Value::Int(5), Value::Int(5), Value::Int(5)]
    );
}

#[test]
fn eval_word_overwrites() {
    let s = eval_lines(&[": foo 1 ;", ": foo 2 ;", "foo"]);
    assert_eq!(s.stack, vec![Value::Int(2)]);
}

#[test]
fn eval_word_definition_multitoken() {
    // Define a word that pushes two values
    let s = eval_lines(&[": pair \"a\" \"b\" ;", "pair"]);
    assert_eq!(
        s.stack,
        vec![Value::Str("a".into()), Value::Str("b".into())]
    );
}

// ========== if/else/then ==========

#[test]
fn eval_if_true() {
    let s = eval_lines(&["1 if 42 then"]);
    assert_eq!(s.stack, vec![Value::Int(42)]);
}

#[test]
fn eval_if_false() {
    let s = eval_lines(&["0 if 42 then"]);
    assert!(s.stack.is_empty());
}

#[test]
fn eval_if_else_true() {
    let s = eval_lines(&["1 if \"yes\" else \"no\" then"]);
    assert_eq!(s.stack, vec![Value::Str("yes".into())]);
}

#[test]
fn eval_if_else_false() {
    let s = eval_lines(&["0 if \"yes\" else \"no\" then"]);
    assert_eq!(s.stack, vec![Value::Str("no".into())]);
}

#[test]
fn eval_nested_if() {
    // outer true, inner true
    let s = eval_lines(&["1 if 1 if 99 then then"]);
    assert_eq!(s.stack, vec![Value::Int(99)]);
}

#[test]
fn eval_nested_if_outer_false() {
    // outer false: should skip everything including inner if/then
    let s = eval_lines(&["0 if 1 if 99 then then"]);
    assert!(s.stack.is_empty());
}

#[test]
fn eval_nested_if_inner_false() {
    // outer true, inner false
    let s = eval_lines(&["1 if 0 if 99 then then"]);
    assert!(s.stack.is_empty());
}

#[test]
fn eval_if_underflow() {
    let mut s = new_state();
    assert!(eval::eval_line(&mut s, "if 1 then").is_err());
}

#[test]
fn eval_if_non_integer() {
    let mut s = new_state();
    s.stack.push(Value::Str("hello".into()));
    assert!(eval::eval_line(&mut s, "if 1 then").is_err());
}

// ========== Glob expansion ==========

#[test]
fn eval_glob_no_match() {
    // Pattern that definitely won't match: falls through as literal string
    let stack = eval("zzzzz_no_match_*");
    assert_eq!(stack, vec![Value::Str("zzzzz_no_match_*".into())]);
}

// ========== Type conversions via eval ==========

#[test]
fn eval_to_output() {
    let s = eval_lines(&["\"data\" >output"]);
    assert_eq!(s.stack, vec![Value::Output("data".into())]);
}

#[test]
fn eval_to_string() {
    let s = eval_lines(&["42 >string"]);
    assert_eq!(s.stack, vec![Value::Str("42".into())]);
}

// ========== Error propagation ==========

#[test]
fn eval_stack_underflow_propagates() {
    let mut s = new_state();
    assert!(eval::eval_line(&mut s, "drop").is_err());
}

// ========== Exit code tracking ==========

#[test]
fn eval_exit_code_success() {
    let s = eval_lines(&["/bin/true", "?"]);
    // Stack: Output("") from true, then Int(0) from ?
    assert_eq!(s.stack.len(), 2);
    assert_eq!(s.stack[1], Value::Int(0));
}

#[test]
fn eval_exit_code_failure() {
    let s = eval_lines(&["/bin/false", "?"]);
    assert_eq!(s.stack.len(), 2);
    assert_eq!(s.stack[1], Value::Int(1));
}

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

// ========== Arithmetic ==========

#[test]
fn eval_add() {
    assert_eq!(eval("3 7 +"), vec![Value::Int(10)]);
}

#[test]
fn eval_sub() {
    assert_eq!(eval("10 3 -"), vec![Value::Int(7)]);
}

#[test]
fn eval_mul() {
    assert_eq!(eval("6 7 *"), vec![Value::Int(42)]);
}

#[test]
fn eval_div() {
    assert_eq!(eval("15 3 /"), vec![Value::Int(5)]);
}

#[test]
fn eval_div_by_zero() {
    let mut s = new_state();
    eval::eval_line(&mut s, "10 0").unwrap();
    assert!(eval::eval_line(&mut s, "/").is_err());
}

#[test]
fn eval_mod() {
    assert_eq!(eval("10 3 mod"), vec![Value::Int(1)]);
}

#[test]
fn eval_mod_by_zero() {
    let mut s = new_state();
    eval::eval_line(&mut s, "10 0").unwrap();
    assert!(eval::eval_line(&mut s, "mod").is_err());
}

#[test]
fn eval_divmod() {
    // 10 /mod 3 -> quotient=3, remainder=1
    assert_eq!(eval("10 3 /mod"), vec![Value::Int(3), Value::Int(1)]);
}

#[test]
fn eval_muldiv() {
    // (2 * 6) / 4 = 3
    assert_eq!(eval("2 6 4 */"), vec![Value::Int(3)]);
}

#[test]
fn eval_arithmetic_chain() {
    // 2 3 + 4 * = (2+3)*4 = 20
    assert_eq!(eval("2 3 + 4 *"), vec![Value::Int(20)]);
}

#[test]
fn eval_negative_arithmetic() {
    assert_eq!(eval("-3 7 +"), vec![Value::Int(4)]);
}

// ========== Comparisons ==========

#[test]
fn eval_eq_true() {
    assert_eq!(eval("5 5 ="), vec![Value::Int(1)]);
}

#[test]
fn eval_eq_false() {
    assert_eq!(eval("5 7 ="), vec![Value::Int(0)]);
}

#[test]
fn eval_eq_strings() {
    let s = eval_lines(&["\"hello\" \"hello\" ="]);
    assert_eq!(s.stack, vec![Value::Int(1)]);
}

#[test]
fn eval_eq_strings_not_equal() {
    let s = eval_lines(&["\"hello\" \"world\" ="]);
    assert_eq!(s.stack, vec![Value::Int(0)]);
}

#[test]
fn eval_neq() {
    assert_eq!(eval("5 7 <>"), vec![Value::Int(1)]);
}

#[test]
fn eval_neq_equal() {
    assert_eq!(eval("5 5 <>"), vec![Value::Int(0)]);
}

#[test]
fn eval_gt_true() {
    assert_eq!(eval("5 3 >"), vec![Value::Int(1)]);
}

#[test]
fn eval_gt_false() {
    assert_eq!(eval("3 5 >"), vec![Value::Int(0)]);
}

#[test]
fn eval_lt_true() {
    assert_eq!(eval("3 5 <"), vec![Value::Int(1)]);
}

#[test]
fn eval_lt_false() {
    assert_eq!(eval("5 3 <"), vec![Value::Int(0)]);
}

#[test]
fn eval_gte_equal() {
    assert_eq!(eval("5 5 >="), vec![Value::Int(1)]);
}

#[test]
fn eval_gte_greater() {
    assert_eq!(eval("7 5 >="), vec![Value::Int(1)]);
}

#[test]
fn eval_gte_less() {
    assert_eq!(eval("3 5 >="), vec![Value::Int(0)]);
}

#[test]
fn eval_lte_equal() {
    assert_eq!(eval("5 5 <="), vec![Value::Int(1)]);
}

#[test]
fn eval_lte_less() {
    assert_eq!(eval("3 7 <="), vec![Value::Int(1)]);
}

#[test]
fn eval_lte_greater() {
    assert_eq!(eval("7 3 <="), vec![Value::Int(0)]);
}

// ========== Boolean logic ==========

#[test]
fn eval_and_both_true() {
    assert_eq!(eval("1 1 and"), vec![Value::Int(1)]);
}

#[test]
fn eval_and_one_false() {
    assert_eq!(eval("1 0 and"), vec![Value::Int(0)]);
}

#[test]
fn eval_or_one_true() {
    assert_eq!(eval("1 0 or"), vec![Value::Int(1)]);
}

#[test]
fn eval_or_both_false() {
    assert_eq!(eval("0 0 or"), vec![Value::Int(0)]);
}

#[test]
fn eval_not_false() {
    assert_eq!(eval("0 not"), vec![Value::Int(1)]);
}

#[test]
fn eval_not_true() {
    assert_eq!(eval("1 not"), vec![Value::Int(0)]);
}

#[test]
fn eval_xor_different() {
    assert_eq!(eval("1 0 xor"), vec![Value::Int(1)]);
}

#[test]
fn eval_xor_same() {
    assert_eq!(eval("1 1 xor"), vec![Value::Int(0)]);
}

#[test]
fn eval_boolean_with_comparison() {
    // 5 > 3 and 10 > 7  =>  1 and 1  =>  1
    assert_eq!(eval("5 3 > 10 7 > and"), vec![Value::Int(1)]);
}

// ========== String operations ==========

#[test]
fn eval_concat() {
    let s = eval_lines(&["\"hello \" \"world\" concat"]);
    assert_eq!(s.stack, vec![Value::Str("hello world".into())]);
}

#[test]
fn eval_concat_empty() {
    let s = eval_lines(&["\"hello\" \"\" concat"]);
    assert_eq!(s.stack, vec![Value::Str("hello".into())]);
}

// ========== Environment builtins ==========

#[test]
fn eval_getenv() {
    let s = eval_lines(&["\"HOME\" getenv"]);
    assert_eq!(s.stack.len(), 1);
    match &s.stack[0] {
        Value::Str(v) => assert!(!v.is_empty()),
        other => panic!("expected Str, got {:?}", other),
    }
}

#[test]
fn eval_setenv_getenv_round_trip() {
    let s = eval_lines(&[
        "\"round_trip_value\" \"YAFSH_TEST_RT\" setenv",
        "\"YAFSH_TEST_RT\" getenv",
    ]);
    assert_eq!(s.stack, vec![Value::Str("round_trip_value".into())]);
    std::env::remove_var("YAFSH_TEST_RT");
}

#[test]
fn eval_unsetenv() {
    std::env::set_var("YAFSH_TEST_UNSET_EVAL", "temp");
    let s = eval_lines(&[
        "\"YAFSH_TEST_UNSET_EVAL\" unsetenv",
        "\"YAFSH_TEST_UNSET_EVAL\" getenv",
    ]);
    assert_eq!(s.stack, vec![Value::Str("".into())]);
}

// ========== File I/O ==========

#[test]
fn eval_write_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("yafsh_eval_write_test.txt");
    let path_str = path.to_string_lossy().to_string();

    let s = eval_lines(&[
        "\"test content\" >output",
        &format!("\"{}\" >file", path_str),
    ]);
    assert!(s.stack.is_empty());

    let contents = std::fs::read_to_string(&path).unwrap();
    assert_eq!(contents, "test content");
    std::fs::remove_file(&path).ok();
}

#[test]
fn eval_append_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("yafsh_eval_append_test.txt");
    let path_str = path.to_string_lossy().to_string();

    std::fs::write(&path, "line1\n").unwrap();

    let s = eval_lines(&[
        "\"line2\\n\" >output",
        &format!("\"{}\" >>file", path_str),
    ]);
    assert!(s.stack.is_empty());

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.starts_with("line1\n"));
    std::fs::remove_file(&path).ok();
}

// ========== pushd/popd ==========

#[test]
fn eval_pushd_popd() {
    let original = std::env::current_dir().unwrap();

    let s = eval_lines(&["\"/tmp\" pushd", "popd"]);
    assert!(s.stack.is_empty());
    assert_eq!(std::env::current_dir().unwrap(), original);
}

// ========== Introspection ==========

#[test]
fn eval_words() {
    // Just verify it runs without error
    let s = eval_lines(&["words"]);
    assert!(s.stack.is_empty());
}

#[test]
fn eval_help() {
    let s = eval_lines(&["help"]);
    assert!(s.stack.is_empty());
}

#[test]
fn eval_see_builtin() {
    let s = eval_lines(&["\"dup\" see"]);
    assert!(s.stack.is_empty());
}

#[test]
fn eval_see_defined_word() {
    let s = eval_lines(&[": greet \"hello\" ;", "\"greet\" see"]);
    assert!(s.stack.is_empty());
}

#[test]
fn eval_see_undefined() {
    let s = eval_lines(&["\"nonexistent\" see"]);
    assert!(s.stack.is_empty());
}

// ========== Combined: arithmetic with if/else ==========

#[test]
fn eval_comparison_with_if() {
    // if 5 > 3 then push "big" else push "small"
    let s = eval_lines(&["5 3 > if \"big\" else \"small\" then"]);
    assert_eq!(s.stack, vec![Value::Str("big".into())]);
}

#[test]
fn eval_comparison_with_if_false() {
    let s = eval_lines(&["3 5 > if \"big\" else \"small\" then"]);
    assert_eq!(s.stack, vec![Value::Str("small".into())]);
}

// ========== Word definition with arithmetic ==========

#[test]
fn eval_word_with_arithmetic() {
    // Define a word that squares a number
    let s = eval_lines(&[": square dup * ;", "5 square"]);
    assert_eq!(s.stack, vec![Value::Int(25)]);
}

#[test]
fn eval_word_with_comparison() {
    // Define a word that checks if a number is positive
    let s = eval_lines(&[": positive? 0 > ;", "5 positive?"]);
    assert_eq!(s.stack, vec![Value::Int(1)]);
}

// ========== begin...until loops ==========

#[test]
fn eval_begin_until_count_to_5() {
    // Count from 0 to 5
    let s = eval_lines(&["0 begin 1 + dup 5 = until"]);
    assert_eq!(s.stack, vec![Value::Int(5)]);
}

#[test]
fn eval_begin_until_executes_at_least_once() {
    // Even with a true condition, the body executes once
    let s = eval_lines(&["10 begin dup 1 + dup 5 > until"]);
    // 10 dup -> 10 10, 1 + -> 10 11, dup -> 10 11 11, 5 > -> 10 11 1 -> pops 1 (true), exits
    assert_eq!(s.stack, vec![Value::Int(10), Value::Int(11)]);
}

#[test]
fn eval_begin_until_doubling() {
    // Double until > 16
    let s = eval_lines(&["1 begin 2 * dup 16 > until"]);
    assert_eq!(s.stack, vec![Value::Int(32)]);
}

#[test]
fn eval_begin_until_with_stack_ops() {
    // Count to 3 and collect values
    let s = eval_lines(&["0 begin 1 + dup 3 = until"]);
    assert_eq!(s.stack, vec![Value::Int(3)]);
}

// ========== begin...while...repeat loops ==========

#[test]
fn eval_begin_while_repeat_countdown() {
    // Count down from 5 to 0, collecting values
    let s = eval_lines(&["5 begin dup 0 > while 1 - repeat"]);
    assert_eq!(s.stack, vec![Value::Int(0)]);
}

#[test]
fn eval_begin_while_repeat_false_never_enters() {
    // Condition is false initially: body never executes
    let s = eval_lines(&["0 begin dup 0 > while 1 - repeat"]);
    assert_eq!(s.stack, vec![Value::Int(0)]);
}

#[test]
fn eval_begin_while_repeat_sum() {
    // Sum from 1 to 5: 0 sum, 5 counter
    let s = eval_lines(&["0 5 begin dup 0 > while swap over + swap 1 - repeat drop"]);
    assert_eq!(s.stack, vec![Value::Int(15)]);
}

#[test]
fn eval_begin_while_repeat_doubling() {
    // Double until >= 100
    let s = eval_lines(&["1 begin dup 100 < while 2 * repeat"]);
    assert_eq!(s.stack, vec![Value::Int(128)]);
}

// ========== do...loop counted loops ==========

#[test]
fn eval_do_loop_simple() {
    // Sum 0+1+2+3+4 using loop index
    let s = eval_lines(&["0 0 5 do i + loop"]);
    assert_eq!(s.stack, vec![Value::Int(10)]);
}

#[test]
fn eval_do_loop_with_start_and_end() {
    // Sum 3+4+5+6
    let s = eval_lines(&["0 3 7 do i + loop"]);
    assert_eq!(s.stack, vec![Value::Int(18)]);
}

#[test]
fn eval_do_loop_empty_range() {
    // Empty range: start == limit, body never executes
    let s = eval_lines(&["42 5 5 do i + loop"]);
    assert_eq!(s.stack, vec![Value::Int(42)]);
}

#[test]
fn eval_do_loop_accumulate() {
    // Sum 1+2+3+4+5
    let s = eval_lines(&["0 1 6 do i + loop"]);
    assert_eq!(s.stack, vec![Value::Int(15)]);
}

#[test]
fn eval_do_loop_nested_with_ij() {
    // Nested loops: collect j*10 + i for 2x2 grid
    let s = eval_lines(&["0 2 do 0 2 do j 10 * i + loop loop"]);
    // Iterations: (j=0,i=0)->0, (j=0,i=1)->1, (j=1,i=0)->10, (j=1,i=1)->11
    // Stack accumulates: 0 1 10 11
    assert_eq!(
        s.stack,
        vec![Value::Int(0), Value::Int(1), Value::Int(10), Value::Int(11)]
    );
}

// ========== do...+loop counted loops ==========

#[test]
fn eval_do_plus_loop_step_by_2() {
    // Collect even numbers 0,2,4,6,8
    let s = eval_lines(&["0 10 do i 2 +loop"]);
    assert_eq!(
        s.stack,
        vec![
            Value::Int(0),
            Value::Int(2),
            Value::Int(4),
            Value::Int(6),
            Value::Int(8)
        ]
    );
}

#[test]
fn eval_do_plus_loop_negative_step() {
    // Count down: 10,8,6,4,2
    let s = eval_lines(&["10 0 do i -2 +loop"]);
    assert_eq!(
        s.stack,
        vec![
            Value::Int(10),
            Value::Int(8),
            Value::Int(6),
            Value::Int(4),
            Value::Int(2)
        ]
    );
}

#[test]
fn eval_do_plus_loop_step_by_3() {
    // 0,3,6,9
    let s = eval_lines(&["0 10 do i 3 +loop"]);
    assert_eq!(
        s.stack,
        vec![Value::Int(0), Value::Int(3), Value::Int(6), Value::Int(9)]
    );
}

// ========== each...then ==========

#[test]
fn eval_each_iterates_lines() {
    // Create multi-line output and iterate
    let mut s = new_state();
    s.stack.push(Value::Output("one\ntwo\nthree".into()));
    eval::eval_line(&mut s, "each then").unwrap();
    // each pushes each line as Str, body is empty so they accumulate
    assert_eq!(
        s.stack,
        vec![
            Value::Str("one".into()),
            Value::Str("two".into()),
            Value::Str("three".into())
        ]
    );
}

#[test]
fn eval_each_with_body() {
    // Iterate and apply operations
    let mut s = new_state();
    s.stack.push(Value::Output("hello\nworld".into()));
    eval::eval_line(&mut s, "each \"!\" concat then").unwrap();
    assert_eq!(
        s.stack,
        vec![
            Value::Str("hello!".into()),
            Value::Str("world!".into())
        ]
    );
}

#[test]
fn eval_each_empty_output() {
    // Empty output: body never executes
    let mut s = new_state();
    s.stack.push(Value::Output("".into()));
    eval::eval_line(&mut s, "each . then").unwrap();
    assert!(s.stack.is_empty());
}

// ========== Loop error handling ==========

#[test]
fn eval_until_without_begin() {
    let mut s = new_state();
    let result = eval::eval_line(&mut s, "until");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("no matching begin"));
}

#[test]
fn eval_repeat_without_begin() {
    let mut s = new_state();
    let result = eval::eval_line(&mut s, "repeat");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("no matching begin"));
}

#[test]
fn eval_loop_without_do() {
    let mut s = new_state();
    let result = eval::eval_line(&mut s, "loop");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("no matching do"));
}

#[test]
fn eval_plus_loop_without_do() {
    let mut s = new_state();
    let result = eval::eval_line(&mut s, "+loop");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("no matching do"));
}

#[test]
fn eval_until_stack_underflow() {
    let mut s = new_state();
    let result = eval::eval_line(&mut s, "begin until");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("stack underflow"));
}

#[test]
fn eval_while_stack_underflow() {
    let mut s = new_state();
    let result = eval::eval_line(&mut s, "begin while repeat");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("stack underflow"));
}

#[test]
fn eval_do_stack_underflow() {
    let mut s = new_state();
    s.stack.push(Value::Int(5));
    let result = eval::eval_line(&mut s, "do loop");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("stack underflow"));
}

#[test]
fn eval_i_outside_loop() {
    let mut s = new_state();
    let result = eval::eval_line(&mut s, "i");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not inside a loop"));
}

#[test]
fn eval_j_outside_nested_loop() {
    let mut s = new_state();
    let result = eval::eval_line(&mut s, "0 2 do j loop");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not inside a nested loop"));
}

#[test]
fn eval_each_stack_underflow() {
    let mut s = new_state();
    let result = eval::eval_line(&mut s, "each then");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("stack underflow"));
}

#[test]
fn eval_each_wrong_type() {
    let mut s = new_state();
    s.stack.push(Value::Str("not output".into()));
    let result = eval::eval_line(&mut s, "each then");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("requires Output"));
}

// ========== Conditional string helpers ==========

#[test]
fn eval_cond_prefix_nonempty() {
    let s = eval_lines(&["\"main\" \"@\" ?prefix"]);
    assert_eq!(s.stack, vec![Value::Str("@main".into())]);
}

#[test]
fn eval_cond_prefix_empty() {
    let s = eval_lines(&["\"\" \"@\" ?prefix"]);
    assert_eq!(s.stack, vec![Value::Str("".into())]);
}

#[test]
fn eval_cond_suffix_nonempty() {
    let s = eval_lines(&["\"main\" \"!\" ?suffix"]);
    assert_eq!(s.stack, vec![Value::Str("main!".into())]);
}

#[test]
fn eval_cond_suffix_empty() {
    let s = eval_lines(&["\"\" \"!\" ?suffix"]);
    assert_eq!(s.stack, vec![Value::Str("".into())]);
}

#[test]
fn eval_cond_wrap_nonempty() {
    let s = eval_lines(&["\"hello\" \"[\" \"]\" ?wrap"]);
    assert_eq!(s.stack, vec![Value::Str("[hello]".into())]);
}

#[test]
fn eval_cond_wrap_empty() {
    let s = eval_lines(&["\"\" \"[\" \"]\" ?wrap"]);
    assert_eq!(s.stack, vec![Value::Str("".into())]);
}

#[test]
fn eval_cond_prefix_in_word_definition() {
    // Simulate prompt building: "$gitbranch" "@" ?prefix
    let s = eval_lines(&[
        ": branch-prefix \"@\" ?prefix ;",
        "\"main\" branch-prefix",
    ]);
    assert_eq!(s.stack, vec![Value::Str("@main".into())]);
}

#[test]
fn eval_cond_prefix_chained() {
    // Chain ?prefix and ?suffix
    let s = eval_lines(&["\"dev\" \"(\" ?prefix \")\" ?suffix"]);
    assert_eq!(s.stack, vec![Value::Str("(dev)".into())]);
}

// ========== Prompt helper builtins ==========

#[test]
fn eval_dollar_stack_empty() {
    let s = eval_lines(&["$stack"]);
    assert_eq!(s.stack, vec![Value::Str("".into())]);
}

#[test]
fn eval_dollar_stack_with_items() {
    let s = eval_lines(&["1 2 3 $stack"]);
    assert_eq!(s.stack.len(), 4);
    assert_eq!(s.stack[3], Value::Str("[3]".into()));
}

#[test]
fn eval_dollar_in() {
    let s = eval_lines(&["1 2 $in"]);
    assert_eq!(s.stack.len(), 3);
    assert_eq!(s.stack[2], Value::Int(2));
}

#[test]
fn eval_dollar_out() {
    let s = eval_lines(&["\"data\" >output $out"]);
    assert_eq!(s.stack.len(), 2);
    assert_eq!(s.stack[1], Value::Int(1));
}

#[test]
fn eval_dollar_cwd() {
    let s = eval_lines(&["$cwd"]);
    assert_eq!(s.stack.len(), 1);
    match &s.stack[0] {
        Value::Str(v) => assert!(!v.is_empty()),
        other => panic!("expected Str, got {:?}", other),
    }
}

#[test]
fn eval_dollar_basename() {
    let s = eval_lines(&["$basename"]);
    assert_eq!(s.stack.len(), 1);
    match &s.stack[0] {
        Value::Str(v) => assert!(!v.is_empty()),
        other => panic!("expected Str, got {:?}", other),
    }
}

#[test]
fn eval_dollar_username() {
    let s = eval_lines(&["$username"]);
    assert_eq!(s.stack.len(), 1);
    match &s.stack[0] {
        Value::Str(v) => assert!(!v.is_empty()),
        other => panic!("expected Str, got {:?}", other),
    }
}

#[test]
fn eval_dollar_exitcode() {
    let s = eval_lines(&["/bin/false", "$exitcode"]);
    // Stack: Output from /bin/false, then "1"
    assert_eq!(s.stack.len(), 2);
    assert_eq!(s.stack[1], Value::Str("1".into()));
}

#[test]
fn eval_dollar_time() {
    let s = eval_lines(&["$time"]);
    assert_eq!(s.stack.len(), 1);
    match &s.stack[0] {
        Value::Str(v) => assert!(v.contains(':'), "time should contain colon: {}", v),
        other => panic!("expected Str, got {:?}", other),
    }
}

#[test]
fn eval_dollar_gitbranch() {
    // Just verify it runs without error
    let s = eval_lines(&["$gitbranch"]);
    assert_eq!(s.stack.len(), 1);
}

#[test]
fn eval_dollar_hostname() {
    let s = eval_lines(&["$hostname"]);
    assert_eq!(s.stack.len(), 1);
    match &s.stack[0] {
        Value::Str(v) => assert!(!v.is_empty()),
        other => panic!("expected Str, got {:?}", other),
    }
}

// ========== Loops in word definitions ==========

#[test]
fn eval_word_with_begin_while_loop() {
    // Define a word with a loop
    let s = eval_lines(&[
        ": countdown begin dup 0 > while 1 - repeat ;",
        "5 countdown",
    ]);
    assert_eq!(s.stack, vec![Value::Int(0)]);
}

#[test]
fn eval_word_with_do_loop() {
    // Define a word with a do loop
    let s = eval_lines(&[": sum-to 0 swap 1 swap do i + loop ;", "5 sum-to"]);
    // sum-to: 0 swap -> n 0, 1 swap -> 0 1 n, do i + loop -> sum 1..n-1
    // Wait, 5 sum-to: stack starts with 5
    // : sum-to 0 swap 1 swap do i + loop ;
    // 5 sum-to -> 0 swap(5,0)->5,0 1 swap(0,1)->5,1,0... hmm that's wrong
    // Let me think again: 5 sum-to
    // body: 0 swap 1 swap do i + loop
    // stack: [5]
    // 0 -> [5, 0]
    // swap -> [0, 5]
    // 1 -> [0, 5, 1]
    // swap -> [0, 1, 5]
    // do -> start=1, limit=5, body = [i, +]
    // i=1: 0 + 1 = 1
    // i=2: 1 + 2 = 3
    // i=3: 3 + 3 = 6
    // i=4: 6 + 4 = 10
    assert_eq!(s.stack, vec![Value::Int(10)]);
}

#[test]
fn eval_word_with_begin_until_loop() {
    // Define a word with begin...until
    let s = eval_lines(&[
        ": triple-until begin 3 * dup 100 > until ;",
        "1 triple-until",
    ]);
    // 1 -> 3 -> 9 -> 27 -> 81 -> 243 (>100, stop)
    assert_eq!(s.stack, vec![Value::Int(243)]);
}

#[test]
fn eval_nested_begin_while_loops() {
    // Nested begin...while...repeat is supported through depth tracking
    let s = eval_lines(&[
        "0 3 begin dup 0 > while swap 2 begin dup 0 > while 1 - swap 1 + swap repeat drop swap 1 - repeat drop",
    ]);
    // Outer loop runs 3 times, inner loop adds 2 each time: 0 + 2 + 2 + 2 = 6
    assert_eq!(s.stack, vec![Value::Int(6)]);
}

#[test]
fn eval_loop_with_if_inside() {
    // Use if/else/then inside a loop
    let s = eval_lines(&["0 0 10 do i 2 mod 0 = if i + then loop"]);
    // Sum of even numbers 0..9: 0+2+4+6+8 = 20
    assert_eq!(s.stack, vec![Value::Int(20)]);
}

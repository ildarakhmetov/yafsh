use crate::eval;
use crate::types::{LoopInfo, LoopType, State, Value};

// ========== Loop body splitting ==========

/// Split tokens at the `while` keyword, returning (before_while, after_while).
fn split_while_body(tokens: &[String]) -> Result<(Vec<String>, Vec<String>), String> {
    if let Some(pos) = tokens.iter().position(|t| t == "while") {
        Ok((tokens[..pos].to_vec(), tokens[pos + 1..].to_vec()))
    } else {
        Err("repeat: no matching while".into())
    }
}

// ========== Loop executors ==========

/// Execute a `begin ... until` loop.
///
/// Runs the body, then pops a condition from the stack.
/// If condition is `Int(0)` (false), loops again.
/// If condition is non-zero, exits.
/// Executes at least once (condition checked at end).
pub fn execute_begin_until(state: &mut State, body: &[String]) -> Result<(), String> {
    loop {
        // Push loop info for nesting tracking
        state.loop_stack.push(LoopInfo::BeginUntilLoop);

        // Execute body
        for token in body {
            eval::eval_token(state, token, false)?;
        }

        state.loop_stack.pop();

        // Check condition
        match state.stack.pop() {
            Some(Value::Int(0)) => {
                // Condition false, continue looping
            }
            Some(Value::Int(_)) => {
                // Condition true, exit loop
                return Ok(());
            }
            Some(_) => return Err("until: requires integer condition".into()),
            None => return Err("until: stack underflow (needs condition)".into()),
        }
    }
}

/// Execute a `begin ... while ... repeat` loop.
///
/// Runs `before_while`, pops condition.
/// If condition is non-zero (true), runs `after_while` and repeats.
/// If condition is zero (false), exits.
/// May not execute body if condition is initially false.
pub fn execute_begin_while(
    state: &mut State,
    before_while: &[String],
    after_while: &[String],
) -> Result<(), String> {
    loop {
        state.loop_stack.push(LoopInfo::BeginWhileLoop);

        // Execute before_while (condition computation)
        for token in before_while {
            eval::eval_token(state, token, false)?;
        }

        // Check condition
        match state.stack.pop() {
            Some(Value::Int(0)) => {
                // Condition false, exit loop
                state.loop_stack.pop();
                return Ok(());
            }
            Some(Value::Int(_)) => {
                // Condition true, execute body and repeat
            }
            Some(_) => {
                state.loop_stack.pop();
                return Err("while: requires integer condition".into());
            }
            None => {
                state.loop_stack.pop();
                return Err("while: stack underflow (needs condition)".into());
            }
        }

        // Execute after_while (loop body)
        for token in after_while {
            eval::eval_token(state, token, false)?;
        }

        state.loop_stack.pop();
    }
}

/// Execute a `do ... loop` counted loop.
///
/// Loops from `start` to `limit - 1` with step 1.
/// The loop index is accessible via `i`.
pub fn execute_do_loop(
    state: &mut State,
    start: i64,
    limit: i64,
    body: &[String],
) -> Result<(), String> {
    let mut idx = start;
    while idx < limit {
        let loop_info = LoopInfo::DoCountedLoop {
            start,
            limit,
            current: idx,
        };
        state.loop_stack.push(loop_info);

        for token in body {
            eval::eval_token(state, token, false)?;
        }

        state.loop_stack.pop();
        idx += 1;
    }
    Ok(())
}

/// Execute a `do ... +loop` counted loop with dynamic step.
///
/// Like `do_loop` but pops step from stack after each body execution.
/// Supports ascending (start < limit) and descending (start > limit) loops.
pub fn execute_do_plus_loop(
    state: &mut State,
    start: i64,
    limit: i64,
    body: &[String],
) -> Result<(), String> {
    let mut idx = start;
    loop {
        // Check if we should continue
        let should_continue = if start < limit {
            idx < limit
        } else {
            idx > limit
        };

        if !should_continue {
            return Ok(());
        }

        let loop_info = LoopInfo::DoPlusCountedLoop {
            start,
            limit,
            current: idx,
        };
        state.loop_stack.push(loop_info);

        for token in body {
            eval::eval_token(state, token, false)?;
        }

        state.loop_stack.pop();

        // Get step from stack
        match state.stack.pop() {
            Some(Value::Int(step)) => {
                idx += step;
            }
            Some(_) => return Err("+loop: requires integer step".into()),
            None => return Err("+loop: stack underflow (needs step)".into()),
        }
    }
}

// ========== Loop body collection ==========

/// Handle loop body collection and dispatch.
///
/// Called for each token while `collecting_loop` is active.
/// Tracks nesting depth for inner begin/do pairs and dispatches
/// to the appropriate executor when the terminating keyword is found.
pub fn handle_loop_collection(state: &mut State, token: &str) -> Result<(), String> {
    let (loop_type, mut body, depth) = state.collecting_loop.take().unwrap();

    match (token, &loop_type, depth) {
        // ---- begin...until ----
        ("until", LoopType::BeginUntil, 0) => {
            // End of begin...until loop (not nested)
            execute_begin_until(state, &body)?;
        }
        ("until", LoopType::BeginUntil, d) => {
            // Nested until, add to body and decrement depth
            body.push(token.to_string());
            state.collecting_loop = Some((loop_type, body, d - 1));
        }

        // ---- begin...while transition ----
        ("while", LoopType::BeginUntil, 0) => {
            // This is actually begin...while...repeat, switch type
            body.push("while".to_string());
            state.collecting_loop = Some((LoopType::BeginWhile, body, 0));
        }
        ("while", LoopType::BeginWhile, _) => {
            // Inside while mode, just add token
            body.push("while".to_string());
            state.collecting_loop = Some((loop_type, body, depth));
        }

        // ---- begin...while...repeat ----
        ("repeat", LoopType::BeginWhile, 0) => {
            // End of begin...while...repeat (not nested)
            let (before_while, after_while) = split_while_body(&body)?;
            execute_begin_while(state, &before_while, &after_while)?;
        }
        ("repeat", LoopType::BeginWhile, d) => {
            // Nested repeat, add to body and decrement depth
            body.push(token.to_string());
            state.collecting_loop = Some((loop_type, body, d - 1));
        }

        // ---- do...loop ----
        ("loop", LoopType::DoLoop | LoopType::DoPlusLoop, 0) => {
            // End of do...loop (not nested)
            match (state.stack.pop(), state.stack.pop()) {
                (Some(Value::Int(limit)), Some(Value::Int(start))) => {
                    execute_do_loop(state, start, limit, &body)?;
                }
                _ => return Err("do: stack underflow (needs start and limit)".into()),
            }
        }
        ("loop", LoopType::DoLoop | LoopType::DoPlusLoop, d) => {
            // Nested loop, add to body and decrement depth
            body.push(token.to_string());
            state.collecting_loop = Some((loop_type, body, d - 1));
        }

        // ---- do...+loop ----
        ("+loop", LoopType::DoPlusLoop | LoopType::DoLoop, 0) => {
            // End of do...+loop (not nested)
            match (state.stack.pop(), state.stack.pop()) {
                (Some(Value::Int(limit)), Some(Value::Int(start))) => {
                    execute_do_plus_loop(state, start, limit, &body)?;
                }
                _ => return Err("do: stack underflow (needs start and limit)".into()),
            }
        }
        ("+loop", LoopType::DoPlusLoop | LoopType::DoLoop, d) => {
            // Nested +loop, add to body and decrement depth
            body.push(token.to_string());
            state.collecting_loop = Some((loop_type, body, d - 1));
        }

        // ---- Nesting: begin/do increase depth ----
        ("begin", _, _) => {
            body.push(token.to_string());
            state.collecting_loop = Some((loop_type, body, depth + 1));
        }
        ("do", _, _) => {
            body.push(token.to_string());
            state.collecting_loop = Some((loop_type, body, depth + 1));
        }

        // ---- Regular token ----
        (_, _, _) => {
            body.push(token.to_string());
            state.collecting_loop = Some((loop_type, body, depth));
        }
    }

    Ok(())
}

/// Handle `each ... then` body collection.
///
/// Collects tokens until `then`, then executes the body for each line
/// of the output content.
pub fn handle_each_collection(state: &mut State, token: &str) -> Result<(), String> {
    let (output_content, mut body) = state.collecting_each.take().unwrap();

    if token == "then" {
        // End of each...then - execute body for each line
        let lines: Vec<String> = output_content.lines().map(|l| l.to_string()).collect();
        for line in &lines {
            // Push line onto stack as Str
            state.stack.push(Value::Str(line.clone()));
            // Execute body tokens
            for t in &body {
                eval::eval_token(state, t, false)?;
            }
        }
        Ok(())
    } else {
        // Accumulate token into body
        body.push(token.to_string());
        state.collecting_each = Some((output_content, body));
        Ok(())
    }
}

mod builtins;
mod eval;
mod tokenizer;
mod types;

use std::io::{self, Write};
use types::{State, Value};

/// Count inputs (Str/Int) vs outputs (Output) on the stack.
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

/// Build the prompt string based on stack state.
fn build_prompt(state: &State) -> String {
    let (inputs, outputs) = count_stack(&state.stack);
    let total = inputs + outputs;

    if total == 0 {
        "yafsh> ".to_string()
    } else if outputs == 0 {
        format!("yafsh[{}]> ", inputs)
    } else if inputs == 0 {
        format!("yafsh[:{}]> ", outputs)
    } else {
        format!("yafsh[{}:{}]> ", inputs, outputs)
    }
}

/// Auto-type: if top of stack is Output, print it (but keep it on stack).
fn auto_type_output(state: &State) {
    if let Some(Value::Output(s)) = state.stack.last() {
        print!("{}", s);
    }
}

fn main() {
    let mut state = State::new();
    builtins::register_builtins(&mut state);

    println!("yafsh 0.1.0");
    println!("Type 'exit' to quit, Ctrl-D for EOF");
    println!();

    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        // Print prompt
        let prompt = build_prompt(&state);
        print!("{}", prompt);
        io::stdout().flush().ok();

        // Read line
        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                // EOF
                println!("\nGoodbye!");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "exit" || trimmed == "quit" {
                    println!("Goodbye!");
                    break;
                }

                // Evaluate
                match eval::eval_line(&mut state, trimmed) {
                    Ok(()) => {
                        auto_type_output(&state);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}

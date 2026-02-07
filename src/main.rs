use std::io::{self, IsTerminal, Write};

use rustyline::error::ReadlineError;
use rustyline::Editor;

use yafsh::builtins;
use yafsh::config;
use yafsh::eval;
use yafsh::highlight::YafshHelper;
use yafsh::types::{State, Value};

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

/// Build the default prompt string based on stack state.
fn build_default_prompt(state: &State) -> String {
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

/// Evaluate the custom `$prompt` word and return the prompt string.
///
/// Saves the current stack, clears it, evaluates `$prompt`, collects the
/// resulting stack items into the prompt string, then restores the original stack.
fn eval_custom_prompt(state: &mut State) -> Option<String> {
    // Check if $prompt is defined in the dictionary
    if !state.dict.contains_key("$prompt") {
        return None;
    }

    // Save the real stack
    let saved_stack = std::mem::take(&mut state.stack);
    state.prompt_eval_original_stack = Some(saved_stack.clone());

    // Evaluate $prompt
    let result = eval::eval_line(state, "$prompt");

    // Collect the prompt from the stack
    let prompt = if result.is_ok() {
        state
            .stack
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("")
    } else {
        String::new()
    };

    // Restore the original stack
    state.stack = saved_stack;
    state.prompt_eval_original_stack = None;

    if prompt.is_empty() && result.is_err() {
        None
    } else {
        Some(prompt)
    }
}

/// Auto-type: if top of stack is Output, print it (but keep it on stack).
fn auto_type_output(state: &State) {
    if let Some(Value::Output(s)) = state.stack.last() {
        print!("{}", s);
    }
}

/// Load and evaluate the RC file (~/.yafshrc) if it exists.
fn load_rc(state: &mut State) {
    if let Some(path) = config::rc_path() {
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                for line in contents.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    if let Err(e) = eval::eval_line(state, trimmed) {
                        eprintln!("~/.yafshrc: {}", e);
                    }
                }
            }
        }
    }
}

/// Run the interactive REPL with rustyline (when stdin is a TTY).
fn run_interactive(state: &mut State) {
    let helper = YafshHelper::new();
    let mut rl = match Editor::with_config(
        rustyline::Config::builder()
            .auto_add_history(true)
            .build(),
    ) {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("Failed to initialize editor: {}", e);
            run_simple(state);
            return;
        }
    };

    rl.set_helper(Some(helper));

    // Load history
    if let Some(path) = config::history_path() {
        let _ = rl.load_history(&path);
    }

    println!("yafsh {}", config::VERSION);
    println!("Type 'exit' to quit, Ctrl-D for EOF");
    println!();

    loop {
        // Build prompt (custom or default)
        let prompt = eval_custom_prompt(state).unwrap_or_else(|| build_default_prompt(state));

        // Sync dictionary words to helper for completion and highlighting
        if let Some(helper) = rl.helper_mut() {
            helper.update_words(state.dict.keys().cloned());
        }

        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "exit" || trimmed == "quit" {
                    println!("Goodbye!");
                    break;
                }

                match eval::eval_line(state, trimmed) {
                    Ok(()) => {
                        auto_type_output(state);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: cancel current line, continue
                println!();
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("\nGoodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Read error: {}", err);
                break;
            }
        }
    }

    // Save history
    if let Some(path) = config::history_path() {
        let _ = rl.save_history(&path);
    }
}

/// Run the simple REPL for pipe mode (when stdin is not a TTY).
fn run_simple(state: &mut State) {
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                // EOF
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "exit" || trimmed == "quit" {
                    break;
                }

                match eval::eval_line(state, trimmed) {
                    Ok(()) => {
                        auto_type_output(state);
                        io::stdout().flush().ok();
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

fn main() {
    let mut state = State::new();
    builtins::register_builtins(&mut state);

    // Load RC file
    load_rc(&mut state);

    if io::stdin().is_terminal() {
        run_interactive(&mut state);
    } else {
        run_simple(&mut state);
    }
}

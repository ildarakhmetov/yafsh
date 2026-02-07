pub mod io;
pub mod stack;
pub mod system;

use crate::types::{State, Word};

/// Register all builtin words into the state dictionary.
pub fn register_builtins(state: &mut State) {
    let reg = |state: &mut State, name: &str, f: fn(&mut State) -> Result<(), String>, doc: &'static str| {
        state.dict.insert(name.to_string(), Word::Builtin(f, Some(doc)));
    };

    // Stack manipulation
    reg(state, "dup", stack::dup, "( a -- a a ) Duplicate top item");
    reg(state, "swap", stack::swap, "( a b -- b a ) Swap top two items");
    reg(state, "drop", stack::drop_word, "( a -- ) Remove top item");
    reg(state, "clear", stack::clear, "( ... -- ) Clear entire stack");
    reg(state, "over", stack::over, "( a b -- a b a ) Copy second item to top");
    reg(state, "rot", stack::rot, "( a b c -- b c a ) Rotate top three items");

    // I/O
    reg(state, ".", io::dot, "( a -- ) Print and remove top item with newline");
    reg(state, "type", io::type_word, "( a -- ) Print and remove top item without newline");
    reg(state, ".s", io::dot_s, "( -- ) Display entire stack without modifying it");
    reg(state, ">output", io::to_output, "( string -- output ) Convert Str to Output for piping");
    reg(state, ">string", io::to_string_word, "( output/int -- string ) Convert Output or Int to Str");

    // System
    reg(state, "exec", system::exec_word, "( args... cmd -- output ) Execute shell command");
    reg(state, "?", system::exit_code, "( -- code ) Push exit code of last command");
    reg(state, "cd", system::cd, "( path -- ) Change directory");
}

pub mod computation;
pub mod introspection;
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

    // File I/O
    reg(state, ">file", io::write_file, "( content filename -- ) Write output to file");
    reg(state, ">>file", io::append_file, "( content filename -- ) Append output to file");

    // System
    reg(state, "exec", system::exec_word, "( args... cmd -- output ) Execute shell command");
    reg(state, "?", system::exit_code, "( -- code ) Push exit code of last command");
    reg(state, "cd", system::cd, "( path -- ) Change directory");

    // Environment
    reg(state, "getenv", system::getenv, "( key -- value ) Get environment variable");
    reg(state, "setenv", system::setenv, "( value key -- ) Set environment variable");
    reg(state, "unsetenv", system::unsetenv, "( key -- ) Unset environment variable");
    reg(state, "env-append", system::env_append, "( value key -- ) Append to colon-separated env var");
    reg(state, "env-prepend", system::env_prepend, "( value key -- ) Prepend to colon-separated env var");
    reg(state, "env", system::env_all, "( -- vars... ) Push all environment variables");

    // Directory navigation
    reg(state, "pushd", system::pushd, "( path -- ) Push current dir and change to path");
    reg(state, "popd", system::popd, "( -- ) Pop and change to directory from stack");

    // Arithmetic
    reg(state, "+", computation::add, "( a b -- a+b ) Add two numbers");
    reg(state, "-", computation::sub, "( a b -- a-b ) Subtract b from a");
    reg(state, "*", computation::mul, "( a b -- a*b ) Multiply two numbers");
    reg(state, "/", computation::div, "( a b -- a/b ) Divide a by b");
    reg(state, "mod", computation::mod_op, "( a b -- a%b ) Modulo (remainder of a/b)");
    reg(state, "/mod", computation::divmod, "( a b -- quot rem ) Quotient and remainder");
    reg(state, "*/", computation::muldiv, "( a b c -- (a*b)/c ) Multiply then divide");

    // Comparisons
    reg(state, "=", computation::eq, "( a b -- flag ) Test equality (1 if equal, 0 if not)");
    reg(state, ">", computation::gt, "( a b -- flag ) Test greater than");
    reg(state, "<", computation::lt, "( a b -- flag ) Test less than");
    reg(state, ">=", computation::gte, "( a b -- flag ) Test greater or equal");
    reg(state, "<=", computation::lte, "( a b -- flag ) Test less or equal");
    reg(state, "<>", computation::neq, "( a b -- flag ) Test not equal");

    // Boolean logic
    reg(state, "and", computation::bool_and, "( a b -- flag ) Boolean AND");
    reg(state, "or", computation::bool_or, "( a b -- flag ) Boolean OR");
    reg(state, "not", computation::bool_not, "( a -- flag ) Boolean NOT");
    reg(state, "xor", computation::bool_xor, "( a b -- flag ) Boolean XOR");

    // String operations
    reg(state, "concat", computation::concat, "( a b -- a+b ) Concatenate two strings");

    // Loop indices
    reg(state, "i", computation::loop_i, "( -- index ) Push current loop index");
    reg(state, "j", computation::loop_j, "( -- index ) Push outer loop index (nested loops)");

    // Introspection
    reg(state, "words", introspection::words, "List all available words");
    reg(state, "help", introspection::help, "Show comprehensive help information");
    reg(state, "see", introspection::see, "( name -- ) Show word definition or documentation");
}

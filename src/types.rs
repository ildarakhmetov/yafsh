use std::collections::HashMap;

/// Core value types on the stack.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// User input, command arguments
    Str(String),
    /// Integer value
    Int(i64),
    /// Output from a shell command (automatically pipes to next command as stdin)
    Output(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(s) => write!(f, "{}", s),
            Value::Int(n) => write!(f, "{}", n),
            Value::Output(s) => write!(f, "{}", s),
        }
    }
}

pub type Stack = Vec<Value>;

/// A builtin function that operates on the full interpreter state.
pub type BuiltinFn = fn(&mut State) -> Result<(), String>;

/// Word types in the dictionary.
#[derive(Clone)]
#[allow(dead_code)]
pub enum Word {
    /// Native builtin function with optional doc string
    Builtin(BuiltinFn, Option<&'static str>),
    /// User-defined word (list of tokens to replay)
    Defined(Vec<String>),
    /// External shell command (cached path)
    #[allow(dead_code)]
    ShellCmd(String),
}

/// Loop type during body collection.
#[derive(Clone, Debug)]
pub enum LoopType {
    BeginUntil,
    BeginWhile,
    DoLoop,
    DoPlusLoop,
}

/// Active loop info (for i/j index access).
#[derive(Clone, Debug)]
pub enum LoopInfo {
    DoCountedLoop { start: i64, limit: i64, current: i64 },
    DoPlusCountedLoop { start: i64, limit: i64, current: i64 },
    BeginUntilLoop,
    BeginWhileLoop,
}

/// Control flow target for skipping.
#[derive(Clone, Debug)]
pub enum SkipTarget {
    Else,
    Then,
}

/// Control flow state for if/then/else.
#[derive(Clone, Debug)]
pub enum ControlFlow {
    Normal,
    Skipping { target: SkipTarget, depth: usize },
}

/// The full interpreter state.
pub struct State {
    pub stack: Stack,
    pub dict: HashMap<String, Word>,
    /// Currently defining a word (name)
    pub defining: Option<String>,
    /// Body of word being defined (accumulated tokens)
    pub def_body: Vec<String>,
    /// Exit code of last shell command
    pub last_exit_code: i32,
    /// Control flow state for if/then/else
    pub control_flow: ControlFlow,
    /// Directory stack for pushd/popd
    pub dir_stack: Vec<String>,
    /// Stack of active loops for i/j index access
    pub loop_stack: Vec<LoopInfo>,
    /// Collecting loop body: (loop_type, body_tokens, nesting_depth)
    pub collecting_loop: Option<(LoopType, Vec<String>, usize)>,
    /// Collecting each body: (output_content, body_tokens)
    pub collecting_each: Option<(String, Vec<String>)>,
    /// Cached result of evaluating the `$prompt` word (custom prompt string)
    pub custom_prompt: Option<String>,
    /// Saved stack during prompt evaluation so $stack/$in/$out see the real stack
    pub prompt_eval_original_stack: Option<Vec<Value>>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        State {
            stack: Vec::new(),
            dict: HashMap::new(),
            defining: None,
            def_body: Vec::new(),
            last_exit_code: 0,
            control_flow: ControlFlow::Normal,
            dir_stack: Vec::new(),
            loop_stack: Vec::new(),
            collecting_loop: None,
            collecting_each: None,
            custom_prompt: None,
            prompt_eval_original_stack: None,
        }
    }
}

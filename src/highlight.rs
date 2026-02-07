use std::borrow::Cow;
use std::collections::HashSet;

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper, Result};

use crate::multiline;
use crate::tokenizer;

/// The rustyline helper for yafsh.
///
/// Combines syntax highlighting, input validation (multiline detection),
/// tab-completion (dictionary words + filenames), and hinting.
pub struct YafshHelper {
    /// Set of known dictionary words, synced before each readline.
    pub dict_words: HashSet<String>,
    /// Filename completer for path completion.
    file_completer: FilenameCompleter,
}

impl Default for YafshHelper {
    fn default() -> Self {
        Self::new()
    }
}

impl YafshHelper {
    pub fn new() -> Self {
        YafshHelper {
            dict_words: HashSet::new(),
            file_completer: FilenameCompleter::new(),
        }
    }

    /// Update the set of known dictionary words.
    pub fn update_words(&mut self, words: impl IntoIterator<Item = String>) {
        self.dict_words.clear();
        self.dict_words.extend(words);
    }
}

impl Helper for YafshHelper {}

// ========== Highlighter ==========

/// ANSI color codes.
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

/// Keywords that get magenta highlighting.
const KEYWORDS: &[&str] = &[
    ":", ";", "if", "else", "then", "begin", "until", "while", "repeat", "do", "loop", "+loop",
    "each", "exit", "quit",
];

impl Highlighter for YafshHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if line.is_empty() {
            return Cow::Borrowed(line);
        }

        let tokens = tokenizer::tokenize_with_positions(line);
        if tokens.is_empty() {
            return Cow::Borrowed(line);
        }

        let mut result = String::with_capacity(line.len() + tokens.len() * 10);
        let mut last_end: usize = 0;

        for tok in &tokens {
            let start = tok.position;
            // Calculate end position in the original string
            let end = if tok.quoted {
                // For quoted tokens, position points to opening quote
                // end is after closing quote (or end of string for unclosed)
                let content_start = start + 1; // after opening "
                let content_end = content_start + tok.text.len();
                // Check if there's a closing quote
                if content_end < line.len() && line.as_bytes()[content_end] == b'"' {
                    content_end + 1
                } else {
                    content_end
                }
            } else {
                start + tok.text.len()
            };

            // Append any gap between last token end and this token start
            if start > last_end {
                result.push_str(&line[last_end..start]);
            }

            let token_text = &line[start..end.min(line.len())];

            // Determine color
            if tok.quoted {
                // Strings are yellow
                result.push_str(YELLOW);
                result.push_str(token_text);
                result.push_str(RESET);
            } else if KEYWORDS.contains(&tok.text.as_str()) {
                // Keywords are magenta
                result.push_str(MAGENTA);
                result.push_str(token_text);
                result.push_str(RESET);
            } else if tok.text.parse::<i64>().is_ok() {
                // Numbers are cyan
                result.push_str(CYAN);
                result.push_str(token_text);
                result.push_str(RESET);
            } else if self.dict_words.contains(&tok.text) {
                // Dictionary words are green
                result.push_str(GREEN);
                result.push_str(token_text);
                result.push_str(RESET);
            } else {
                result.push_str(token_text);
            }

            last_end = end.min(line.len());
        }

        // Append any trailing text
        if last_end < line.len() {
            result.push_str(&line[last_end..]);
        }

        Cow::Owned(result)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        // Always re-highlight (simple approach)
        true
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _default: bool) -> Cow<'b, str> {
        Cow::Borrowed(prompt)
    }
}

// ========== Validator ==========

impl Validator for YafshHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
        let input = ctx.input();
        if multiline::is_incomplete(input) {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

// ========== Completer ==========

impl Completer for YafshHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>)> {
        // Find the word being typed
        let (word_start, word) = find_word_at(line, pos);

        if word.is_empty() {
            return Ok((pos, Vec::new()));
        }

        // Try dictionary word completion
        let mut completions: Vec<Pair> = self
            .dict_words
            .iter()
            .filter(|w| w.starts_with(word))
            .map(|w| Pair {
                display: w.clone(),
                replacement: w.clone(),
            })
            .collect();
        completions.sort_by(|a, b| a.display.cmp(&b.display));

        // Also try filename completion
        if let Ok((file_start, file_completions)) = self.file_completer.complete(line, pos, ctx) {
            if !file_completions.is_empty() {
                // If file completions exist, merge them
                // Use the file_start position if file completions are more specific
                if !file_completions.is_empty() && completions.is_empty() {
                    return Ok((file_start, file_completions));
                }
                // Merge both sets using word_start
                for fc in file_completions {
                    completions.push(fc);
                }
            }
        }

        Ok((word_start, completions))
    }
}

/// Find the word being typed at the cursor position.
/// Returns (start_position, word_slice).
fn find_word_at(line: &str, pos: usize) -> (usize, &str) {
    let bytes = line.as_bytes();
    let mut start = pos;
    while start > 0 && !bytes[start - 1].is_ascii_whitespace() {
        start -= 1;
    }
    (start, &line[start..pos])
}

// ========== Hinter (no-op) ==========

impl Hinter for YafshHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

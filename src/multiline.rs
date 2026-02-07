/// Check whether the given input text is incomplete and needs continuation lines.
///
/// Returns `true` if the input has:
/// - Unclosed double quotes (odd count of `"`)
/// - Unbalanced `:` vs `;`
/// - Unbalanced `begin` vs `until`/`repeat`
/// - Unbalanced `do` vs `loop`/`+loop`
/// - Unbalanced `if`/`each` vs `then`
pub fn is_incomplete(text: &str) -> bool {
    // Check unclosed quotes: odd number of unescaped double-quotes
    let quote_count = text.chars().filter(|&c| c == '"').count();
    if quote_count % 2 != 0 {
        return true;
    }

    // Tokenize by whitespace for keyword balancing (ignore quoted regions)
    let words = extract_words(text);

    let mut colon_depth: i32 = 0;
    let mut begin_depth: i32 = 0;
    let mut do_depth: i32 = 0;
    let mut if_each_depth: i32 = 0;

    for word in &words {
        match word.as_str() {
            ":" => colon_depth += 1,
            ";" => colon_depth -= 1,
            "begin" => begin_depth += 1,
            "until" | "repeat" => begin_depth -= 1,
            "do" => do_depth += 1,
            "loop" | "+loop" => do_depth -= 1,
            "if" | "each" => if_each_depth += 1,
            "then" => if_each_depth -= 1,
            _ => {}
        }
    }

    colon_depth > 0 || begin_depth > 0 || do_depth > 0 || if_each_depth > 0
}

/// Extract words from text, skipping content inside double quotes.
fn extract_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;

    for c in text.chars() {
        if c == '"' {
            in_quote = !in_quote;
            continue;
        }
        if in_quote {
            continue;
        }
        if c.is_whitespace() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_simple() {
        assert!(!is_incomplete("hello world"));
    }

    #[test]
    fn test_incomplete_unclosed_quote() {
        assert!(is_incomplete("\"hello world"));
    }

    #[test]
    fn test_complete_closed_quote() {
        assert!(!is_incomplete("\"hello world\""));
    }

    #[test]
    fn test_incomplete_colon_no_semicolon() {
        assert!(is_incomplete(": greet \"hello\""));
    }

    #[test]
    fn test_complete_colon_semicolon() {
        assert!(!is_incomplete(": greet \"hello\" ;"));
    }

    #[test]
    fn test_incomplete_begin_no_until() {
        assert!(is_incomplete("begin 1 +"));
    }

    #[test]
    fn test_complete_begin_until() {
        assert!(!is_incomplete("begin 1 + dup 5 = until"));
    }

    #[test]
    fn test_incomplete_begin_while_no_repeat() {
        assert!(is_incomplete("begin dup 0 > while 1 -"));
    }

    #[test]
    fn test_complete_begin_while_repeat() {
        assert!(!is_incomplete("begin dup 0 > while 1 - repeat"));
    }

    #[test]
    fn test_incomplete_do_no_loop() {
        assert!(is_incomplete("0 5 do i"));
    }

    #[test]
    fn test_complete_do_loop() {
        assert!(!is_incomplete("0 5 do i + loop"));
    }

    #[test]
    fn test_complete_do_plus_loop() {
        assert!(!is_incomplete("0 10 do i 2 +loop"));
    }

    #[test]
    fn test_incomplete_if_no_then() {
        assert!(is_incomplete("1 if 42"));
    }

    #[test]
    fn test_complete_if_then() {
        assert!(!is_incomplete("1 if 42 then"));
    }

    #[test]
    fn test_incomplete_each_no_then() {
        assert!(is_incomplete("each ."));
    }

    #[test]
    fn test_complete_each_then() {
        assert!(!is_incomplete("each . then"));
    }

    #[test]
    fn test_incomplete_nested() {
        assert!(is_incomplete(": foo if 42"));
    }

    #[test]
    fn test_complete_nested() {
        assert!(!is_incomplete(": foo if 42 then ;"));
    }

    #[test]
    fn test_keywords_in_quotes_ignored() {
        // "if" inside quotes should not count
        assert!(!is_incomplete("\"if\" ."));
    }

    #[test]
    fn test_empty_string() {
        assert!(!is_incomplete(""));
    }

    #[test]
    fn test_multiline_definition() {
        assert!(is_incomplete(": greet\n  \"hello\" ."));
        assert!(!is_incomplete(": greet\n  \"hello\" . ;"));
    }
}

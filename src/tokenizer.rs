/// A token with its quote status.
/// `text` is the token content, `quoted` indicates if it was inside double quotes.
pub struct Token {
    pub text: String,
    pub quoted: bool,
}

/// Tokenize a line of input with quote awareness.
///
/// - Quoted strings (`"hello world"`) become a single token with `quoted = true`.
/// - Whitespace outside quotes separates tokens.
/// - Returns a list of (text, is_quoted) pairs.
pub fn tokenize(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let chars = line.chars();

    for c in chars {
        match c {
            '"' if !in_quote => {
                // Start of quoted string: flush any current unquoted token
                if !current.is_empty() {
                    tokens.push(Token {
                        text: std::mem::take(&mut current),
                        quoted: false,
                    });
                }
                in_quote = true;
            }
            '"' if in_quote => {
                // End of quoted string: emit as quoted token (even if empty)
                tokens.push(Token {
                    text: std::mem::take(&mut current),
                    quoted: true,
                });
                in_quote = false;
            }
            c if c.is_whitespace() && !in_quote => {
                // Whitespace outside quotes: token separator
                if !current.is_empty() {
                    tokens.push(Token {
                        text: std::mem::take(&mut current),
                        quoted: false,
                    });
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    // Flush remaining token
    if !current.is_empty() {
        tokens.push(Token {
            text: current,
            quoted: in_quote, // unclosed quote stays quoted
        });
    }

    tokens
}

/// Check if a string represents an integer.
pub fn is_int(s: &str) -> bool {
    s.parse::<i64>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let tokens = tokenize("hello world");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert!(!tokens[0].quoted);
        assert_eq!(tokens[1].text, "world");
        assert!(!tokens[1].quoted);
    }

    #[test]
    fn test_quoted_string() {
        let tokens = tokenize("\"hello world\" foo");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello world");
        assert!(tokens[0].quoted);
        assert_eq!(tokens[1].text, "foo");
        assert!(!tokens[1].quoted);
    }

    #[test]
    fn test_empty_quoted_string() {
        let tokens = tokenize("\"\" foo");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "");
        assert!(tokens[0].quoted);
    }

    #[test]
    fn test_integers() {
        assert!(is_int("42"));
        assert!(is_int("-1"));
        assert!(is_int("0"));
        assert!(!is_int("hello"));
        assert!(!is_int("12abc"));
    }

    #[test]
    fn test_mixed() {
        let tokens = tokenize(": greet \"hello\" . ;");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].text, ":");
        assert_eq!(tokens[1].text, "greet");
        assert_eq!(tokens[2].text, "hello");
        assert!(tokens[2].quoted);
        assert_eq!(tokens[3].text, ".");
        assert_eq!(tokens[4].text, ";");
    }
}

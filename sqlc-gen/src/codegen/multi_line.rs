use proc_macro2::{Punct, Spacing, TokenStream};
use quote::ToTokens;

/// Returns a `Punct` (punctuation) token for a given character `c`.
/// Creates a punctuation token from a character with `Spacing::Joint`.
/// It is useful for converting characters into token streams in Rust's procedural macros.
pub fn get_punct_from_char(c: char) -> Punct {
    Punct::new(c, Spacing::Joint)
}

/// Converts a character `c` into a `TokenStream`.
/// It uses `get_punct_from_char` to create a `Punct` and then converts it to `TokenStream`.
pub fn get_punct_from_char_tokens(c: char) -> TokenStream {
    get_punct_from_char(c).to_token_stream()
}

/// Returns a `TokenStream` representing a newline character (`\n`).
/// Useful for adding newlines when generating code with procedural macros.
pub fn get_newline_tokens() -> TokenStream {
    get_punct_from_char('\n').to_token_stream()
}

/// A helper struct to represent a multi-line string.
#[derive(Debug, Clone)]
pub struct MultiLine<'a>(pub &'a str);

impl<'a> MultiLine<'a> {
    /// Converts a single line of text into a `TokenStream`.
    /// It iterates over each character in the line and converts it to a `TokenStream`.
    fn line_to_tokens(&self, line: &str) -> TokenStream {
        line.chars().map(get_punct_from_char_tokens).collect()
    }

    /// Converts the entire multi-line string into a `TokenStream`.
    /// It processes each line and collects all the `TokenStream`s, adding newlines as necessary.
    fn lines_to_tokens(&self) -> (TokenStream, usize) {
        let mut tokens = TokenStream::new();
        let mut total_lines = 0;

        for (i, line) in self.0.lines().enumerate() {
            if i > 0 {
                tokens.extend(get_newline_tokens());
            }
            tokens.extend(self.line_to_tokens(line));
            total_lines += 1;
        }

        (tokens, total_lines)
    }
}

impl<'a> ToTokens for MultiLine<'a> {
    /// Converts the multi-line string into a `TokenStream` and appends it to `tokens`.
    /// If there are multiple lines, it surrounds the generated code with newlines.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (lines_tokens, total_lines) = self.lines_to_tokens();

        if total_lines > 1 {
            tokens.extend(get_newline_tokens());
            tokens.extend(lines_tokens);
            tokens.extend(get_newline_tokens());
        } else {
            tokens.extend(lines_tokens);
        }
    }
}

/// A helper struct to represent a multi-line string wrapped in raw string literal syntax.
#[derive(Debug, Clone)]
pub struct MultiLineString<'a>(pub &'a str);

impl<'a> ToTokens for MultiLineString<'a> {
    /// Converts the multi-line string into a raw string literal (`r#"..."#`) in `TokenStream` form.
    /// Wraps the multi-line string in a raw string literal and adds necessary delimiters.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Add the raw string literal delimiters (`r#"`)
        for &c in &['r', '#', '"'] {
            tokens.extend(get_punct_from_char_tokens(c));
        }

        // Add the multi-line string content
        tokens.extend(MultiLine(self.0).to_token_stream());

        // Add the closing raw string literal delimiters (`"#`)
        for &c in &['"', '#', ' '] {
            tokens.extend(get_punct_from_char_tokens(c));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_punct_from_char() {
        let punct = get_punct_from_char('a');
        assert_eq!(punct.to_string(), "a");
    }

    #[test]
    fn test_get_punct_from_char_tokens() {
        let tokens = get_punct_from_char_tokens('b');
        let expected = "b".to_string();
        assert_eq!(tokens.to_string(), expected);
    }

    #[test]
    fn test_get_newline_tokens() {
        let newline_tokens = get_newline_tokens();
        assert_eq!(newline_tokens.to_string(), "\n".to_string());
    }

    #[test]
    fn test_multi_line_single_line() {
        let multiline = MultiLine("Hello, world!");
        let mut tokens = TokenStream::new();
        multiline.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "Hello, world!");
    }

    #[test]
    fn test_multi_line_multiple_lines() {
        let multiline = MultiLine("Hello,\nworld!");
        let mut tokens = TokenStream::new();
        multiline.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "Hello,\nworld!");
    }

    #[test]
    fn test_multi_line_string() {
        let multiline_str = MultiLineString("Hello, world!\nThis is a test.");
        let mut tokens = TokenStream::new();
        multiline_str.to_tokens(&mut tokens);
        assert_eq!(
            tokens.to_string(),
            r#"Hello, world!
This is a test."#
        );
    }
}

use proc_macro2::{Punct, Spacing, TokenStream};
use quote::ToTokens;

pub fn get_punct_from_char(c: char) -> Punct {
    Punct::new(c, Spacing::Joint)
}

pub fn get_punct_from_char_tokens(c: char) -> TokenStream {
    get_punct_from_char(c).to_token_stream()
}

pub fn get_newline_tokens() -> TokenStream {
    let newline_char = char::from_u32(0x000A).unwrap();
    get_punct_from_char_tokens(newline_char)
}

#[derive(Debug, Clone)]
pub struct MultiLine<'a>(pub &'a str);

impl<'a> MultiLine<'a> {
    fn line_to_tokens(&self, line: &str) -> TokenStream {
        let mut tokens = TokenStream::new();
        for c in line.chars() {
            tokens.extend(get_punct_from_char_tokens(c));
        }

        tokens
    }

    fn lines_to_tokens(&self) -> (TokenStream, usize) {
        let mut tokens = TokenStream::new();
        let mut total_lines = 0;
        let lines = self.0.lines();
        for (i, line) in lines.enumerate() {
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

#[derive(Debug, Clone)]
pub struct MultiLineString<'a>(pub &'a str);

impl<'a> ToTokens for MultiLineString<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        ['r', '#', '"'].map(|c| tokens.extend(get_punct_from_char_tokens(c)));
        tokens.extend(MultiLine(self.0).to_token_stream());
        ['"', '#', ' '].map(|c| tokens.extend(get_punct_from_char_tokens(c)));
    }
}

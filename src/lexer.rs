use std::ops::Range;

use crate::error::{LexerError, Spanned};

pub type Token = Spanned<TokenKind>;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Open,
    Close,
    Separator,
    Const(Const),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Const {
    Int(i64),
    Float(f64),
    Str(String),
    Name(String),
}

const SPLITTERS: &[&str] = &["->", "=", ":", ".", "?"];
const OPEN_CHAR: char = '(';
const CLOSE_CHAR: char = ')';
const STR_CHAR: char = '"';
const SEPARATOR_CHAR: char = ',';
const INLINE_COMMENT_STARTER: &str = "//";

pub fn tokenize(code: &str) -> Result<Vec<Token>, LexerError> {
    enum Status {
        Normal,
        Name,
        InlineComment,
        Str,
        Num,
    }

    struct Eater<'a> {
        s: &'a str,
        pos: usize,
        start: usize,
        status: Status,
        result: Vec<Token>,
    }

    impl<'a> Eater<'a> {
        fn get(self) -> Vec<Token> {
            self.result
        }

        fn push<E>(&mut self, f: impl FnOnce(&str) -> Result<TokenKind, E>) -> Result<(), E> {
            if self.start != self.pos {
                self.result.push(Token {
                    span: self.span(),
                    value: f(self.get_current())?,
                });
                self.reset();
            }
            Ok(())
        }

        fn get_current(&self) -> &str {
            &self.s[self.span()]
        }

        fn span(&self) -> Range<usize> {
            self.start..self.pos
        }

        fn push_name(&mut self) {
            self.push::<()>(|s| Ok(TokenKind::Const(Const::Name(String::from(s)))))
                .unwrap();
        }

        fn push_num(&mut self) -> Result<(), LexerError> {
            self.push(|s| s.parse().map(|n| TokenKind::Const(Const::Int(n))))
                .or_else(|_| self.push(|s| s.parse().map(|n| TokenKind::Const(Const::Float(n)))))
                .map_err(|_| LexerError::NumError(self.span()))
        }

        fn push_str(&mut self) {
            let content = &self.s[self.start + STR_CHAR.len_utf8()..self.pos - STR_CHAR.len_utf8()];
            self.result.push(Token {
                span: self.span(),
                value: TokenKind::Const(Const::Str(String::from(content))),
            });
            self.reset();
        }

        fn step(&mut self, i: usize) {
            self.pos += i
        }

        fn get_left(&self) -> &str {
            &self.s[self.pos..]
        }

        fn next(&mut self) -> Option<char> {
            if self.pos < self.s.len() {
                let c = self.s[self.pos..].chars().next().unwrap();
                Some(c)
            } else {
                None
            }
        }

        fn reset(&mut self) {
            self.start = self.pos;
        }
    }

    let mut eater = Eater {
        s: code,
        pos: 0,
        start: 0,
        status: Status::Normal,
        result: Vec::new(),
    };

    while let Some(c) = eater.next() {
        match eater.status {
            Status::Normal => {
                if c.is_whitespace() {
                    eater.step(c.len_utf8());
                    eater.reset();
                } else if c == OPEN_CHAR {
                    eater.step(c.len_utf8());
                    eater.push::<()>(|_| Ok(TokenKind::Open)).unwrap();
                } else if c == CLOSE_CHAR {
                    eater.step(c.len_utf8());
                    eater.push::<()>(|_| Ok(TokenKind::Close)).unwrap();
                } else if c == SEPARATOR_CHAR {
                    eater.step(c.len_utf8());
                    eater.push::<()>(|_| Ok(TokenKind::Separator)).unwrap();
                } else if eater.get_left().starts_with(INLINE_COMMENT_STARTER) {
                    eater.status = Status::InlineComment;
                } else if c.is_ascii_digit() {
                    eater.status = Status::Num;
                } else if c == STR_CHAR {
                    eater.step(c.len_utf8());
                    eater.status = Status::Str;
                } else {
                    eater.status = Status::Name;
                }
            }
            Status::Name => {
                if c.is_whitespace()
                    || c == OPEN_CHAR
                    || c == CLOSE_CHAR
                    || c == STR_CHAR
                    || c == SEPARATOR_CHAR
                    || eater.get_left().starts_with(INLINE_COMMENT_STARTER)
                {
                    eater.push_name();
                    eater.status = Status::Normal;
                } else if let Some(splitter) = SPLITTERS
                    .iter()
                    .find(|&&sp| eater.get_left().starts_with(sp))
                {
                    eater.push_name();
                    eater.step(splitter.len());
                    eater.push_name();
                    eater.status = Status::Normal;
                } else {
                    eater.step(c.len_utf8());
                }
            }
            Status::InlineComment => {
                if c == '\n' || c == '\r' {
                    eater.status = Status::Normal;
                } else {
                    eater.step(c.len_utf8());
                }
            }
            Status::Num => {
                if !c.is_ascii_digit() && c != '.' {
                    eater.status = Status::Normal;
                    eater.push_num()?;
                } else {
                    eater.step(c.len_utf8());
                }
            }
            Status::Str => {
                if c == STR_CHAR {
                    eater.status = Status::Normal;
                    eater.step(c.len_utf8());
                    eater.push_str();
                } else {
                    eater.step(c.len_utf8());
                }
            }
        }
    }

    match eater.status {
        Status::Normal => (),
        Status::Name => eater.push_name(),
        Status::InlineComment => (),
        Status::Num => eater.push_num()?,
        Status::Str => return Err(LexerError::StrError(eater.span())),
    }

    Ok(eater.get())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_token_kind(input: &str) -> Vec<TokenKind> {
        tokenize(input)
            .unwrap()
            .into_iter()
            .map(|t| t.value)
            .collect()
    }

    fn name(name: &str) -> TokenKind {
        TokenKind::Const(Const::Name(String::from(name)))
    }

    fn string(string: &str) -> TokenKind {
        TokenKind::Const(Const::Str(String::from(string)))
    }

    fn int(int: i64) -> TokenKind {
        TokenKind::Const(Const::Int(int))
    }

    fn float(float: f64) -> TokenKind {
        TokenKind::Const(Const::Float(float))
    }

    #[test]
    fn tokenize_basic() {
        let input = "a b\nc  \t d 33 e(->\t):f,.xxxxxx?\nx0.14 0.14\"a\"// \"abc";
        let output = vec![
            name("a"),
            name("b"),
            name("c"),
            name("d"),
            int(33),
            name("e"),
            TokenKind::Open,
            name("->"),
            TokenKind::Close,
            name(":"),
            name("f"),
            TokenKind::Separator,
            name("."),
            name("xxxxxx"),
            name("?"),
            name("x0"),
            name("."),
            int(14),
            float(0.14),
            string("a"),
        ];
        assert_eq!(get_token_kind(input), output);
    }

    #[test]
    fn tokenize_blank() {
        let input = "";
        let output = vec![];
        assert_eq!(get_token_kind(input), output);
        let input = "       ";
        let output = vec![];
        assert_eq!(get_token_kind(input), output);
        let input = "\n\n\t\t  ";
        let output = vec![];
        assert_eq!(get_token_kind(input), output);
    }

    #[test]
    fn tokenize_inline_comment() {
        let input = "aa bb.c// some // comment";
        let output = vec![name("aa"), name("bb"), name("."), name("c")];
        assert_eq!(get_token_kind(input), output);
        let input = "aa///comment\nbb//another\n//->.?";
        let output = vec![name("aa"), name("bb")];
        assert_eq!(get_token_kind(input), output);
        let input = "//only\n//comment";
        let output = vec![];
        assert_eq!(get_token_kind(input), output);
    }

    #[test]
    fn tokenize_num() {
        let input = "1";
        let output = vec![int(1)];
        assert_eq!(get_token_kind(input), output);
        let input = "001 002";
        let output = vec![int(1), int(2)];
        assert_eq!(get_token_kind(input), output);
        let input = "1.1";
        let output = vec![float(1.1)];
        assert_eq!(get_token_kind(input), output);
    }

    #[test]
    fn tokenize_str() {
        let input = r#"" 1.1" "#;
        let output = vec![string(" 1.1")];
        assert_eq!(get_token_kind(input), output);
        let input = r#""""#;
        let output = vec![string("")];
        assert_eq!(get_token_kind(input), output);
        let input = r#""2"z""?"
az""#;
        let output = vec![
            string("2"),
            name("z"),
            string(""),
            name("?"),
            string("\naz"),
        ];
        assert_eq!(get_token_kind(input), output);
    }
}

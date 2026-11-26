use crate::{
    error::ParserError,
    lexer::{Const, Token, TokenKind},
};
#[derive(Debug, PartialEq)]

pub enum AST {
    Call { left: Box<AST>, right: Box<AST> },
    Const(Const),
    None,
}

pub fn parse(code: Vec<Token>) -> Result<AST, ParserError> {
    parse0(&mut code.into_iter(), true)
}

fn parse0(iter: &mut impl Iterator<Item = Token>, top: bool) -> Result<AST, ParserError> {
    let mut root = AST::None;
    while let Some(token) = iter.next() {
        let next = match token.kind {
            TokenKind::Open => parse0(iter, false)?,
            TokenKind::Close => {
                return if top {
                    Err(ParserError::ExtraTokenAfterClosed(token.into()))
                } else {
                    Ok(root)
                };
            }
            TokenKind::Const(current) => AST::Const(current),
        };
        root = match root {
            AST::None => next,
            left => AST::Call {
                left: Box::new(left),
                right: Box::new(next),
            },
        }
    }
    if top {
        Ok(root)
    } else {
        Err(ParserError::NotClosedError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    #[test]
    fn parse_basic() {
        let input = lexer::tokenize("a (b c)").unwrap();
        let output = AST::Call {
            left: Box::new(AST::Const(Const::Name(String::from("a")))),
            right: Box::new(AST::Call {
                left: Box::new(AST::Const(Const::Name(String::from("b")))),
                right: Box::new(AST::Const(Const::Name(String::from("c")))),
            }),
        };
        assert_eq!(parse(input).unwrap(), output);
    }
}

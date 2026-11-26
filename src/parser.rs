use crate::{
    error::ParserError,
    lexer::{Const, Token, TokenKind},
};
#[derive(Debug, PartialEq)]

pub enum AST {
    Call { left: Box<AST>, right: Box<AST> },
    Name(String),
    Const(Const),
}

pub fn parse(code: Vec<Token>) -> Result<Option<AST>, ParserError> {
    Ok(if let Some(first) = code.first() {
        Some(parse0(first, &code[1..])?)
    } else {
        None
    })
}

fn parse0(next: &Token, rest: &[Token]) -> Result<AST, ParserError> {
    match &next.kind {
        TokenKind::Open => {
            if let Some(first) = rest.first() {
                Ok(parse0(first, &rest[1..])?)
            } else {
                Err(ParserError::NotCloseError(next.into()))
            }
        }
        TokenKind::Close => todo!(),
        value => todo!(),
    }
}

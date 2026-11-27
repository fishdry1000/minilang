use crate::{
    error::ParserError,
    lexer::{Const, Token, TokenKind},
};
#[derive(Debug, PartialEq)]

pub enum AST {
    Call { left: Box<AST>, right: Box<AST> },
    Const(Const),
    Tuple(Vec<AST>),
}

pub fn parse(code: Vec<Token>) -> Result<AST, ParserError> {
    parse0(&mut code.into_iter(), None)
}

fn parse0(iter: &mut impl Iterator<Item = Token>, prev: Option<Token>) -> Result<AST, ParserError> {
    let mut root = None;
    let mut building_tuple: Option<Vec<AST>> = None;
    fn res(root: Option<AST>, building_tuple: Option<Vec<AST>>) -> AST {
        match building_tuple {
            Some(mut tup) => {
                tup.extend(root);
                AST::Tuple(tup)
            }
            None => root.unwrap_or(AST::Const(Const::Unit)),
        }
    }

    while let Some(token) = iter.next() {
        let next = match token.kind {
            TokenKind::Const(current) => AST::Const(current),
            TokenKind::Open => parse0(iter, Some(token))?,
            TokenKind::Close => {
                return match prev {
                    None => Err(ParserError::ExtraCloseError(token.into())),
                    Some(_) => Ok(res(root, building_tuple)),
                };
            }
            TokenKind::Separator => {
                building_tuple.get_or_insert_with(Vec::new).push(
                    root.take()
                        .ok_or_else(|| ParserError::NothingToSeparate(token.into()))?,
                );
                continue;
            }
        };
        root = Some(match root {
            None => next,
            Some(left) => AST::Call {
                left: Box::new(left),
                right: Box::new(next),
            },
        })
    }
    match prev {
        Some(prev) => Err(ParserError::NotClosedError(prev.into())),
        None => Ok(res(root, building_tuple)),
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
        let input = lexer::tokenize("").unwrap();
        let output = AST::Const(Const::Unit);
        assert_eq!(parse(input).unwrap(), output);
    }
    #[test]
    fn parse_comma() {
        let input = lexer::tokenize("a, b").unwrap();
        let output = AST::Tuple(vec![
            AST::Const(Const::Name(String::from("a"))),
            AST::Const(Const::Name(String::from("b"))),
        ]);
        assert_eq!(parse(input).unwrap(), output);
        let input = lexer::tokenize("(a, b, c)").unwrap();
        let output = AST::Tuple(vec![
            AST::Const(Const::Name(String::from("a"))),
            AST::Const(Const::Name(String::from("b"))),
            AST::Const(Const::Name(String::from("c"))),
        ]);
        assert_eq!(parse(input).unwrap(), output);
        let input = lexer::tokenize("a, (b, c,)").unwrap();
        let output = AST::Tuple(vec![
            AST::Const(Const::Name(String::from("a"))),
            AST::Tuple(vec![
                AST::Const(Const::Name(String::from("b"))),
                AST::Const(Const::Name(String::from("c"))),
            ]),
        ]);
        assert_eq!(parse(input).unwrap(), output);
        let input = lexer::tokenize("a,(b,c),d").unwrap();
        let output = AST::Tuple(vec![
            AST::Const(Const::Name(String::from("a"))),
            AST::Tuple(vec![
                AST::Const(Const::Name(String::from("b"))),
                AST::Const(Const::Name(String::from("c"))),
            ]),
            AST::Const(Const::Name(String::from("d"))),
        ]);
        assert_eq!(parse(input).unwrap(), output);
        let input = lexer::tokenize("a, b, (), ").unwrap();
        let output = AST::Tuple(vec![
            AST::Const(Const::Name(String::from("a"))),
            AST::Const(Const::Name(String::from("b"))),
            AST::Const(Const::Unit),
        ]);
        assert_eq!(parse(input).unwrap(), output);
        let input = lexer::tokenize("f(x, y)").unwrap();
        let output = AST::Call {
            left: Box::new(AST::Const(Const::Name(String::from("f")))),
            right: Box::new(AST::Tuple(vec![
                AST::Const(Const::Name(String::from("x"))),
                AST::Const(Const::Name(String::from("y"))),
            ])),
        };
        assert_eq!(parse(input).unwrap(), output);
        let input = lexer::tokenize("(x, y)f").unwrap();
        let output = AST::Call {
            left: Box::new(AST::Tuple(vec![
                AST::Const(Const::Name(String::from("x"))),
                AST::Const(Const::Name(String::from("y"))),
            ])),
            right: Box::new(AST::Const(Const::Name(String::from("f")))),
        };
        assert_eq!(parse(input).unwrap(), output);
        let input = lexer::tokenize("((),), (),").unwrap();
        let output = AST::Tuple(vec![
            AST::Tuple(vec![AST::Const(Const::Unit)]),
            AST::Const(Const::Unit),
        ]);
        assert_eq!(parse(input).unwrap(), output);
    }
}

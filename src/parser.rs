use std::ops::Range;

use crate::{
    error::{ParserError, Spanned},
    lexer::{Const, Token, TokenKind},
};

#[derive(Debug, PartialEq)]
pub enum ASTKind {
    Call { left: Box<AST>, right: Box<AST> },
    Const(Const),
    Tuple(Vec<AST>),
}

pub type AST = Spanned<ASTKind>;

pub fn parse(code: Vec<Token>) -> Result<AST, ParserError> {
    parse0(&mut code.into_iter(), None)
}

fn parse0(
    iter: &mut impl Iterator<Item = Token>,
    open: Option<Range<usize>>,
) -> Result<AST, ParserError> {
    let mut root = None;
    let mut tuple = Vec::new();
    let mut end = 0;

    while let Some(Token { value, span }) = iter.next() {
        end = span.end;
        let next = match value {
            TokenKind::Const(value) => AST {
                value: ASTKind::Const(value),
                span,
            },
            TokenKind::Open => parse0(iter, Some(span))?,
            TokenKind::Close => {
                let open = open.ok_or(ParserError::ExtraCloseError(span))?;
                let mut node = finish(root, tuple, end);
                node.span = open.start..end;
                return Ok(node);
            }
            TokenKind::Separator => {
                tuple.push(root.take().ok_or(ParserError::NothingToSeparate(span))?);
                continue;
            }
        };
        end = next.span.end;
        root = Some(match root {
            None => next,
            Some(left) => AST {
                span: left.span.start..end,
                value: ASTKind::Call {
                    left: Box::new(left),
                    right: Box::new(next),
                },
            },
        });
    }

    match open {
        Some(span) => Err(ParserError::NotClosedError(span)),
        None => Ok(finish(root, tuple, end)),
    }
}

fn finish(root: Option<AST>, mut tuple: Vec<AST>, end: usize) -> AST {
    if let Some(first) = tuple.first() {
        let start = first.span.start;
        tuple.extend(root);
        AST {
            value: ASTKind::Tuple(tuple),
            span: start..end,
        }
    } else {
        root.unwrap_or(AST {
            value: ASTKind::Tuple(tuple),
            span: end..end,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    #[derive(Debug, PartialEq)]
    enum TestAst {
        Call(Box<TestAst>, Box<TestAst>),
        Const(Const),
        Tuple(Vec<TestAst>),
    }

    impl From<AST> for TestAst {
        fn from(ast: AST) -> Self {
            match ast.value {
                ASTKind::Call { left, right } => call((*left).into(), (*right).into()),
                ASTKind::Const(value) => Self::Const(value),
                ASTKind::Tuple(items) => Self::Tuple(items.into_iter().map(Self::from).collect()),
            }
        }
    }

    fn name(value: &str) -> TestAst {
        TestAst::Const(Const::Name(value.to_owned()))
    }

    fn tuple(items: Vec<TestAst>) -> TestAst {
        TestAst::Tuple(items)
    }

    fn call(left: TestAst, right: TestAst) -> TestAst {
        TestAst::Call(Box::new(left), Box::new(right))
    }

    fn parse_source(source: &str) -> TestAst {
        parse(lexer::tokenize(source).unwrap()).unwrap().into()
    }

    #[test]
    fn parse_basic() {
        assert_eq!(
            parse_source("a (b c)"),
            call(name("a"), call(name("b"), name("c"))),
        );
        assert_eq!(parse_source(""), tuple(vec![]));
    }

    #[test]
    fn parse_comma() {
        assert_eq!(parse_source("a, b"), tuple(vec![name("a"), name("b")]));
        assert_eq!(
            parse_source("(a, b, c)"),
            tuple(vec![name("a"), name("b"), name("c")]),
        );
        assert_eq!(
            parse_source("a, (b, c,)"),
            tuple(vec![name("a"), tuple(vec![name("b"), name("c")])]),
        );
        assert_eq!(
            parse_source("a,(b,c),d"),
            tuple(vec![
                name("a"),
                tuple(vec![name("b"), name("c")]),
                name("d"),
            ]),
        );
        assert_eq!(
            parse_source("a, b, (), "),
            tuple(vec![name("a"), name("b"), tuple(vec![])]),
        );
        assert_eq!(
            parse_source("f(x, y)"),
            call(name("f"), tuple(vec![name("x"), name("y")])),
        );
        assert_eq!(
            parse_source("(x, y)f"),
            call(tuple(vec![name("x"), name("y")]), name("f")),
        );
        assert_eq!(
            parse_source("((),), (),"),
            tuple(vec![tuple(vec![tuple(vec![])]), tuple(vec![])]),
        );
    }
}

use crate::{lexer::Const, parser::AST};

pub enum TypedAST {
    Call {
        left: Box<TypedAST>,
        right: Box<TypedAST>,
    },
    Const {
        val: Const,
        typing: AST,
    },
    None,
}

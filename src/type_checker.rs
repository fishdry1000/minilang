use std::sync::atomic::AtomicUsize;

use crate::{
    error::TypeError,
    lexer::Const,
    parser::{AST, ASTKind},
};

#[derive(Debug)]
pub enum TypedAST {
    Call {
        left: Box<TypedAST>,
        right: Box<TypedAST>,
    },
    Transfer {
        from: Box<TypedAST>,
        to: Box<TypedAST>,
    },
    Const(ConstType),
    Tuple(Vec<TypedAST>),
    TypeVar(TypeVar),
}

impl TypedAST {
    fn expand(self) -> Result<Self, TypeError> {
        Ok(self)
    }
}

#[derive(Debug)]
pub enum ConstType {
    Int,
    Float,
    Str,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TypeVar(usize);

static TYPE_VAR_COUNTER: AtomicUsize = AtomicUsize::new(0);
impl TypeVar {
    fn new() -> TypeVar {
        TypeVar(TYPE_VAR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

pub fn check(code: &AST) -> TypedAST {
    build(code)
}

fn get_builtin_type(name: &str) -> Option<TypedAST> {
    match name {
        "*" => Some(TypedAST::Transfer {
            from: Box::new(TypedAST::Tuple(vec![
                TypedAST::Const(ConstType::Int),
                TypedAST::Const(ConstType::Int),
            ])),
            to: Box::new(TypedAST::Const(ConstType::Int)),
        }),
        _ => None,
    }
}

fn build(code: &AST) -> TypedAST {
    match &code.value {
        ASTKind::Call { left, right } => TypedAST::Call {
            left: Box::new(build(left)),
            right: Box::new(build(right)),
        },
        ASTKind::Const(code) => match code {
            Const::Int(_) => TypedAST::Const(ConstType::Int),
            Const::Float(_) => TypedAST::Const(ConstType::Float),
            Const::Str(_) => TypedAST::Const(ConstType::Str),
            Const::Name(name) => {
                get_builtin_type(name).unwrap_or_else(|| TypedAST::TypeVar(TypeVar::new()))
            }
        },
        ASTKind::Tuple(asts) => TypedAST::Tuple(asts.iter().map(|e| build(e)).collect()),
    }
}

use std::sync::atomic::AtomicUsize;

use crate::{lexer::Const, parser::AST};

#[derive(Debug)]
pub enum TypedAST {
    Call {
        left: Box<TypedAST>,
        right: Box<TypedAST>,
    },
    Const(Type),
    Tuple(Vec<TypedAST>),
}

#[derive(Debug)]
pub enum Type {
    Int,
    Float,
    Str,
    Unit,
    TypeVar(TypeVar),
}

#[derive(Debug)]
pub struct TypeVar(usize);

static TYPE_VAR_COUNTER: AtomicUsize = AtomicUsize::new(0);
impl TypeVar {
    fn new() -> TypeVar {
        TypeVar(TYPE_VAR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

impl From<&Const> for Type {
    fn from(value: &Const) -> Self {
        match value {
            Const::Int(_) => Type::Int,
            Const::Float(_) => Type::Float,
            Const::Str(_) => Type::Str,
            Const::Name(_) => Type::TypeVar(TypeVar::new()),
            Const::Unit => Type::Unit,
        }
    }
}

pub fn check(code: &AST) -> TypedAST {
    match code {
        AST::Call { left, right } => TypedAST::Call {
            left: Box::new(check(left)),
            right: Box::new(check(right)),
        },
        AST::Const(code) => TypedAST::Const(Type::from(code)),
        AST::Tuple(asts) => TypedAST::Tuple(asts.iter().map(|e| check(e)).collect()),
    }
}

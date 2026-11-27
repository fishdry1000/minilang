use std::{collections::HashMap, ops::Range, sync::atomic::AtomicUsize};

use crate::{
    error::TypeError,
    lexer::Const,
    parser::{AST, ASTKind},
};

#[derive(Debug)]
pub struct TypedAST {
    typing: TypeVar,
    kind: TypedASTKind,
    span: Range<usize>,
}

impl TypedAST {
    fn expand(self) -> Result<Self, TypeError> {
        Ok(self)
    }
}

#[derive(Debug)]
pub enum TypedASTKind {
    Call {
        left: Box<TypedAST>,
        right: Box<TypedAST>,
    },
    Const(Const),
    Tuple(Vec<TypedAST>),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Type {
    Call { left: TypeVar, right: TypeVar },
    Transfer { from: TypeVar, to: TypeVar },
    Const(TypeVar),
    Tuple(Vec<TypeVar>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeVar {
    Int,
    Float,
    Str,
    TypeVar(usize),
}

static TYPE_VAR_COUNTER: AtomicUsize = AtomicUsize::new(0);
impl TypeVar {
    fn new_type_var() -> TypeVar {
        TypeVar::TypeVar(TYPE_VAR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

pub fn check(code: &AST) -> TypedAST {
    build(code, &mut TypeVarHelper::new())
}

fn get_builtin_type(name: &str, type_vars: &mut TypeVarHelper) -> Option<TypeVar> {
    match name {
        "*" => Some({
            let from = type_vars.link(Type::Tuple(vec![TypeVar::Int, TypeVar::Int]));
            let to = type_vars.link(Type::Const(TypeVar::Int));
            type_vars.link(Type::Transfer { from, to })
        }),
        _ => None,
    }
}

struct TypeVarHelper {
    to_type: HashMap<TypeVar, Type>,
    to_var: HashMap<Type, TypeVar>,
}

impl TypeVarHelper {
    fn new() -> Self {
        TypeVarHelper {
            to_type: HashMap::new(),
            to_var: HashMap::new(),
        }
    }

    fn link(&mut self, new: Type) -> TypeVar {
        if let Type::Call { left, right } = &new {
            let left = self.to_type.get(left).expect("should found");
            if let Type::Transfer { from, to } = left {
                if right == from {
                    let to = to.clone();
                    self.bind(&new, &to);
                    return to;
                }
            }
        }
        match self.to_var.get(&new) {
            Some(i) => i.clone(),
            None => self.new_binding(new),
        }
    }

    fn new_binding(&mut self, t: Type) -> TypeVar {
        let v = match &t {
            Type::Const(type_var) => type_var.clone(),
            _ => TypeVar::new_type_var(),
        };
        self.bind(&t, &v);
        v
    }

    fn bind(&mut self, t: &Type, v: &TypeVar) {
        self.to_type.insert(v.clone(), t.clone());
        self.to_var.insert(t.clone(), v.clone());
    }
}

fn build(code: &AST, type_vars: &mut TypeVarHelper) -> TypedAST {
    let AST { span, value } = code;
    let span = span.clone();
    match value {
        ASTKind::Const(sct) => TypedAST {
            kind: TypedASTKind::Const(sct.clone()),
            typing: match sct {
                Const::Int(_) => TypeVar::Int,
                Const::Float(_) => TypeVar::Float,
                Const::Str(_) => TypeVar::Str,
                Const::Name(name) => {
                    get_builtin_type(name, type_vars).unwrap_or_else(|| TypeVar::new_type_var())
                }
            },
            span,
        },
        ASTKind::Call { left, right } => {
            let left = build(left, type_vars);
            let right = build(right, type_vars);
            TypedAST {
                typing: type_vars.link(Type::Call {
                    left: left.typing.clone(),
                    right: right.typing.clone(),
                }),
                kind: TypedASTKind::Call {
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            }
        }
        ASTKind::Tuple(asts) => {
            let tuple: Vec<TypedAST> = asts.iter().map(|e| build(e, type_vars)).collect();
            TypedAST {
                typing: type_vars.link(Type::Tuple(
                    tuple.iter().map(|e| e.typing.clone()).collect(),
                )),
                kind: TypedASTKind::Tuple(tuple),
                span,
            }
        }
    }
}

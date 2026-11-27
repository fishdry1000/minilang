use std::{collections::HashMap, ops::Range, sync::atomic::AtomicUsize};

use crate::{
    error::TypeError,
    lexer::Const,
    parser::{AST, ASTKind},
};

#[derive(Debug)]
pub struct TypedAST {
    typing: Type,
    kind: TypedASTKind,
    span: Range<usize>,
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
    Call { left: ConstType, right: ConstType },
    Const(ConstType),
    Tuple(Vec<ConstType>),
}

impl TypedAST {
    fn expand(self) -> Result<Self, TypeError> {
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstType {
    Int,
    Float,
    Str,
    TypeVar(usize),
}

static TYPE_VAR_COUNTER: AtomicUsize = AtomicUsize::new(0);
impl ConstType {
    fn new_type_var() -> ConstType {
        ConstType::TypeVar(TYPE_VAR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

pub fn check(code: &AST) -> TypedAST {
    build(code, &mut TypeVarHelper::new())
}

fn get_builtin_type(name: &str, type_vars: &mut TypeVarHelper) -> Option<ConstType> {
    match name {
        "*" => Some({
            let left = type_vars.link(&Type::Tuple(vec![ConstType::Int, ConstType::Int]));
            let right = type_vars.link(&Type::Const(ConstType::Int));
            type_vars.link(&Type::Call { left, right })
        }),
        _ => None,
    }
}

struct TypeVarHelper {
    type_vars: Vec<ConstType>,
    map: HashMap<Type, usize>,
}

impl TypeVarHelper {
    fn new() -> Self {
        TypeVarHelper {
            type_vars: vec![],
            map: HashMap::new(),
        }
    }

    fn link(&mut self, to: &Type) -> ConstType {
        match self.map.get(to) {
            Some(&i) => self.type_vars.get(i).unwrap().clone(),
            None => {
                self.map.insert(to.clone(), self.type_vars.len());
                self.type_vars.push(ConstType::new_type_var());
                self.type_vars.last().unwrap().clone()
            }
        }
    }
}

fn build(code: &AST, type_vars: &mut TypeVarHelper) -> TypedAST {
    let AST { span, value } = code;
    let span = span.clone();
    match value {
        ASTKind::Const(sct) => TypedAST {
            kind: TypedASTKind::Const(sct.clone()),
            typing: Type::Const(match sct {
                Const::Int(_) => ConstType::Int,
                Const::Float(_) => ConstType::Float,
                Const::Str(_) => ConstType::Str,
                Const::Name(name) => {
                    get_builtin_type(name, type_vars).unwrap_or_else(|| ConstType::new_type_var())
                }
            }),
            span,
        },
        ASTKind::Call { left, right } => {
            let left = build(left, type_vars);
            let right = build(right, type_vars);
            TypedAST {
                typing: Type::Call {
                    left: type_vars.link(&left.typing),
                    right: type_vars.link(&right.typing),
                },
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
                typing: Type::Tuple(tuple.iter().map(|e| type_vars.link(&e.typing)).collect()),
                kind: TypedASTKind::Tuple(tuple),
                span,
            }
        }
    }
}

use std::{hash::Hash, ops::Range, sync::atomic::AtomicUsize};

use crate::{
    algorithm::{BiHashMap, UnionFind},
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

#[derive(Debug)]
pub enum TypedASTKind {
    Call {
        left: Box<TypedAST>,
        right: Box<TypedAST>,
    },
    Const(Const),
    Tuple(Vec<TypedAST>),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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

pub fn check(code: &AST) -> Result<TypedAST, TypeError> {
    let mut ts = TypeTree::new();
    let mut tree = build(code, &mut ts)?;
    clean(&mut tree, &mut ts);
    Ok(tree)
}

fn get_builtin_type(name: &str, type_vars: &mut TypeTree) -> TypeVar {
    match name {
        "*" => {
            let from = type_vars
                .link(&Type::Tuple(vec![TypeVar::Int, TypeVar::Int]))
                .unwrap();
            let to = type_vars.link(&Type::Const(TypeVar::Int)).unwrap();
            type_vars.link(&Type::Transfer { from, to }).unwrap()
        }
        "=" => {
            let v = type_vars
                .link(&Type::Const(TypeVar::new_type_var()))
                .unwrap();
            let from = type_vars.link(&Type::Tuple(vec![v.clone(), v])).unwrap();
            let to = type_vars.link(&&Type::Tuple(vec![])).unwrap();
            type_vars.link(&Type::Transfer { from, to }).unwrap()
        }
        _ => type_vars
            .link(&Type::Const(TypeVar::new_type_var()))
            .unwrap(),
    }
}

struct TypeTree {
    type_vars: BiHashMap<TypeVar, Type>,
    union_find: UnionFind<TypeVar>,
}

impl TypeTree {
    fn new() -> Self {
        Self {
            type_vars: BiHashMap::new(),
            union_find: UnionFind::new(),
        }
    }

    fn link(&mut self, new: &Type) -> Result<TypeVar, TypeError> {
        if let Type::Call { left, right } = new {
            let left = self.get(left);
            if let Type::Transfer { from, to } = left.clone() {
                self.union(&from, right)?;
                return Ok(to);
            }
        }
        Ok(self.find_or_new(new))
    }

    fn new_binding(&mut self, t: &Type) -> TypeVar {
        let v = match t {
            Type::Const(type_var) => type_var.clone(),
            _ => TypeVar::new_type_var(),
        };
        self.type_vars.bind(v.clone(), t.clone());
        v
    }

    // normalized
    fn get(&mut self, v: &TypeVar) -> Type {
        let head = self.union_find.find(v);
        // dbg!(&head);
        self.type_vars.get(&head).expect("not found").clone()
    }

    fn find(&mut self, t: &Type) -> Option<TypeVar> {
        let normalized = self.normalize(t);
        self.type_vars.find(&normalized).cloned()
    }

    fn find_or_new(&mut self, t: &Type) -> TypeVar {
        self.find(t).unwrap_or_else(|| self.new_binding(t))
    }

    fn normalize(&mut self, t: &Type) -> Type {
        match t {
            Type::Call { left, right } => Type::Call {
                left: self.union_find.find(left),
                right: self.union_find.find(right),
            },
            Type::Transfer { from, to } => Type::Transfer {
                from: self.union_find.find(from),
                to: self.union_find.find(to),
            },
            Type::Const(type_var) => Type::Const(self.union_find.find(type_var)),
            Type::Tuple(type_vars) => {
                Type::Tuple(type_vars.iter().map(|v| self.union_find.find(v)).collect())
            }
        }
    }

    fn union(&mut self, va: &TypeVar, vb: &TypeVar) -> Result<TypeVar, TypeError> {
        let a = self.get(va);
        let b = self.get(vb);
        let mut swap = false;
        match &a {
            Type::Call {
                left: left_a,
                right: right_a,
            } => {
                let Type::Call {
                    left: left_b,
                    right: right_b,
                } = &b
                else {
                    Err(TypeError::MismatchedTypeError {
                        except: a,
                        received: b,
                    })?
                };
                self.union(left_a, left_b)?;
                self.union(right_a, right_b)?;
            }
            Type::Transfer {
                from: from_a,
                to: to_a,
            } => {
                let Type::Transfer {
                    from: from_b,
                    to: to_b,
                } = &b
                else {
                    Err(TypeError::MismatchedTypeError {
                        except: a,
                        received: b,
                    })?
                };
                self.union(from_a, from_b)?;
                self.union(to_a, to_b)?;
            }
            Type::Const(type_var_a) => {
                let Type::Const(type_var_b) = &b else {
                    Err(TypeError::MismatchedTypeError {
                        except: a,
                        received: b,
                    })?
                };
                if std::mem::discriminant(&self.union_find.find(type_var_a))
                    != std::mem::discriminant(&self.union_find.find(type_var_b))
                    && !matches!(type_var_a, TypeVar::TypeVar(_))
                    && !matches!(type_var_b, TypeVar::TypeVar(_))
                {
                    return Err(TypeError::MismatchedTypeError {
                        except: a,
                        received: b,
                    });
                }
                if matches!(type_var_b, TypeVar::TypeVar(_)) {
                    swap = true
                }
            }
            Type::Tuple(type_vars_a) => {
                let Type::Tuple(type_vars_b) = &b else {
                    Err(TypeError::MismatchedTypeError {
                        except: a,
                        received: b,
                    })?
                };
                if type_vars_a.len() != type_vars_b.len() {
                    return Err(TypeError::MismatchedTypeError {
                        except: a,
                        received: b,
                    });
                }
                for i in 0..type_vars_a.len() {
                    self.union(&type_vars_a[i], &type_vars_b[i])?;
                }
            }
        }
        let v = if swap {
            self.union_find.union(vb, va)
        } else {
            self.union_find.union(va, vb)
        };
        // println!("{va:?}, {vb:?} -> {v:?}");
        let t = self.get(&v);
        self.type_vars.rm(va);
        self.type_vars.rm(vb);
        self.type_vars.bind(v.clone(), t);
        Ok(v)
    }
}

fn build(code: &AST, type_vars: &mut TypeTree) -> Result<TypedAST, TypeError> {
    let AST { span, value } = code;
    let span = span.clone();
    Ok(match value {
        ASTKind::Const(sct) => TypedAST {
            kind: TypedASTKind::Const(sct.clone()),
            typing: match sct {
                Const::Int(_) => TypeVar::Int,
                Const::Float(_) => TypeVar::Float,
                Const::Str(_) => TypeVar::Str,
                Const::Name(name) => get_builtin_type(name, type_vars),
            },
            span,
        },
        ASTKind::Call { left, right } => {
            let left = build(left, type_vars)?;
            let right = build(right, type_vars)?;
            TypedAST {
                typing: type_vars.link(&Type::Call {
                    left: left.typing.clone(),
                    right: right.typing.clone(),
                })?,
                kind: TypedASTKind::Call {
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            }
        }
        ASTKind::Tuple(asts) => {
            let tuple: Vec<TypedAST> = asts
                .iter()
                .map(|e| build(e, type_vars))
                .collect::<Result<_, _>>()?;
            TypedAST {
                typing: type_vars.link(&Type::Tuple(
                    tuple.iter().map(|e| e.typing.clone()).collect(),
                ))?,
                kind: TypedASTKind::Tuple(tuple),
                span,
            }
        }
    })
}

fn clean(code: &mut TypedAST, type_vars: &mut TypeTree) {
    let TypedAST {
        typing,
        kind,
        span: _span,
    } = code;
    match kind {
        TypedASTKind::Call { left, right } => {
            clean(left, type_vars);
            clean(right, type_vars);
        }
        TypedASTKind::Const(_) => (),
        TypedASTKind::Tuple(typed_asts) => {
            typed_asts.iter_mut().for_each(|t| clean(t, type_vars));
        }
    }
    *typing = type_vars.union_find.find(typing);
}

use std::rc::Rc;

use super::types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VarUsage {
    Move,
    Copy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExprDataInner<Name> {
    Unit,

    Var { usage: VarUsage, index: usize },

    Abs {
        type_params: Rc<Vec<TypeParam<Name>>>,
        arg_name: Name,
        arg_type: Type<Name>,
        body: ExprData<Name>,
    },

    App {
        callee: ExprData<Name>,
        type_params: Rc<Vec<Type<Name>>>,
        arg: ExprData<Name>,
    },

    Pair {
        left: ExprData<Name>,
        right: ExprData<Name>,
    },

    Let {
        names: Rc<Vec<Name>>,
        val: ExprData<Name>,
        body: ExprData<Name>,
    },

    LetExists {
        type_names: Rc<Vec<Name>>,
        val_name: Name,
        val: ExprData<Name>,
        body: ExprData<Name>,
    },

    MakeExists {
        params: Rc<Vec<(Name, Type<Name>)>>,
        type_body: Type<Name>,
        body: ExprData<Name>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExprData<Name> {
    inner: Rc<ExprDataInner<Name>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExprContent<Name> {
    Unit { free_vars: usize, free_types: usize },

    Var {
        usage: VarUsage,
        free_vars: usize,
        free_types: usize,
        index: usize,
    },

    Abs {
        type_params: Rc<Vec<TypeParam<Name>>>,
        arg_name: Name,
        arg_type: Type<Name>,
        body: Expr<Name>,
    },

    App {
        callee: Expr<Name>,
        type_params: Rc<Vec<Type<Name>>>,
        arg: Expr<Name>,
    },

    Pair { left: Expr<Name>, right: Expr<Name> },

    Let {
        names: Rc<Vec<Name>>,
        val: Expr<Name>,
        body: Expr<Name>,
    },

    LetExists {
        type_names: Rc<Vec<Name>>,
        val_name: Name,
        val: Expr<Name>,
        body: Expr<Name>,
    },

    MakeExists {
        params: Rc<Vec<(Name, Type<Name>)>>,
        type_body: Type<Name>,
        body: Expr<Name>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expr<Name> {
    free_vars: usize,
    free_types: usize,
    data: ExprData<Name>,
}

impl<Name: Clone> Expr<Name> {
    pub fn free_vars(&self) -> usize {
        self.free_vars
    }

    pub fn free_types(&self) -> usize {
        self.free_types
    }

    pub fn from_content(content: ExprContent<Name>) -> Self {
        match content {
            ExprContent::Unit {
                free_vars,
                free_types,
            } => {
                Expr {
                    free_vars,
                    free_types,
                    data: ExprData { inner: Rc::new(ExprDataInner::Unit) },
                }
            }

            ExprContent::Var {
                usage,
                free_vars,
                free_types,
                index,
            } => {
                assert!(index < free_vars);
                Expr {
                    free_vars,
                    free_types,
                    data: ExprData { inner: Rc::new(ExprDataInner::Var { usage, index }) },
                }
            }

            ExprContent::Abs {
                type_params,
                arg_name,
                arg_type,
                body,
            } => {
                assert_eq!(
                    arg_type.free(),
                    body.free_types,
                    "Free type variables do not match",
                );

                assert!(
                    type_params.len() <= body.free_types,
                    "Must have at least {} free type variables",
                );

                assert!(
                    1 <= body.free_vars,
                    "Must have at least one free term variable",
                );

                Expr {
                    free_vars: body.free_vars - 1,
                    free_types: body.free_types - type_params.len(),
                    data: ExprData {
                        inner: Rc::new(ExprDataInner::Abs {
                            type_params,
                            arg_name,
                            arg_type,
                            body: body.data,
                        }),
                    },
                }
            }

            ExprContent::App {
                callee,
                type_params,
                arg,
            } => {
                Expr {
                    free_vars: arg.free_vars,
                    free_types: arg.free_types,
                    data: ExprData {
                        inner: Rc::new(ExprDataInner::App {
                            callee: callee.data,
                            type_params,
                            arg: arg.data,
                        }),
                    },
                }
            }

            ExprContent::Pair { left, right } => {
                assert_eq!(
                    left.free_vars,
                    right.free_vars,
                    "Free term variables do not match",
                );

                assert_eq!(
                    left.free_types,
                    right.free_types,
                    "Free type variables do not match",
                );

                Expr {
                    free_vars: left.free_vars,
                    free_types: left.free_types,
                    data: ExprData {
                        inner: Rc::new(ExprDataInner::Pair {
                            left: left.data,
                            right: right.data,
                        }),
                    },
                }
            }

            ExprContent::Let { names, val, body } => {
                assert!(names.len() > 0, "Must bind at least one variable");

                assert_eq!(
                    val.free_types,
                    body.free_types,
                    "Free type variables do not match",
                );

                assert_eq!(
                    val.free_vars + names.len(),
                    body.free_vars,
                    "Free term variables do not match",
                );

                Expr {
                    free_vars: val.free_vars,
                    free_types: val.free_types,
                    data: ExprData {
                        inner: Rc::new(ExprDataInner::Let {
                            names,
                            val: val.data,
                            body: body.data,
                        }),
                    },
                }
            }

            ExprContent::LetExists {
                type_names,
                val_name,
                val,
                body,
            } => {
                assert!(type_names.len() > 0, "Must bind at least one type");

                assert_eq!(
                    val.free_types + type_names.len(),
                    body.free_types,
                    "Free type variables do not match",
                );

                assert_eq!(
                    val.free_vars + 1,
                    body.free_vars,
                    "Free term variables do not match",
                );

                Expr {
                    free_vars: val.free_vars,
                    free_types: val.free_types,
                    data: ExprData {
                        inner: Rc::new(ExprDataInner::LetExists {
                            type_names,
                            val_name,
                            val: val.data,
                            body: body.data,
                        }),
                    },
                }
            }

            ExprContent::MakeExists {
                params,
                type_body,
                body,
            } => {
                assert!(params.len() > 0, "Must bind at least one type");

                assert_eq!(
                    body.free_types + params.len(),
                    type_body.free(),
                    "Free type variables do not match",
                );

                for &(_, ref param) in params.iter() {
                    assert_eq!(
                        param.free(),
                        body.free_types,
                        "Free type variables do not match",
                    );
                }

                Expr {
                    free_vars: body.free_vars,
                    free_types: body.free_types,
                    data: ExprData {
                        inner: Rc::new(ExprDataInner::MakeExists {
                            params,
                            type_body,
                            body: body.data,
                        }),
                    },
                }
            }
        }
    }

    pub fn to_content(&self) -> ExprContent<Name> {
        match &*self.data.inner {
            &ExprDataInner::Unit => {
                ExprContent::Unit {
                    free_vars: self.free_vars,
                    free_types: self.free_types,
                }
            }

            &ExprDataInner::Var { usage, index } => {
                ExprContent::Var {
                    free_vars: self.free_vars,
                    free_types: self.free_types,
                    usage,
                    index,
                }
            }

            &ExprDataInner::Abs {
                ref type_params,
                ref arg_name,
                ref arg_type,
                ref body,
            } => {
                ExprContent::Abs {
                    type_params: type_params.clone(),
                    arg_name: arg_name.clone(),
                    arg_type: arg_type.clone(),
                    body: Expr {
                        free_types: self.free_types + type_params.len(),
                        free_vars: self.free_vars + 1,
                        data: body.clone(),
                    },
                }
            }

            &ExprDataInner::App {
                ref callee,
                ref type_params,
                ref arg,
            } => {
                ExprContent::App {
                    callee: Expr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: callee.clone(),
                    },
                    type_params: type_params.clone(),
                    arg: Expr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: arg.clone(),
                    },
                }
            }

            &ExprDataInner::Pair {
                ref left,
                ref right,
            } => {
                ExprContent::Pair {
                    left: Expr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: left.clone(),
                    },
                    right: Expr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: right.clone(),
                    },
                }
            }

            &ExprDataInner::Let {
                ref names,
                ref val,
                ref body,
            } => {
                ExprContent::Let {
                    names: names.clone(),
                    val: Expr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: val.clone(),
                    },
                    body: Expr {
                        free_types: self.free_types,
                        free_vars: self.free_vars + names.len(),
                        data: body.clone(),
                    },
                }
            }

            &ExprDataInner::LetExists {
                ref type_names,
                ref val_name,
                ref val,
                ref body,
            } => {
                ExprContent::LetExists {
                    type_names: type_names.clone(),
                    val_name: val_name.clone(),
                    val: Expr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: val.clone(),
                    },
                    body: Expr {
                        free_types: self.free_types + type_names.len(),
                        free_vars: self.free_vars + 1,
                        data: body.clone(),
                    },
                }
            }

            &ExprDataInner::MakeExists {
                ref params,
                ref type_body,
                ref body,
            } => {
                ExprContent::MakeExists {
                    params: params.clone(),
                    type_body: type_body.clone(),
                    body: Expr {
                        free_vars: self.free_vars,
                        free_types: self.free_types,
                        data: body.clone(),
                    },
                }
            }
        }
    }
}

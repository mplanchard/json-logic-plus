//! A namespace is a set of defined variables and functions that may be
//! made available in an execution context

use crate::op::func::Function;

use std::collections::HashMap;

struct Namespace<'a> {
    funcs: HashMap<&'a str, Function>,
}

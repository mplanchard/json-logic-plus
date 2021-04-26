//! Functions

use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::op::NumParams;
use crate::{error::Error, Parser};

/// A (potentially user-defined) function
///
/// The simplest function definition looks like:
///
/// ```jsonc
/// {
///     "def": [        // function definition operator
///         "is_even",  // function name
///         [a],        // function params
///         // function expression
///         {
///             "===": [
///                 {"%": [{"param": "a"}, 2]},
///                 0
///             ]
///         }
///     ]
/// }
/// ```
///
/// Once defined, the above function can be used like:
///
/// ```jsonc
/// {"is_even": [5]}  // false
/// {"is_even": [2]}  // true
/// ```
///
/// Function expressions may use any of the standard operators or any
/// previously defined functions.
///
pub struct Function<'a> {
    name: &'a str,
    params: Vec<&'a Value>,
    expression: &'a Value,
}
impl<'a> Function<'a> {
    const OPERATOR: &'static str = "defn";
}
impl<'a> Parser<'a> for Function<'a> {
    fn from_value(value: &'a Value) -> Result<Option<Self>, Error> {
        let tmp = match value {
            // Check for an object
            Value::Object(obj) => Some(obj),
            _ => None,
        }
        .and_then(|obj| match obj.len() {
            // Check for an object with only one key
            1 => Some(obj),
            _ => None,
        })
        .and_then(
            // See if the object's key is the Function operator, returning
            // its values if so
            |obj| obj.get(Function::OPERATOR),
        )
        .map(
            // Validate parameters and construct a function object
            |vals| match vals {
                // Check that the single key's value is an array of args
                Value::Array(vals) => Ok(vals),
                _ => Err(Error::InvalidOperation {
                    key: Function::OPERATOR.into(),
                    reason: format!(
                        "Argument for {} must be a length 3 array of the function \
                         name, an array of parameter names, and the function \
                         expression",
                        Function::OPERATOR
                    ),
                })
                .and_then(|vals: &Vec<Value>| match vals.len() {
                    // Validate that there are three args
                    3 => Ok(vals),
                    _ => Err(Error::WrongArgumentCount {
                        expected: NumParams::Exactly(3),
                        actual: vals.len(),
                    }),
                })
                .and_then(|vals| {
                    // Validate the arg types
                    let (name, params, exp) = match vals[0] {
                        Value::String(name) => Ok(name.as_str()),
                        _ => Err(Error::InvalidArgument {
                            value: vals[0].clone(),
                            operation: Function::OPERATOR,
                            reason: "Function name must be a string".into()
                        })
                    }.and_then(|name| match vals[1] {
                        Value::Array(params) => {
                            // check that each parameter is a string
                            params.iter().try_for_each(|param| match param {
                                Value::String(p) => Ok(()),
                                _ => Err(Error::InvalidArgument {
                                    value: vals[1].clone(),
                                    operation: Function::OPERATOR,
                                    reason: format!(
                                        "Function parameters must be an array of \
                                         parameter names as strings. Received parameter \
                                         {}, which is not a string",
                                        param
                                    )
                                })
                            })?;
                            Ok((name, &params, vals[2]))
                        }
                        _ => Err(Error::InvalidArgument {
                            value: vals[1].clone(),
                            operation: Function::OPERATOR,
                            reason: "Function parameters must be an array of parameter names".into()
                        })
                    })?;

                }),
            },
        )
        .transpose();
    }

    fn evaluate(&self, data: &'a Value) -> Result<crate::value::Evaluated, Error> {
        todo!()
    }
}
impl From<Function<'_>> for Value {
    fn from(func: Function) -> Self {
        let mut rv = Map::with_capacity(1);
        rv.insert(
            Function::OPERATOR.into(),
            Value::Array(vec![
                func.name.into(),
                Value::Array(func.params.into_iter().map(|i| i.clone()).collect()),
                func.expression.clone(),
            ]),
        );
        Value::Object(rv)
    }
}

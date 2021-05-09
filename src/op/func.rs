//! FUNctions

use std::convert::{TryFrom, TryInto};

use serde_json::{Map, Value};

use crate::{error::Error, op::CommonOperator, op::NumParams, Parser};

/// A (potentially user-defined) function
///
/// The simplest function definition looks like:
///
/// ```jsonc
/// {
///     "defn": [        // function definition operator
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
#[derive(Clone, Debug, PartialEq)]
pub struct Function<'a> {
    name: Identifier,
    params: Vec<Identifier>,
    expression: &'a Value,
    num_params: NumParams,
}
impl Function<'_> {
    const OPERATOR: &'static str = "defn";

    /// Return Some with the passed value if it is a function, or None otherwise.
    pub fn filter_value(value: &Value) -> Option<&Value> {
        Self::get_in_params(value).and(Some(value))
    }

    /// Return parameters from a Value if it's a function, otherwise None
    fn get_in_params(value: &Value) -> Option<&Value> {
        match value {
            Value::Object(o) => Some(o),
            _ => None,
        }
        .and_then(|obj| match obj.len() {
            // Objects with more than one keys can't be function definitions, even
            // if they contain a "defn" key.
            1 => Some(obj),
            _ => None,
        })
        .and_then(|o| o.get(Self::OPERATOR))
    }

    /// Convert a value into a parameter list
    fn to_parameters(value: &Value) -> Result<Vec<Identifier>, Error> {
        match value {
            Value::Array(params) => Ok(params),
            _ => Err(Error::InvalidArgument {
                value: value.clone(),
                operation: Self::OPERATOR,
                reason: "Parameter list must be an array".into(),
            }),
        }
        .and_then(|params| {
            params
                .iter()
                // Construct an iterable of Result<Identifier, Error>
                .map(|param| {
                    param.try_into().map_err(|e| Error::InvalidArgument {
                        value: value.clone(),
                        operation: Self::OPERATOR,
                        reason: format!(
                            "Could not parse parameter {} due to: {}",
                            param, e
                        ),
                    })
                })
                // Collect it into a Result<Vec<Identifier>, Error>
                .collect()
        })
    }
}
impl<'a> Function<'a> {
    pub fn new(
        name: Identifier,
        params: Vec<Identifier>,
        expression: &'a Value,
        num_params: NumParams,
    ) -> Self {
        Self {
            name,
            params,
            expression,
            num_params,
        }
    }
}
impl CommonOperator for Function<'_> {
    fn param_info(&self) -> &NumParams {
        &self.num_params
    }
}
impl<'a> Parser<'a> for Function<'a> {
    /// Attempt to parse a function from a Value.
    ///
    /// If the Value cannot be interpreted as a function, return Ok(None). If the
    /// Value can be interpreted as a function but is an invalid function expression,
    /// return an error. Otherwise, return Ok(Some(func)).
    fn from_value(value: &'a Value) -> Result<Option<Self>, Error> {
        struct InParams<'b> {
            name: &'b Value,
            params: &'b Value,
            expr: &'b Value,
        }

        Self::get_in_params(value)
            .map(|vals| {
                match vals {
                    Value::Array(v) => Ok(v),
                    _ => Err(Error::InvalidArgument {
                        value: vals.clone(),
                        operation: Self::OPERATOR,
                        reason: "Argument to 'defn' must be an array".into(),
                    }),
                }
                .and_then(|vals| match vals.len() {
                    3 => Ok(vals),
                    _ => Err(Error::WrongArgumentCount {
                        expected: NumParams::Exactly(3),
                        actual: vals.len(),
                    }),
                })
                .map(|vals| InParams {
                    name: &vals[0],
                    params: &vals[1],
                    expr: &vals[2],
                })
                .and_then(|in_params| {
                    let params = Function::to_parameters(in_params.params)?;
                    let num_params = params.len();
                    // For now prevent functions returning functions.
                    let expr = Function::filter_value(in_params.expr)
                        .map(|expr| {
                            Err(Error::InvalidArgument {
                                value: expr.clone(),
                                operation: Function::OPERATOR,
                                reason: "A function's body may not be a expression"
                                    .into(),
                            })
                        })
                        .unwrap_or(Ok(&in_params.expr))?;
                    Ok(Function::new(
                        in_params.name.try_into().map_err(|e: Error| {
                            Error::invalid_argument(
                                in_params.name.clone(),
                                Self::OPERATOR,
                                e.to_string(),
                            )
                        })?,
                        params,
                        expr,
                        NumParams::Exactly(num_params),
                    ))
                })
            })
            .transpose()
    }

    fn evaluate(&self, data: &'a Value) -> Result<crate::value::Evaluated, Error> {
        todo!()
    }
}
impl From<Function<'_>> for Value {
    fn from(func: Function<'_>) -> Self {
        let params =
            Value::Array(func.params.iter().map(|param| param.into()).collect());
        let expression = func.expression.clone();
        let values = Value::Array(vec![params, expression]);
        let mut val = Map::new();
        val.insert(Function::OPERATOR.into(), values);
        Value::Object(val)
    }
}

/// An Identifier for a function or variable.
///
/// Identifiers contain one or more non-whitespace unicode characters. Note that
/// these are characters, not code points, so you can get away with silly things
/// like é being a valid identifier (since it is two characters but one code
/// point). However, this should not be considered part of the specification,
/// which may eventually evolve to use code points if needed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identifier {
    value: String,
}

impl Identifier {
    /// Construct a new identifier.
    fn new<S: Into<String>>(value: S) -> Result<Self, Error> {
        let value: String = value.into();
        if value.len() < 1 {
            return Err(Error::InvalidIdentifier(value));
        }
        value.chars().try_for_each(|c| match c.is_whitespace() {
            true => Err(Error::InvalidIdentifier(value.clone())),
            false => Ok(()),
        })?;
        Ok(Self { value })
    }

    /// Return a reference to the identifier as a string slice.
    fn value(&self) -> &str {
        &self.value
    }
}
impl TryFrom<&Value> for Identifier {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(i) => Ok(i),
            _ => Err(Error::InvalidIdentifier(value.to_string())),
        }
        .and_then(|i| Identifier::new(i))
    }
}
impl From<Identifier> for Value {
    fn from(ident: Identifier) -> Self {
        (&ident).into()
    }
}
impl From<&Identifier> for Value {
    fn from(ident: &Identifier) -> Self {
        Value::String(ident.value.clone())
    }
}

#[cfg(test)]
mod test_function {
    use std::mem;

    use super::*;
    use serde_json::json;

    use crate::test_common::json_exp;

    #[derive(Debug)]
    struct FunctionParsingCase<'a> {
        value: Value,
        function: Option<Function<'a>>,
        err: Option<Error>,
    }

    impl<'a> FunctionParsingCase<'a> {
        fn new(
            value: Value,
            function: Option<Function<'a>>,
            err: Option<Error>,
        ) -> Self {
            Self {
                value,
                function,
                err,
            }
        }
        fn ok(value: Value, function: Function<'a>) -> Self {
            Self::new(value, Some(function), None)
        }
        fn err(value: Value, error: Error) -> Self {
            Self::new(value, None, Some(error))
        }
        fn not_a_func(value: Value) -> Self {
            Self::new(value, None, None)
        }
    }

    /// We only check that errors are the expected enum variants, not their
    /// contents, so make some quick factories for garbage defaults
    impl Error {
        fn default_invalid_argument() -> Self {
            Self::invalid_argument(Value::Null, "foo", "foo")
        }
        fn default_wrong_argument_count() -> Self {
            Self::wrong_argument_count(NumParams::Unary, 2)
        }
    }

    fn function_parsing_cases<'a>() -> Vec<FunctionParsingCase<'a>> {
        vec![
            FunctionParsingCase::not_a_func(json!(12)),
            FunctionParsingCase::not_a_func(json!({"+": [1, 2]})),
            // any object with more than one key cannot be a function expression
            FunctionParsingCase::not_a_func(json!({"defn": [1, 2], "foo": 3})),
            FunctionParsingCase::ok(
                json!({"defn": ["foo", ["a", "b"], &*json_exp::ADD_TWO]}),
                Function::new(
                    Identifier::new("foo").unwrap(),
                    vec![Identifier::new("a").unwrap(), Identifier::new("b").unwrap()],
                    &*json_exp::ADD_TWO,
                    NumParams::Exactly(2),
                ),
            ),
            // Invalid b/c wrong argument type
            FunctionParsingCase::err(
                json!({"defn": "foo"}),
                Error::default_invalid_argument(),
            ),
            // Invalid b/c wrong number of args
            FunctionParsingCase::err(
                json!({"defn": []}),
                Error::default_wrong_argument_count(),
            ),
            // Invalid because the params are not strings
            FunctionParsingCase::err(
                json!({"defn": ["a", [12, 13], 15]}),
                Error::default_invalid_argument(),
            ),
            // Invalid because the param is an invalid identifier
            FunctionParsingCase::err(
                json!({"defn": ["a", [""], 15]}),
                Error::default_invalid_argument(),
            ),
            // Invalid b/c the name is an invalid identifier
            FunctionParsingCase::err(
                json!({"defn": ["", ["a"], 15]}),
                Error::default_invalid_argument(),
            ),
            // Invalid b/c it returns a function definition
            FunctionParsingCase::err(
                json!({"defn": ["a", ["a"], {"defn": ["a", [], 12]}]}),
                Error::default_invalid_argument(),
            ),
        ]
    }

    #[test]
    fn parse_function() {
        function_parsing_cases().iter().for_each(|case| {
            let parsed = Function::from_value(&case.value);

            let debug_msg = format!("case: {:?}, res: {:?}", &case, &parsed);
            if let Some(exp_err) = &case.err {
                let err_msg = format!("Got {:?}", &parsed);
                let act_err = parsed.expect_err(&err_msg);
                assert_eq!(
                    mem::discriminant(exp_err),
                    mem::discriminant(&act_err),
                    "{}",
                    &debug_msg
                );
            } else {
                let err_msg = format!("Got {:?}", &parsed);
                let act_func = parsed.expect(&err_msg);
                if let Some(exp_func) = &case.function {
                    assert_eq!(
                        exp_func,
                        &act_func.expect("Func was none"),
                        "{}",
                        &debug_msg
                    );
                } else {
                    assert!(act_func.is_none(), "{:?}", case)
                }
            }
        })
    }
}

#[cfg(test)]
mod test_identifier {

    use super::*;

    struct IdentifierCase {
        identifier: String,
        is_valid: bool,
    }

    impl IdentifierCase {
        fn new<T: Into<String>>(identifier: T, is_valid: bool) -> Self {
            Self {
                identifier: identifier.into(),
                is_valid,
            }
        }
        fn ok<T: Into<String>>(identifier: T) -> Self {
            Self::new(identifier.into(), true)
        }
        fn err<T: Into<String>>(identifier: T) -> Self {
            Self::new(identifier.into(), false)
        }
    }

    fn identifier_cases() -> Vec<IdentifierCase> {
        vec![
            IdentifierCase::ok("foo"),
            IdentifierCase::ok("ba"),
            IdentifierCase::ok("ok_thing"),
            IdentifierCase::ok("ok-thing"),
            IdentifierCase::ok("okThing"),
            IdentifierCase::ok("okay_ûnicodé"),
            // too short
            IdentifierCase::err(""),
            // whitespace
            IdentifierCase::err("a b"),
            IdentifierCase::err(" ab"),
            IdentifierCase::err("ab\t"),
            IdentifierCase::err("ab\n"),
        ]
    }

    #[test]
    fn test_identifier() {
        identifier_cases().iter().for_each(|case| {
            let res = Identifier::new(case.identifier.clone());
            match case.is_valid {
                true => {
                    res.unwrap();
                }
                false => {
                    res.unwrap_err();
                }
            }
        })
    }
}

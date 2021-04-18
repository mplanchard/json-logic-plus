//! FUnctions

use serde_json::Value;

use crate::{error::Error, op::CommonOperator, op::NumParams};

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
#[derive(Debug)]
pub struct Function {
    name: Identifier,
    params: Vec<Identifier>,
    expression: Value,
    num_params: NumParams,
}
impl CommonOperator for Function {
    fn param_info(&self) -> &NumParams {
        &self.num_params
    }
}

/// An Identifier for a function or variable.
///
/// Identifiers contain two or more non-whitespace unicode characters. Note that
/// these are characters, not code points, so you can get away with silly things
/// like é being a valid identifier (since it is two characters but one code
/// point). However, this should not be considered part of the specification,
/// which may eventually evolve to use code points if needed.
#[derive(Clone, Debug)]
pub(crate) struct Identifier {
    value: String,
}

impl Identifier {
    /// Construct a new identifier.
    fn new(value: String) -> Result<Self, Error> {
        if value.len() < 2 {
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
            IdentifierCase::err("a"),
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

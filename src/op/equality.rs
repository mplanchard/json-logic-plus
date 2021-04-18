//! Equality operators

use crate::error::Error;
use serde_json::Value;

/// Compare two values for equality.
///
/// Returns true if the item are equal.
pub fn equal(items: &Vec<&Value>) -> Result<Value, Error> {
    // We assume the number of parameters has already been checked.
    let first = items[0];
    let second = items[1];

    Ok(Value::Bool(first == second))
}

/// Compare any number of items for inequality.
///
/// Returns false if the items are equal
pub fn not_equal(items: &Vec<&Value>) -> Result<Value, Error> {
    // We assume the number of parameters has already been checked.
    let first = items[0];
    let second = items[1];

    Ok(Value::Bool(first != second))
}

#[cfg(test)]
pub mod test_equality_operators {
    use super::*;
    use serde_json::json;

    pub fn eq_cases() -> Vec<(Value, Value, Value)> {
        vec![
            (json!(true), json!(true), json!(true)),
            (json!(true), json!(false), json!(false)),
            (json!(true), json!(1), json!(false)),
            (json!(true), json!([]), json!(false)),
            (json!(false), json!(null), json!(false)),
            (json!([]), json!([]), json!(true)),
            (json!([1, 2]), json!([1, 2]), json!(true)),
            (json!([2, 1]), json!([1, 2]), json!(false)),
            (json!({}), json!({}), json!(true)),
            (
                json!({"a": 1, "b": 2}),
                json!({"b": 2, "a": 1}),
                json!(true),
            ),
            (
                json!({"a": 1, "b": [1, 2, {"c": 1}]}),
                json!({"b": [1, 2, {"c": 1}], "a": 1}),
                json!(true),
            ),
            (
                json!({"a": 1, "b": 2}),
                json!({"b": 2, "a": 3}),
                json!(false),
            ),
        ]
    }

    #[test]
    fn test_equal() {
        eq_cases().iter().for_each(|(first, second, exp)| {
            assert_eq!(
                &equal(&vec![first, second]).unwrap(),
                exp,
                "Comparing {:?} to {:?} failed",
                first,
                second
            )
        })
    }

    #[test]
    fn test_not_equal() {
        eq_cases().iter().for_each(|(first, second, exp)| {
            if let Value::Bool(exp) = exp {
                assert_eq!(
                    not_equal(&vec![first, second]).unwrap(),
                    Value::Bool(!exp),
                    "Comparing {:?} to {:?} failed",
                    first,
                    second
                )
            } else {
                assert!(false)
            }
        })
    }
}

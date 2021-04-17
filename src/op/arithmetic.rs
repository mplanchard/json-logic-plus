//! Numeric Operations

use serde_json::{Number, Value};

use crate::error::Error;
use crate::js_op;
use crate::value::to_number_value;

enum JsonNumber {
    U64(u64),
    I64(i64),
    F64(f64),
}
impl JsonNumber {
    fn from_number(n: &Number) -> Option<JsonNumber> {
        n.as_u64()
            .map(JsonNumber::U64)
            .or(n.as_i64().map(JsonNumber::I64))
            .or(n.as_f64().map(JsonNumber::F64))
    }
}

trait CheckedAdd: Sized {
    /// Add two numbers, returning None if the addition overflows
    fn checked_add(&self, v: &Self) -> Option<Self>;
}
impl CheckedAdd for Number {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        // We want to preserve "integerness" in the resulting value. We could
        // just convert everything to f64 and add it together, but then we'd be
        // returning like 5.0 when someone says 2 + 3. So, we're going to try to
        // retain the possible integer types (u64 and i64) if we can! If self is
        // either of those types, we try to interpret second as an integer as
        // well, and if so, to add them together. In the case of overflow, we
        // try to fall back to floating point addition, since an f64 can hold
        // much larger numbers. If either value is an f64, we just add them
        // together as floats.
        let first = JsonNumber::from_number(self);
        let second = JsonNumber::from_number(v);

        match (first, second) {
            (None, None) | (None, _) | (_, None) => None,
            (Some(JsonNumber::U64(first)), Some(JsonNumber::U64(second))) => {
                first
                    .checked_add(second)
                    .map(Number::from)
                    // if checked_add overflows for u64, we can probably fit
                    // it into an f64. Infinity and NaN will still return None
                    .or(Number::from_f64(first as f64 + second as f64))
            }
            (Some(JsonNumber::U64(first)), Some(JsonNumber::I64(second))) => {
                // Determine whether to add or subtract second
                match second {
                    second if second < 0 => {
                        second
                            // convert to a positive number
                            .checked_abs()
                            // this is a safe conversion
                            .map(|i| i as u64)
                            // Now we'll subtract the second from the first
                            .and_then(|second| {
                                // second is a u64 at this point
                                first.checked_sub(second).map(Number::from).or(
                                    // if that returned None, it's because
                                    // the subtraction went below 0. If that
                                    // is the case, we know first must be
                                    // lesss than the max i64, so this is a
                                    // safe conversion
                                    (first as i64)
                                        // and this should get as back
                                        // to a negative.
                                        .checked_sub(second as i64)
                                        .map(Number::from),
                                )
                            })
                            // Fall back to f64 if all else fails
                            .or(Number::from_f64(first as f64 - second as f64))
                    }
                    _ => first
                        .checked_add(second as u64)
                        .map(Number::from)
                        // Fall back to f64 if we can
                        .or(Number::from_f64(first as f64 + second as f64)),
                }
            }
            (Some(JsonNumber::I64(first)), Some(JsonNumber::I64(second))) => {
                // Determine whether to add or subtract second
                match second {
                    second if second < 0 => {
                        // convert to a positive number
                        second
                            .checked_abs()
                            .and_then(|second| first.checked_sub(second))
                            .map(Number::from)
                            // fall back to f64 to handle a larger range of values
                            .or(Number::from_f64(first as f64 - second as f64))
                    }
                    _ => first
                        .checked_add(second)
                        .map(Number::from)
                        .or(Number::from_f64(first as f64 + second as f64)),
                }
            }
            (Some(JsonNumber::I64(first)), Some(JsonNumber::U64(second))) => {
                // Reuse the logic above by flipping the arguments and calling again.
                Number::from(second).checked_add(&first.into())
            }
            // In any other case, one of our values is a float, so we'll try
            // to get both values as floats and add them together. This will
            // return None if the result is Infinity or NaN.
            _ => self.as_f64().and_then(|first| {
                v.as_f64()
                    .and_then(|second| Number::from_f64(first + second))
            }),
        }
    }
}

fn compare<F>(func: F, items: &Vec<&Value>) -> Result<Value, Error>
where
    F: Fn(&Value, &Value) -> bool,
{
    if items.len() == 2 {
        Ok(Value::Bool(func(items[0], items[1])))
    } else {
        Ok(Value::Bool(
            func(items[0], items[1]) && func(items[1], items[2]),
        ))
    }
}

/// Do < for either 2 or 3 values
pub fn lt(items: &Vec<&Value>) -> Result<Value, Error> {
    compare(js_op::abstract_lt, items)
}

/// Do <= for either 2 or 3 values
pub fn lte(items: &Vec<&Value>) -> Result<Value, Error> {
    compare(js_op::abstract_lte, items)
}

/// Do > for either 2 or 3 values
pub fn gt(items: &Vec<&Value>) -> Result<Value, Error> {
    compare(js_op::abstract_gt, items)
}

/// Do >= for either 2 or 3 values
pub fn gte(items: &Vec<&Value>) -> Result<Value, Error> {
    compare(js_op::abstract_gte, items)
}

/// Perform subtraction or convert a number to a negative
pub fn minus(items: &Vec<&Value>) -> Result<Value, Error> {
    let value = if items.len() == 1 {
        js_op::to_negative(items[0])?
    } else {
        js_op::abstract_minus(items[0], items[1])?
    };
    to_number_value(value)
}

/// Perform addition on two numbers.
///
/// This is a non-JS-compliant operation, which is to say it does no implicit
/// type conversion. The only acceptable arguments are numbers.
pub fn add(items: &Vec<&Value>) -> Result<Value, Error> {
    let (first, second) = (items[0], items[1]);
    match (first, second) {
        (Value::Number(first), Value::Number(second)) => first
            .checked_add(second)
            .map(Value::Number)
            .ok_or(Error::OverflowBinaryOp(
                "add",
                format!("{}", first),
                format!("{}", second),
            )),
        (Value::Number(_), _) => Err(Error::InvalidArgument {
            value: second.clone(),
            operation: "add",
            reason: "arguments to add must be numbers".into(),
        }),
        (_, _) => Err(Error::InvalidArgument {
            value: first.clone(),
            operation: "add",
            reason: "arguments to add must be numbers".into(),
        }),
    }
}

#[cfg(test)]
pub(crate) mod test_arithmetic {
    use super::*;
    use serde_json::json;

    #[derive(Debug)]
    pub struct TestCase {
        pub items: Value,
        pub exp_err: bool,
        pub res: Option<Value>,
    }
    impl TestCase {
        fn err(items: Value) -> TestCase {
            TestCase {
                items,
                exp_err: true,
                res: None,
            }
        }
        fn ok(items: Value, res: Value) -> TestCase {
            TestCase {
                items,
                exp_err: false,
                res: Some(res),
            }
        }
    }

    pub fn add_cases() -> Vec<TestCase> {
        vec![
            TestCase::ok(json!([1, 2]), json!(3)),
            TestCase::ok(json!([-1, 2]), json!(1)),
            TestCase::ok(json!([2, -1]), json!(1)),
            TestCase::ok(json!([1.0, 2.0]), json!(3.0)),
            TestCase::ok(json!([1, -2]), json!(-1)),
            TestCase::ok(json!([-2, 1]), json!(-1)),
            TestCase::ok(json!([1, 1.0]), json!(2.0)),
            TestCase::ok(json!([1.0, 1]), json!(2.0)),
            // we can handle things over the maximum i64
            TestCase::ok(
                json!([std::i64::MAX as u64, 1]),
                json!(std::i64::MAX as u64 + 1),
            ),
            // going below the minimum i64 transitions us to floats
            TestCase::ok(
                json!([std::i64::MIN, -1]),
                json!(std::i64::MIN as f64 - 1.0),
            ),
            // going over the maximum u64 transitions us to floats
            TestCase::ok(json!([std::u64::MAX, 1]), json!(std::u64::MAX as f64 + 1.0)),
            // float overflow is an error
            TestCase::err(json!([std::f64::MAX, std::f64::MAX])),
            // Passing non-numbers is an error
            TestCase::err(json!([12, "foo"])),
            TestCase::err(json!(["bar", "foo"])),
            TestCase::err(json!([12, true])),
            TestCase::err(json!([12, null])),
        ]
    }

    #[test]
    fn test_add_cases() {
        add_cases().iter().for_each(|case| {
            let items = match &case.items {
                Value::Array(vals) => vals,
                _ => panic!("Invalid case"),
            };
            let res = add(&items.iter().collect());

            match case.exp_err {
                true => {
                    res.unwrap_err();
                }
                false => {
                    assert_eq!(&res.unwrap(), &case.res.clone().unwrap(), "{:?}", case)
                }
            }
        })
    }
}

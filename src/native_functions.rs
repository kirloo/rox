use std::time::{SystemTime, UNIX_EPOCH};

use crate::execution::{EvalError, EvalErrorKind, FuncImpl, NativeFn, Value};

pub fn clock(_: Vec<Value>) -> Result<Value, EvalError> {
    Ok(Value::Number(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64(),
    ))
}

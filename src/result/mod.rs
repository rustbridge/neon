//! Types and traits for working with JavaScript exceptions.

#[cfg(all(feature = "napi-1", feature = "serde"))]
pub use neon_runtime::serde::Error as SerdeError;

use context::Context;
use handle::Handle;
use std::fmt::{Display, Formatter, Result as FmtResult};
use types::Value;

/// An error sentinel type used by `NeonResult` (and `JsResult`) to indicate that the JavaScript engine
/// has entered into a throwing state.
///
/// `Throw` deliberately does not implement `std::error::Error`, because it's generally not a good idea
/// to chain JavaScript exceptions with other kinds of Rust errors, since entering into the throwing
/// state means that the JavaScript engine is unavailable until the exception is handled.
#[derive(Debug)]
pub struct Throw;

impl Display for Throw {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.write_str("JavaScript Error")
    }
}

/// The result of a computation that might send the JS engine into a throwing state.
pub type NeonResult<T> = Result<T, Throw>;

/// The result of a computation that produces a JavaScript value and might send the JS engine into a throwing state.
pub type JsResult<'b, T> = NeonResult<Handle<'b, T>>;

/// An extension trait for `Result` values that can be converted into `JsResult` values by throwing a JavaScript
/// exception in the error case.
pub trait JsResultExt<'a, V: Value> {
    fn or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> JsResult<'a, V>;
}

/// An extension trait for `Result` values that can be converted into `NeonResult` values by throwing a JavaScript
/// exception in the error case.
pub trait ResultExt<T> {
    fn or_throw<'a, C: Context<'a>>(self, cx: &mut C) -> NeonResult<T>;
}

#[cfg(all(feature = "napi-1", feature = "serde"))]
impl<T> ResultExt<T> for Result<T, SerdeError> {
    fn or_throw<'a, C: Context<'a>>(self, cx: &mut C) -> NeonResult<T> {
        let err = match self {
            Ok(v) => return Ok(v),
            Err(err) => err,
        };

        if err.is_exception_pending() {
            Err(Throw)
        } else {
            cx.throw_error(err.to_string())
        }
    }
}

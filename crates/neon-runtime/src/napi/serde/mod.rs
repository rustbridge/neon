//! Serde implementation for converting between Rust and JavaScript data types

mod de;
mod error;
mod js;
mod se;

use serde_crate::{Deserialize, Serialize};

use crate::napi;

pub use self::error::Error;
use self::js::*;

/// Attempts to read a JavaScript value into a Rust data type using the serde::Deserialize implementation
/// # Safety
/// * `env` must point to the JavaScript runtime executing on the current thread
/// * `value` must be a valid JavaScript object associated with the same runtime as `env`
pub unsafe fn from_value<T: ?Sized>(env: napi::Env, value: napi::Value) -> Result<T, Error>
where
    T: Deserialize<'static>,
{
    T::deserialize(de::Deserializer::new(env, value))
}

/// Attempts to write Rust data into a JavaScript value using the serde::Serialize implementation
/// # Safety
/// * The returned `napi::Value` must not outlive the `env` parameter
/// * `env` must point to the JavaScript runtime executing on the current thread
pub unsafe fn to_value<T: ?Sized>(env: napi::Env, value: &T) -> Result<napi::Value, Error>
where
    T: Serialize,
{
    value.serialize(se::Serializer::new(env))
}

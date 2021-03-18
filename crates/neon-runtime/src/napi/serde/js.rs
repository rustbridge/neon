//! N-API wrappers used by serde transcoding
//!
//! In many cases, these functions provide similar functionality to functions
//! available elsewhere in `neon-runtime`. However, keeping serde fully self
//! contained has a few benefits:
//!
//! * Wrappers can be written, altered, combined and otherwise optimized for
//!   providing the most efficient possible serde implementation
//! * All wrappers can be written idiomatically instead of matching the legacy
//!   behavior of `neon-sys`
//! * The serde implementation remains self contained for potential extraction
//!   into a separate crate
//!
//! # Safety
//!
//! _Do not export anything from this file outside of the serde module._
//!
//! Nearly all functions in this crate are `unsafe` and should be treated that
//! way despite being marked safe. However, since the serde implementation is
//! unsafe in it's entirety, these wrappers are marked safe to make the
//! implementation easier to read and review.

use std::mem::MaybeUninit;
use std::ptr;
use std::slice;

use crate::napi;

trait Verify {
    fn verify(self) -> Result<(), napi::Status>;
}

impl Verify for napi::Status {
    fn verify(self) -> Result<(), napi::Status> {
        if self == napi::Status::Ok {
            Ok(())
        } else {
            Err(self)
        }
    }
}

pub(super) fn get_value_bool(env: napi::Env, value: napi::Value) -> Result<bool, napi::Status> {
    let mut out = false;

    unsafe {
        napi::get_value_bool(env, value, &mut out as *mut bool).verify()?;
    };

    Ok(out)
}

pub(super) fn get_value_double(env: napi::Env, value: napi::Value) -> Result<f64, napi::Status> {
    let mut out = 0f64;

    unsafe {
        napi::get_value_double(env, value, &mut out as *mut f64).verify()?;
    };

    Ok(out)
}

pub(super) fn get_string_len(env: napi::Env, value: napi::Value) -> Result<usize, napi::Status> {
    let mut out = 0usize;

    unsafe {
        napi::get_value_string_utf8(env, value, ptr::null_mut(), 0, &mut out as *mut usize)
            .verify()?;
    }

    Ok(out)
}

pub(super) fn get_value_string(env: napi::Env, value: napi::Value) -> Result<String, napi::Status> {
    let mut out = 0usize;
    let string_len = get_string_len(env, value)?;
    let buf_len = string_len + 1;
    let mut buf = Vec::<u8>::with_capacity(buf_len);

    unsafe {
        napi::get_value_string_utf8(
            env,
            value,
            buf.as_mut_ptr().cast(),
            buf_len,
            &mut out as *mut usize,
        )
        .verify()?;

        debug_assert_eq!(out, string_len);
        buf.set_len(string_len);

        Ok(String::from_utf8_unchecked(buf))
    }
}

pub(super) fn get_value_arraybuffer(
    env: napi::Env,
    value: napi::Value,
) -> Result<Vec<u8>, napi::Status> {
    let mut len = 0usize;
    let mut out = MaybeUninit::uninit();

    unsafe {
        napi::get_arraybuffer_info(env, value, out.as_mut_ptr(), &mut len as *mut usize)
            .verify()?;
    };

    let buf = unsafe { slice::from_raw_parts(out.assume_init().cast(), len) };

    Ok(buf.to_vec())
}

pub(super) fn get_array_len(env: napi::Env, value: napi::Value) -> Result<u32, napi::Status> {
    let mut len = 0u32;

    unsafe {
        napi::get_array_length(env, value, &mut len as *mut u32).verify()?;
    };

    Ok(len)
}

pub(super) fn get_array_element(
    env: napi::Env,
    arr: napi::Value,
    i: u32,
) -> Result<napi::Value, napi::Status> {
    let mut out = MaybeUninit::uninit();

    unsafe {
        napi::get_element(env, arr, i, out.as_mut_ptr()).verify()?;
        Ok(out.assume_init())
    }
}

pub(super) fn typeof_value(
    env: napi::Env,
    value: napi::Value,
) -> Result<napi::ValueType, napi::Status> {
    let mut out = MaybeUninit::uninit();

    unsafe {
        napi::typeof_value(env, value, out.as_mut_ptr()).verify()?;
        Ok(out.assume_init())
    }
}

pub(super) fn get_property_names(
    env: napi::Env,
    value: napi::Value,
) -> Result<napi::Value, napi::Status> {
    let mut out = MaybeUninit::uninit();

    unsafe {
        napi::get_property_names(env, value, out.as_mut_ptr()).verify()?;
        Ok(out.assume_init())
    }
}

pub(super) fn get_property(
    env: napi::Env,
    object: napi::Value,
    key: napi::Value,
) -> Result<napi::Value, napi::Status> {
    let mut out = MaybeUninit::uninit();

    unsafe {
        napi::get_property(env, object, key, out.as_mut_ptr()).verify()?;
        Ok(out.assume_init())
    }
}

pub(super) fn get_null(env: napi::Env) -> Result<napi::Value, napi::Status> {
    let mut value = MaybeUninit::uninit();

    unsafe {
        napi::get_null(env, value.as_mut_ptr()).verify()?;
        Ok(value.assume_init())
    }
}

pub(super) fn create_double(
    env: napi::Env,
    v: impl Into<f64>,
) -> Result<napi::Value, napi::Status> {
    let mut value = MaybeUninit::uninit();
    let v = v.into();

    unsafe {
        napi::create_double(env, v, value.as_mut_ptr()).verify()?;
        Ok(value.assume_init())
    }
}

pub(super) fn create_bool(env: napi::Env, v: bool) -> Result<napi::Value, napi::Status> {
    let mut value = MaybeUninit::uninit();

    unsafe {
        napi::get_boolean(env, v, value.as_mut_ptr()).verify()?;
        Ok(value.assume_init())
    }
}

pub(super) fn create_string(
    env: napi::Env,
    v: impl AsRef<str>,
) -> Result<napi::Value, napi::Status> {
    let mut value = MaybeUninit::uninit();
    let v = v.as_ref();

    unsafe {
        napi::create_string_utf8(env, v.as_ptr().cast(), v.len(), value.as_mut_ptr()).verify()?;
        Ok(value.assume_init())
    }
}

pub(super) fn create_object(env: napi::Env) -> Result<napi::Value, napi::Status> {
    let mut value = MaybeUninit::uninit();

    unsafe {
        napi::create_object(env, value.as_mut_ptr()).verify()?;
        Ok(value.assume_init())
    }
}

pub(super) fn create_array_with_length(
    env: napi::Env,
    len: usize,
) -> Result<napi::Value, napi::Status> {
    let mut value = MaybeUninit::uninit();

    unsafe {
        napi::create_array_with_length(env, len, value.as_mut_ptr()).verify()?;
        Ok(value.assume_init())
    }
}

pub(super) fn create_arraybuffer(env: napi::Env, v: &[u8]) -> Result<napi::Value, napi::Status> {
    let mut value = MaybeUninit::uninit();
    let mut data = MaybeUninit::uninit();

    unsafe {
        napi::create_arraybuffer(env, v.len(), data.as_mut_ptr(), value.as_mut_ptr()).verify()?;
    };

    let data = unsafe {
        let data = data.assume_init().cast();

        std::slice::from_raw_parts_mut(data, v.len())
    };

    data.copy_from_slice(v);

    Ok(unsafe { value.assume_init() })
}

pub(super) fn object_set(
    env: napi::Env,
    o: napi::Value,
    k: napi::Value,
    v: napi::Value,
) -> Result<(), napi::Status> {
    unsafe {
        napi::set_property(env, o, k, v).verify()?;
    };

    Ok(())
}

pub(super) fn array_set(
    env: napi::Env,
    arr: napi::Value,
    k: u32,
    v: napi::Value,
) -> Result<(), napi::Status> {
    unsafe {
        napi::set_element(env, arr, k, v).verify()?;
    }

    Ok(())
}

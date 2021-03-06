//! Facilities for working with JS functions.

use std::os::raw::c_void;
use std::ptr::null;

use crate::call::CCallback;
use crate::napi::bindings as napi;
use crate::raw::{Env, Local};

/// Mutates the `out` argument provided to refer to a newly created `v8::Function`. Returns
/// `false` if the value couldn't be created.
pub unsafe fn new(out: &mut Local, env: Env, callback: CCallback) -> bool {
    let status = napi::create_function(
        env,
        null(),
        0,
        Some(std::mem::transmute(callback.static_callback)),
        callback.dynamic_callback,
        out as *mut Local,
    );

    status == napi::Status::Ok
}

pub unsafe fn get_dynamic_callback(_env: Env, data: *mut c_void) -> *mut c_void {
    data
}

pub unsafe fn call(
    out: &mut Local,
    env: Env,
    fun: Local,
    this: Local,
    argc: i32,
    argv: *mut c_void,
) -> bool {
    let status = napi::call_function(
        env,
        this,
        fun,
        argc as usize,
        argv as *const _,
        out as *mut _,
    );

    status == napi::Status::Ok
}

pub unsafe fn construct(
    out: &mut Local,
    env: Env,
    fun: Local,
    argc: i32,
    argv: *mut c_void,
) -> bool {
    let status = napi::new_instance(env, fun, argc as usize, argv as *const _, out as *mut _);

    status == napi::Status::Ok
}

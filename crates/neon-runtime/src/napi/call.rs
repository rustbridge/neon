use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::ptr::null_mut;

use smallvec::{smallvec, SmallVec};

use crate::napi::bindings as napi;
use crate::raw::{Env, FunctionCallbackInfo, Local};

#[repr(C)]
pub struct CCallback {
    pub static_callback: *mut c_void,
    pub dynamic_callback: *mut c_void,
}

impl Default for CCallback {
    fn default() -> Self {
        CCallback {
            static_callback: null_mut(),
            dynamic_callback: null_mut(),
        }
    }
}

pub unsafe fn is_construct(env: Env, info: FunctionCallbackInfo) -> bool {
    let mut target: MaybeUninit<Local> = MaybeUninit::zeroed();

    let status = napi::get_new_target(env, info, target.as_mut_ptr());

    assert_eq!(status, napi::Status::Ok);

    // get_new_target is guaranteed to assign to target, so it's initialized.
    let target: Local = target.assume_init();

    // By the get_new_target contract, target will either be NULL if the current
    // function was called without `new`, or a valid napi_value handle if the current
    // function was called with `new`.
    !target.is_null()
}

pub unsafe fn this(env: Env, info: FunctionCallbackInfo, out: &mut Local) {
    let status = napi::get_cb_info(env, info, null_mut(), null_mut(), out as *mut _, null_mut());
    assert_eq!(status, napi::Status::Ok);
}

/// Mutates the `out` argument provided to refer to the associated data value of the
/// `napi_callback_info`.
pub unsafe fn data(env: Env, info: FunctionCallbackInfo, out: &mut *mut c_void) {
    let mut data = null_mut();
    let status = napi::get_cb_info(
        env,
        info,
        null_mut(),
        null_mut(),
        null_mut(),
        &mut data as *mut _,
    );
    if status == napi::Status::Ok {
        *out = data;
    }
}

/// Gets the number of arguments passed to the function.
pub unsafe fn len(env: Env, info: FunctionCallbackInfo) -> i32 {
    let mut argc = 0usize;
    let status = napi::get_cb_info(
        env,
        info,
        &mut argc as *mut _,
        null_mut(),
        null_mut(),
        null_mut(),
    );
    assert_eq!(status, napi::Status::Ok);
    argc as i32
}

/// Returns the function arguments as a `SmallVec<[Local; 8]>`
pub unsafe fn argv(env: Env, info: FunctionCallbackInfo) -> SmallVec<[Local; 8]> {
    let len = len(env, info);
    let mut args = smallvec![null_mut(); len as usize];
    let mut num_args = args.len();
    let status = napi::get_cb_info(
        env,
        info,
        &mut num_args as *mut _,
        args.as_mut_ptr(),
        null_mut(),
        null_mut(),
    );
    assert_eq!(status, napi::Status::Ok);
    args
}

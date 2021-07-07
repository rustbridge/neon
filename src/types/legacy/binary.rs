use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::os::raw::c_void;
use std::slice;

use crate::borrow::internal::Pointer;
use crate::borrow::{Borrow, BorrowMut, LoanError, Ref, RefMut};
use crate::context::Lock;
use crate::handle::Managed;
use crate::types::{JsArrayBuffer, JsBuffer};

/// A reference to the internal backing buffer data of a `Buffer` or `ArrayBuffer` object, which can be accessed via the `Borrow` and `BorrowMut` traits.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct BinaryData<'a> {
    base: *mut c_void,
    size: usize,
    phantom: PhantomData<&'a ()>,
}

unsafe impl<'a> Pointer for BinaryData<'a> {
    unsafe fn as_ptr(&self) -> *const c_void {
        self.base
    }

    unsafe fn as_mut(&mut self) -> *mut c_void {
        self.base
    }
}

/// The trait for element types by which a buffer's binary data can be indexed.
pub trait BinaryViewType: Sized {}

impl BinaryViewType for u8 {}
impl BinaryViewType for i8 {}
impl BinaryViewType for u16 {}
impl BinaryViewType for i16 {}
impl BinaryViewType for u32 {}
impl BinaryViewType for i32 {}
impl BinaryViewType for u64 {}
impl BinaryViewType for i64 {}
impl BinaryViewType for f32 {}
impl BinaryViewType for f64 {}

impl<'a> BinaryData<'a> {
    /// Produces an immutable slice as a view into the contents of this buffer.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// # use neon::prelude::*;
    /// # fn get_x_and_y(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// let b: Handle<JsArrayBuffer> = cx.argument(0)?;
    /// let (x, y) = cx.borrow(&b, |data| {
    ///     let slice = data.as_slice::<i32>();
    ///     (slice[0], slice[1])
    /// });
    /// # println!("({}, {})", x, y);
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    pub fn as_slice<T: BinaryViewType>(self) -> &'a [T] {
        if self.size == 0 {
            &[]
        } else {
            let base = self.base.cast();
            let len = self.size / mem::size_of::<T>();
            unsafe { slice::from_raw_parts(base, len) }
        }
    }

    /// Produces a mutable slice as a view into the contents of this buffer.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// # use neon::prelude::*;
    /// # fn modify_buffer(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /// let mut b: Handle<JsArrayBuffer> = cx.argument(0)?;
    /// cx.borrow_mut(&mut b, |data| {
    ///     let slice = data.as_mut_slice::<f64>();
    ///     slice[0] /= 2.0;
    ///     slice[1] *= 2.0;
    /// });
    /// # Ok(cx.undefined())
    /// # }
    /// ```
    pub fn as_mut_slice<T: BinaryViewType>(self) -> &'a mut [T] {
        if self.size == 0 {
            &mut []
        } else {
            let base = self.base.cast();
            let len = self.size / mem::size_of::<T>();
            unsafe { slice::from_raw_parts_mut(base, len) }
        }
    }

    /// Produces the length of the buffer, in bytes.
    pub fn len(self) -> usize {
        self.size
    }

    /// Returns `true` if the buffer is empty
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
}

impl<'a> Borrow for &'a JsBuffer {
    type Target = BinaryData<'a>;

    fn try_borrow<'b>(self, guard: &'b Lock<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        let mut data = MaybeUninit::<BinaryData>::uninit();

        // Initialize pointer
        unsafe {
            let pointer = data.as_mut_ptr();
            (*pointer).size =
                neon_runtime::buffer::data(guard.env.to_raw(), &mut (*pointer).base, self.to_raw());
        }

        // UB if pointer is not initialized!
        unsafe { Ref::new(guard, data.assume_init()) }
    }
}

impl<'a> Borrow for &'a mut JsBuffer {
    type Target = BinaryData<'a>;

    fn try_borrow<'b>(self, guard: &'b Lock<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        (self as &'a JsBuffer).try_borrow(guard)
    }
}

impl<'a> BorrowMut for &'a mut JsBuffer {
    fn try_borrow_mut<'b>(
        self,
        guard: &'b Lock<'b>,
    ) -> Result<RefMut<'b, Self::Target>, LoanError> {
        let mut data = MaybeUninit::<BinaryData>::uninit();

        // Initialize pointer
        unsafe {
            let pointer = data.as_mut_ptr();
            (*pointer).size =
                neon_runtime::buffer::data(guard.env.to_raw(), &mut (*pointer).base, self.to_raw());
        }

        // UB if pointer is not initialized!
        unsafe { RefMut::new(guard, data.assume_init()) }
    }
}

impl<'a> Borrow for &'a JsArrayBuffer {
    type Target = BinaryData<'a>;

    fn try_borrow<'b>(self, guard: &'b Lock<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        let mut data = MaybeUninit::<BinaryData>::uninit();

        // Initialize pointer
        unsafe {
            let pointer = data.as_mut_ptr();
            (*pointer).size = neon_runtime::arraybuffer::data(
                guard.env.to_raw(),
                &mut (*pointer).base,
                self.to_raw(),
            );
        }

        // UB if pointer is not initialized!
        unsafe { Ref::new(guard, data.assume_init()) }
    }
}

impl<'a> Borrow for &'a mut JsArrayBuffer {
    type Target = BinaryData<'a>;

    fn try_borrow<'b>(self, guard: &'b Lock<'b>) -> Result<Ref<'b, Self::Target>, LoanError> {
        (self as &'a JsArrayBuffer).try_borrow(guard)
    }
}

impl<'a> BorrowMut for &'a mut JsArrayBuffer {
    fn try_borrow_mut<'b>(
        self,
        guard: &'b Lock<'b>,
    ) -> Result<RefMut<'b, Self::Target>, LoanError> {
        let mut data = MaybeUninit::<BinaryData>::uninit();

        // Initialize pointer
        unsafe {
            let pointer = data.as_mut_ptr();
            (*pointer).size = neon_runtime::arraybuffer::data(
                guard.env.to_raw(),
                &mut (*pointer).base,
                self.to_raw(),
            );
        }

        // UB if pointer is not initialized!
        unsafe { RefMut::new(guard, data.assume_init()) }
    }
}

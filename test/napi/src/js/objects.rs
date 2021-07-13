use neon::prelude::*;

pub fn return_js_global_object(mut cx: FunctionContext) -> JsResult<JsObject> {
    Ok(cx.global())
}

pub fn return_js_object(mut cx: FunctionContext) -> JsResult<JsObject> {
    Ok(cx.empty_object())
}

pub fn return_js_object_with_mixed_content(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_object: Handle<JsObject> = cx.empty_object();
    let n = cx.number(9000.0);
    js_object.set(&mut cx, "number", n)?;
    let s = cx.string("hello node");
    js_object.set(&mut cx, "string", s)?;
    Ok(js_object)
}

pub fn return_js_object_with_number(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_object: Handle<JsObject> = cx.empty_object();
    let n = cx.number(9000.0);
    js_object.set(&mut cx, "number", n)?;
    Ok(js_object)
}

pub fn return_js_object_with_string(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_object: Handle<JsObject> = cx.empty_object();
    let s = cx.string("hello node");
    js_object.set(&mut cx, "string", s)?;
    Ok(js_object)
}

pub fn return_js_object_with_symbol_property_key(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_object = cx.empty_object();
    let s = cx.symbol("neon:description");
    js_object.set(&mut cx, s, s)?;
    Ok(js_object)
}

pub fn return_array_buffer(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    let b: Handle<JsArrayBuffer> = cx.array_buffer(16)?;
    Ok(b)
}

pub fn read_array_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let b: Handle<JsArrayBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32 as usize;
    let x = {
        let guard = cx.lock();
        let data = b.borrow(&guard);
        let slice = data.as_slice::<u32>();
        slice[i]
    };
    Ok(cx.number(x))
}

pub fn read_array_buffer_with_borrow(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let b: Handle<JsArrayBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32 as usize;
    let x = cx.borrow(&b, |data| data.as_slice::<u32>()[i]);
    Ok(cx.number(x))
}

pub fn sum_array_buffer_with_borrow(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let b: Handle<JsArrayBuffer> = cx.argument(0)?;
    let x: u8 = cx.borrow(&b, |data| data.as_slice::<u8>().iter().sum());
    Ok(cx.number(x))
}

pub fn write_array_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut b: Handle<JsArrayBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32 as usize;
    let x = cx.argument::<JsNumber>(2)?.value(&mut cx) as u32;
    {
        let guard = cx.lock();
        let data = b.borrow_mut(&guard);
        let slice = data.as_mut_slice::<u32>();
        slice[i] = x;
    }
    Ok(cx.undefined())
}

pub fn write_array_buffer_with_borrow_mut(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut b: Handle<JsArrayBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32 as usize;
    let x = cx.argument::<JsNumber>(2)?.value(&mut cx) as u32;
    cx.borrow_mut(&mut b, |data| {
        data.as_mut_slice::<u32>()[i] = x;
    });
    Ok(cx.undefined())
}

pub fn increment_array_buffer_with_borrow_mut(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut b: Handle<JsArrayBuffer> = cx.argument(0)?;
    cx.borrow_mut(&mut b, |data| {
        data.as_mut_slice::<i8>().iter_mut().for_each(|x| *x += 1);
    });
    Ok(cx.undefined())
}

pub fn return_uninitialized_buffer(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let b: Handle<JsBuffer> = unsafe { JsBuffer::uninitialized(&mut cx, 16)? };
    Ok(b)
}

pub fn return_buffer(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let b: Handle<JsBuffer> = cx.buffer(16)?;
    Ok(b)
}

pub fn return_external_buffer(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let data = cx.argument::<JsString>(0)?.value(&mut cx);
    let buf = JsBuffer::external(&mut cx, data.into_bytes());

    Ok(buf)
}

pub fn return_external_array_buffer(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    let data = cx.argument::<JsString>(0)?.value(&mut cx);
    let buf = JsArrayBuffer::external(&mut cx, data.into_bytes());

    Ok(buf)
}

pub fn read_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let b: Handle<JsBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32 as usize;
    let x = {
        let guard = cx.lock();
        let data = b.borrow(&guard);
        let slice = data.as_slice::<u32>();
        slice[i]
    };
    Ok(cx.number(x))
}

pub fn read_buffer_with_borrow(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let b: Handle<JsBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32 as usize;
    let x = cx.borrow(&b, |data| data.as_slice::<u32>()[i]);
    Ok(cx.number(x))
}

pub fn sum_buffer_with_borrow(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let b: Handle<JsBuffer> = cx.argument(0)?;
    let x: u8 = cx.borrow(&b, |data| data.as_slice::<u8>().iter().sum());
    Ok(cx.number(x))
}

pub fn write_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut b: Handle<JsBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32 as usize;
    let x = cx.argument::<JsNumber>(2)?.value(&mut cx) as u32;
    {
        let guard = cx.lock();
        let data = b.borrow_mut(&guard);
        let slice = data.as_mut_slice::<u32>();
        slice[i] = x;
    }
    Ok(cx.undefined())
}

pub fn write_buffer_with_borrow_mut(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut b: Handle<JsBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32 as usize;
    let x = cx.argument::<JsNumber>(2)?.value(&mut cx) as u32;
    cx.borrow_mut(&mut b, |data| {
        data.as_mut_slice::<u32>()[i] = x;
    });
    Ok(cx.undefined())
}

pub fn increment_buffer_with_borrow_mut(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut b: Handle<JsBuffer> = cx.argument(0)?;
    cx.borrow_mut(&mut b, |data| {
        data.as_mut_slice::<i8>().iter_mut().for_each(|x| *x += 1);
    });
    Ok(cx.undefined())
}

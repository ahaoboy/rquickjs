use crate::{
    context::Ctx,
    qjs,
    value::{self, rf::JsStringRef},
    Error, Result,
};
use std::{mem, slice, str, string::String as StdString};

/// Rust representation of a javascript string.
#[derive(Debug, Clone, PartialEq)]
pub struct String<'js>(pub(crate) JsStringRef<'js>);

impl<'js> String<'js> {
    /// Convert the javascript string to a rust string.
    pub fn to_string(&self) -> Result<StdString> {
        let mut len = mem::MaybeUninit::uninit();
        let ptr =
            unsafe { qjs::JS_ToCStringLen(self.0.ctx.ctx, len.as_mut_ptr(), self.0.as_js_value()) };
        if ptr.is_null() {
            // Might not ever happen but I am not 100% sure
            // so just incase check it.
            return Err(Error::Unknown);
        }
        let len = unsafe { len.assume_init() };
        let bytes: &[u8] = unsafe { slice::from_raw_parts(ptr as _, len as _) };
        let result = str::from_utf8(bytes).map(|s| s.into());
        unsafe { qjs::JS_FreeCString(self.0.ctx.ctx, ptr) };
        result.map_err(Error::from)
    }

    /// Create a new js string from an rust string.
    pub fn from_str(ctx: Ctx<'js>, s: &str) -> Result<Self> {
        let len = s.as_bytes().len();
        let ptr = s.as_ptr();
        Ok(String(unsafe {
            let js_val = qjs::JS_NewStringLen(ctx.ctx, ptr as _, len as _);
            let js_val = value::handle_exception(ctx, js_val)?;
            JsStringRef::from_js_value(ctx, js_val)
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    #[test]
    fn from_javascript() {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let val = ctx.eval::<Value, _>(" 'foo bar baz' ");
            if let Ok(Value::String(x)) = val {
                assert_eq!(x.to_string().unwrap(), "foo bar baz".to_string())
            } else {
                panic!("val not a string");
            };
        });
    }

    #[test]
    fn to_javascript() {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let string = String::from_str(ctx, "foo").unwrap();
            let func = ctx.eval::<Function, _>("(x) =>  x + 'bar'").unwrap();
            let text: StdString = func.call(string).unwrap();
            assert_eq!(text, "foobar".to_string());
        });
    }
}

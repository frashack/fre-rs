use super::*;


/// An abstraction over objects of all types.
/// 
/// **In typical usage of this crate, this trait should not be implemented directly.**
///
/// This trait represents a common interface for types that are backed by an [`FREObject`].
/// It is only intended to be implemented when defining a new object type.
///
/// # Safety
///
/// Implementing this trait requires strict guarantees about memory layout.
/// The implementing type must be layout-compatible with [`FREObject`].
/// In practice, this means it must be annotated with `#[repr(transparent)]`
/// and wrap the underlying [`FREObject`] without altering its representation.
///
/// This requirement exists because the methods provided by this trait may
/// perform reinterpretation of the underlying memory. Failure to uphold
/// these guarantees will result in undefined behavior.
/// 
pub unsafe trait AsObject<'a>: Sized + Copy + Eq + Display + Into<FREObject> + Into<Object<'a>> {

    /// The Type associated with the struct.
    ///
    /// This does not represent the actual runtime type of the object.
    /// 
    const TYPE: Type;

    fn as_object (self) -> Object<'a> {
        debug_assert_eq!(size_of_val(&self), size_of::<FREObject>());
        unsafe {transmute_unchecked(self)}
    }
    fn as_ptr (self) -> FREObject {
        unsafe {transmute(self.as_object())}
    }
    fn is_null(self) -> bool {self.as_ptr().is_null()}

    /// Returns the runtime type of the object.
    ///
    /// The result depends on the underlying [`FREGetObjectType`] implementation.
    /// Most unsupported types return [`Type::Object`].
    /// 
    fn get_type(self) -> Type {
        let mut ty = FREObjectType(i32::default());
        let r = unsafe {FREGetObjectType(self.as_ptr(), &mut ty)};
        assert!(r.is_ok());
        ty.into()
    }
    
    fn get_property (self, name: UCStr) -> Result<Object<'a>, ExternalError<'a>> {
        let mut object = std::ptr::null_mut();
        let mut thrown = std::ptr::null_mut();
        let r = unsafe {FREGetObjectProperty(self.as_ptr(), name.as_ptr(), &mut object, &mut thrown)};
        if let Ok(e) = ExternalError::try_from(r, Some(unsafe {transmute(thrown)})) {
            Err(e)
        }else{
            Ok(unsafe {transmute(object)})
        }
    }

    fn set_property <O: AsObject<'a>> (self, name: UCStr, value: O) -> Result<(), ExternalError<'a>> {
        let mut thrown = std::ptr::null_mut();
        let r = unsafe {FRESetObjectProperty(self.as_ptr(), name.as_ptr(), value.as_ptr(), &mut thrown)};
        if let Ok(e) = ExternalError::try_from(r, Some(unsafe {transmute(thrown)})) {
            Err(e)
        }else{
            Ok(())
        }
    }

    fn call_method (self, name: UCStr, args: Option<&[Object]>) -> Result<Object<'a>, ExternalError<'a>> {
        let args = args.unwrap_or_default();
        debug_assert!(args.len() <= u32::MAX as usize);
        let mut obj = std::ptr::null_mut();
        let mut thrown = std::ptr::null_mut();
        let r = unsafe {FRECallObjectMethod(self.as_ptr(), name.as_ptr(), args.len() as u32, transmute(args.as_ptr()), &mut obj, &mut thrown)};
        if let Ok(e) = ExternalError::try_from(r, Some(unsafe {transmute(thrown)})) {
            Err(e)
        }else{
            Ok(unsafe {transmute(obj)})
        }
    }

    /// Return [`Err`] if this is `null` or `undefined`.
    #[allow(non_snake_case)]
    fn toString (self) -> Result<as3::String<'a>, ExternalError<'a>> {
        const TO_STRING: UCStr = unsafe {UCStr::from_literal_unchecked(c"toString")};
        self.call_method(TO_STRING, None).map(|r|{unsafe {transmute(r)}})
    }
}


/// An abstraction for fallible casting between object types.
///
/// **In typical usage of this crate, this trait should not be implemented directly.**
/// 
/// Primarily used for object type casting, and should be implemented when defining a new type.
/// 
pub unsafe trait TryAs<'a, T>: AsObject<'a> + TryInto<T>
where T: AsObject<'a> {
    fn try_as (self) -> Result<T, Type> {
        let ty = self.get_type();
        if ty == T::TYPE {
            debug_assert_eq!(size_of_val(&self), size_of::<T>());
            Ok(unsafe {transmute_unchecked(self)})
        }else{Err(ty)}
    }
}
unsafe impl<'a, O> TryAs<'a, Object<'a>> for O
where O: AsObject<'a> {
    fn try_as (self) -> Result<Object<'a>, Type> {Ok(self.as_object())}
}


/// A wrapper around [`FREObject`].
///
/// This type assumes that the underlying handle is always valid.
/// The runtime is responsible for ensuring the correctness and lifetime
/// of the handle.
///
/// Note that the handle may still be [`as3::null`], depending on the API behavior.
/// In such cases, [`as3::null`] is treated as a valid value at the ABI level,
/// but may represent the absence of an object.
/// 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Object <'a> (FREObject, PhantomData<&'a()>);
impl<'a> Object<'a> {
    pub fn new (_: &CurrentContext<'a>, class: UCStr, args: Option<&[Object<'a>]>) -> Result<Object<'a>, ExternalError<'a>> {
        let args = args.unwrap_or_default();
        debug_assert!(args.len() <= u32::MAX as usize);
        let mut object = std::ptr::null_mut();
        let mut thrown = std::ptr::null_mut();
        let r = unsafe {FRENewObject(class.as_ptr(), args.len() as u32, transmute(args.as_ptr()), &mut object, &mut thrown)};
        if let Ok(e) = ExternalError::try_from(r, Some(unsafe {transmute(thrown)})) {
            Err(e)
        }else{
            assert!(!object.is_null());
            Ok(unsafe {transmute(object)})
        }
    }

    /// [`FRENativeWindow`] is only valid for the duration of this closure call.
    /// 
    /// Using [`as3::null`] inside the closure is meaningless and may lead to unintended FFI call ordering.
    /// 
    /// This is a minimal safety wrapper around the underlying FFI. Its current
    /// placement, shape, and usage are not ideal, and it is expected to be
    /// refactored if the ANE C API allows more precise determination of an
    /// object's concrete type.
    /// 
    pub fn with_native_window <F, R> (self, f: F) -> Result<R, FfiError>
    where F: FnOnce (FRENativeWindow) -> R + Sync
    {
        let mut handle = std::ptr::null_mut();
        let result = unsafe {FREAcquireNativeWindowHandle(self.as_ptr(), &mut handle)};
        if let Ok(e) = FfiError::try_from(result) {return Err(e)};
        let r = f(handle);
        let result = unsafe {FREReleaseNativeWindowHandle(self.as_ptr())};
        assert!(result.is_ok());
        Ok(r)
    }

    /// Using [`as3::null`] inside the closure is meaningless and may lead to unintended FFI call ordering.
    /// 
    /// This is a minimal safety wrapper around the underlying FFI. Its current
    /// placement, shape, and usage are not ideal, and it is expected to be
    /// refactored if the ANE C API allows more precise determination of an
    /// object's concrete type.
    /// 
    pub fn with_native_window_3d <F, R> (self, f: F) -> Result<R, ExternalError<'a>>
    where F: FnOnce (FRENativeWindow, &[Option<Context3D<'a>>]) -> R + Sync
    {
        const NAME_STAGE: UCStr = unsafe {UCStr::from_literal_unchecked(c"stage")};
        const NAME_STAGE_3DS: UCStr = unsafe {UCStr::from_literal_unchecked(c"stage3Ds")};
        const NAME_CONTEXT_3D: UCStr = unsafe {UCStr::from_literal_unchecked(c"context3D")};
        let stage3ds: Vector = self.get_property(NAME_STAGE)?
            .get_property(NAME_STAGE_3DS)?
            .try_as()
            .map_err(|_|ExternalError::C(FfiError::TypeMismatch))?;
        let ctx3ds: Box<[Option<Context3D>]>  = stage3ds.iter()
            .map(|stage3d|{
                stage3d.get_property(NAME_CONTEXT_3D)
                    .ok()
            })
            .map(|i|{
                if let Some(ctx3d) = i {
                    if ctx3d.is_null() {None} else {Some(unsafe {transmute(ctx3d)})}
                } else {None}
            })
            .collect();
        let mut handle = std::ptr::null_mut();
        let result = unsafe {FREAcquireNativeWindowHandle(self.as_ptr(), &mut handle)};
        if let Ok(e) = FfiError::try_from(result) {return Err(e.into())};
        let r = f(handle, ctx3ds.as_ref());
        let result = unsafe {FREReleaseNativeWindowHandle(self.as_ptr())};
        debug_assert!(result.is_ok());
        Ok(r)
    }

}
impl TryFrom<Object<'_>> for i32 {
    type Error = FfiError;
    fn try_from(value: Object) -> Result<Self, Self::Error> {
        let mut val = i32::default();
        let r = unsafe {FREGetObjectAsInt32(value.0, &mut val)};
        if let Ok(e) = r.try_into() {return Err(e);}
        Ok(val)
    }
}
impl TryFrom<Object<'_>> for u32 {
    type Error = FfiError;
    fn try_from(value: Object) -> Result<Self, Self::Error> {
        let mut val = u32::default();
        let r = unsafe {FREGetObjectAsUint32(value.0, &mut val)};
        if let Ok(e) = r.try_into() {return Err(e);}
        Ok(val)
    }
}
impl TryFrom<Object<'_>> for f64 {
    type Error = FfiError;
    fn try_from(value: Object) -> Result<Self, Self::Error> {
        let mut val = f64::default();
        let r = unsafe {FREGetObjectAsDouble(value.0, &mut val)};
        if let Ok(e) = r.try_into() {return Err(e);}
        Ok(val)
    }
}
impl TryFrom<Object<'_>> for bool {
    type Error = FfiError;
    fn try_from(value: Object) -> Result<Self, Self::Error> {
        let mut val = u32::default();
        let r = unsafe {FREGetObjectAsBool(value.0, &mut val)};
        if let Ok(e) = r.try_into() {return Err(e);}
        Ok(val != 0)
    }
}
impl<'a> TryFrom<Object<'a>> for &'a str {
    type Error = FfiError;
    fn try_from(value: Object) -> Result<Self, Self::Error> {
        let mut len = u32::default();
        let mut ptr = std::ptr::null();
        let r = unsafe {FREGetObjectAsUTF8(value.0, &mut len, &mut ptr)};
        if let Ok(e) = r.try_into() {return Err(e);}
        let bytes = unsafe {std::slice::from_raw_parts(ptr, len as usize)};
        let s = unsafe {str::from_utf8_unchecked(bytes)};
        Ok(s)
    }
}
impl Display for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {return write!(f, "null");}
        match self.toString() {
            Ok(s) => {Display::fmt(s.value(), f)},
            Err(e) => {
                match e {
                    ExternalError::C(e) => Display::fmt(&e, f),
                    ExternalError::ActionScript(e) => {
                        if let Ok(s) = e.thrown().toString().map(|s|{s.value()}) {
                            write!(f, "{e} {s}")
                        } else {Display::fmt(&e, f)}
                    },
                }
            },
        }
    }
}
impl Default for Object<'_> {
    fn default() -> Self {as3::null}
}
unsafe impl<'a> AsObject<'a> for Object<'a> {const TYPE: Type = Type::Object;}
impl From<()> for Object<'_> {fn from(_: ()) -> Self {as3::null}}
impl<'a, O: AsObject<'a>> From<Option<O>> for Object<'a> {
    fn from(value: Option<O>) -> Self {
        if let Some(obj) = value {
            obj.as_object()
        } else {as3::null}
    }
}
impl From<Object<'_>> for FREObject {fn from(value: Object) -> Self {value.as_ptr()}}
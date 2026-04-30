use super::*;
use crate::__private::Sealed;


/// An abstraction over all AS3 objects.
/// 
/// This trait is **`Sealed`** and should not be implemented manually.
/// 
/// To define a new AS3 class, use the [`class!`] macro.
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
pub unsafe trait AsObject<'a>: Sealed + Sized + Debug + Display + Copy + Into<Object<'a>> + Into<FREObject> {

    /// The [`Type`] associated with the class.
    ///
    /// This does not represent the actual runtime type of the object.
    /// 
    const TYPE: Type;

    #[inline]
    fn as_object (self) -> Object<'a> {unsafe {transmute_unchecked(self)}}

    /// Casts this object to `T` without checks.
    /// 
    /// Prefer to use [`TryAs::try_as`] instead whenever possible.
    ///
    /// # Safety
    ///
    /// The concrete object type must be fully determined, must be type-checked
    /// and validated in AS3, and the conversion must strictly follow the
    /// semantics of the AS3 `as` operator.
    ///
    /// Violating these requirements may cause related APIs to perform illegal FFI
    /// calls, which may panic even if those APIs do not explicitly document panics.
    /// Violations may also result in undefined behavior.
    /// 
    #[allow(unsafe_op_in_unsafe_fn)]
    #[inline]
    unsafe fn as_unchecked <T: AsObject<'a>> (self) -> T {T::from_unchecked(self)}

    /// Casts an object to `Self` without checks.
    /// 
    /// Prefer to use [`TryAs::try_as`] instead whenever possible.
    /// 
    /// # Safety
    /// 
    /// The concrete object type must be fully determined, must be type-checked
    /// and validated in AS3, and the conversion must strictly follow the
    /// semantics of the AS3 `as` operator.
    /// 
    /// Violating these requirements may cause related APIs to perform illegal FFI
    /// calls, which may panic even if those APIs do not explicitly document panics.
    /// Violations may also result in undefined behavior.
    /// 
    #[inline]
    unsafe fn from_unchecked <O: AsObject<'a>> (object: O) -> Self {
        #[cfg(debug_assertions)]
        if object.is_null() && Self::TYPE != Object::TYPE && Self::TYPE != Type::null {
            panic!("Cannot cast `null` to `{}`.", Self::TYPE);
        }
        unsafe {transmute_unchecked(object)}
    }

    #[inline]
    fn as_ptr (self) -> FREObject {unsafe {transmute_unchecked(self)}}

    #[inline]
    fn is_null(self) -> bool {false}

    /// Returns the runtime type of the object.
    /// 
    /// The result depends on the underlying [`FREGetObjectType`] implementation.
    /// Most unsupported types return [`Type::NonNullObject`].
    /// 
    fn get_type(self) -> Type {
        let mut ty = MaybeUninit::<FREObjectType>::uninit();
        let r = unsafe {FREGetObjectType(self.as_ptr(), ty.as_mut_ptr())};
        if r.is_ok() {
            let ty = unsafe {ty.assume_init()};
            ty.into()
        } else if r == FREResult::FRE_WRONG_THREAD {

            // Assumes that only `as3::null` can cause `FREResult::FRE_WRONG_THREAD`.
            Type::null
        } else {unreachable!()}
    }
    
    fn get_property (self, name: UCStr) -> Result<Object<'a>, ExternalError<'a>> {
        let mut object = MaybeUninit::<FREObject>::uninit();
        let mut thrown = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FREGetObjectProperty(self.as_ptr(), name.as_ptr(), object.as_mut_ptr(), thrown.as_mut_ptr())};
        if let Some(e) = ExternalError::try_from(r, Some(unsafe {transmute(thrown)})) {
            Err(e)
        }else{
            Ok(unsafe {transmute(object)})
        }
    }

    fn set_property <O: AsObject<'a>> (self, name: UCStr, value: O) -> Result<(), ExternalError<'a>> {
        let mut thrown = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FRESetObjectProperty(self.as_ptr(), name.as_ptr(), value.as_ptr(), thrown.as_mut_ptr())};
        if let Some(e) = ExternalError::try_from(r, Some(unsafe {transmute(thrown)})) {
            Err(e)
        }else{
            Ok(())
        }
    }

    fn call_method (self, name: UCStr, args: Option<&[Object]>) -> Result<Object<'a>, ExternalError<'a>> {
        let args = args.unwrap_or_default();
        debug_assert!(args.len() <= u32::MAX as usize);
        let mut object = MaybeUninit::<FREObject>::uninit();
        let mut thrown = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FRECallObjectMethod(self.as_ptr(), name.as_ptr(), args.len() as u32, transmute(args.as_ptr()), object.as_mut_ptr(), thrown.as_mut_ptr())};
        if let Some(e) = ExternalError::try_from(r, Some(unsafe {transmute(thrown)})) {
            Err(e)
        }else{
            Ok(unsafe {transmute(object)})
        }
    }

    /// Returns [`Err`] when this is `null` or `undefined`.
    /// 
    /// Returns [`Err`] when an AS3 error occurs.
    /// 
    #[allow(non_snake_case)]
    fn toString (self) -> Result<as3::String<'a>, ExternalError<'a>> {
        self.call_method(crate::ucstringify!(toString), None).map(|r|r.try_as().unwrap())
    }
}
impl<'a, O> From<O> for Object<'a>
where O: AsNonNullObject<'a> {
    fn from(object: O) -> Self {object.as_object()}
}
// crate::class!(...) => impl AsObject for ...


/// Implemented for all classes except [`Object`], providing casting to [`NonNullObject`].
/// 
pub trait AsNonNullObject<'a>: AsObject<'a> {
    #[inline]
    fn as_non_null_object(self) -> NonNullObject<'a> {unsafe {self.as_unchecked()}}
}
// crate::class!(...) => impl AsNonNullObject for ...
// crate::class!(...) => impl From<...> for NonNullObject


/// An abstraction for fallible casting between object types.
///
/// Implement [`TryFrom<Self, Error = Type>`] instead of implementing this trait directly.
/// 
pub trait TryAs<'a, T>: AsObject<'a> + TryInto<T, Error = Type>
where T: AsObject<'a> + TryFrom<Self, Error = Type> {

    /// This function must follow the semantics of the AS3 `as` operator.
    /// 
    fn try_as (self) -> Result<T, Type>;
}
impl<'a, O, T> TryAs<'a, T> for O
where
    O: AsObject<'a> + TryInto<T, Error = Type>,
    T: AsObject<'a> + TryFrom<O, Error = Type>,
{
    fn try_as (self) -> Result<T, Type> {T::try_from(self)}
}
// crate::class!(...) => impl TryFrom<...> for Object
// crate::class!(...) => impl TryFrom<...> for NonNullObject


/// A reference to an AS3 object.
///
/// This type assumes that the underlying handle is always valid.
/// The runtime is responsible for ensuring the correctness and lifetime
/// of the handle.
///
/// The handle may still be [`as3::null`], depending on the API behavior.
/// In such cases, [`as3::null`] is treated as a valid value at the ABI level,
/// but may represent the absence of an object.
/// 
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Object<'a> (FREObject, PhantomData<&'a ()>);

/// A reference to a non-null AS3 object.
/// 
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct NonNullObject<'a> (NonNull<c_void>, PhantomData<&'a ()>);

impl<'a> Object<'a> {
    pub fn new (ctx: &CurrentContext<'a>) -> NonNullObject<'a> {NonNullObject::new(ctx)}
    pub fn non_null (self) -> Option<NonNullObject<'a>> {NonNullObject::from_object(self)}
}
impl<'a> NonNullObject<'a> {
    pub fn new (ctx: &CurrentContext<'a>) -> Self {ctx.construct(crate::ucstringify!(Object), None).unwrap()}
    pub fn from_object (object: Object<'a>) -> Option<NonNullObject<'a>> {NonNull::new(object.0).map(|object|unsafe {transmute(object)})}
}

impl<'a> TryFrom<Object<'a>> for NonNullObject<'a> {
    type Error = ();
    fn try_from(object: Object<'a>) -> Result<Self, Self::Error> {Self::from_object(object).ok_or(())}
}

impl TryFrom<Object<'_>> for i32 {
    type Error = FfiError;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let mut value = MaybeUninit::<i32>::uninit();
        let r = unsafe {FREGetObjectAsInt32(object.0, value.as_mut_ptr())};
        if let Ok(e) = r.try_into() {return Err(e);}
        let value = unsafe {value.assume_init()};
        Ok(value)
    }
}
impl TryFrom<&Object<'_>> for i32 {type Error = FfiError; fn try_from(object: &Object) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&mut Object<'_>> for i32 {type Error = FfiError; fn try_from(object: &mut Object) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<NonNullObject<'_>> for i32 {type Error = FfiError; fn try_from(object: NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&NonNullObject<'_>> for i32 {type Error = FfiError; fn try_from(object: &NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&mut NonNullObject<'_>> for i32 {type Error = FfiError; fn try_from(object: &mut NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}

impl TryFrom<Object<'_>> for u32 {
    type Error = FfiError;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let mut value = MaybeUninit::<u32>::uninit();
        let r = unsafe {FREGetObjectAsUint32(object.0, value.as_mut_ptr())};
        if let Ok(e) = r.try_into() {return Err(e);}
        let value = unsafe {value.assume_init()};
        Ok(value)
    }
}
impl TryFrom<&Object<'_>> for u32 {type Error = FfiError; fn try_from(object: &Object) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&mut Object<'_>> for u32 {type Error = FfiError; fn try_from(object: &mut Object) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<NonNullObject<'_>> for u32 {type Error = FfiError; fn try_from(object: NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&NonNullObject<'_>> for u32 {type Error = FfiError; fn try_from(object: &NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&mut NonNullObject<'_>> for u32 {type Error = FfiError; fn try_from(object: &mut NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}

impl TryFrom<Object<'_>> for f64 {
    type Error = FfiError;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let mut value = MaybeUninit::<f64>::uninit();
        let r = unsafe {FREGetObjectAsDouble(object.0, value.as_mut_ptr())};
        if let Ok(e) = r.try_into() {return Err(e);}
        let value = unsafe {value.assume_init()};
        Ok(value)
    }
}
impl TryFrom<&Object<'_>> for f64 {type Error = FfiError; fn try_from(object: &Object) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&mut Object<'_>> for f64 {type Error = FfiError; fn try_from(object: &mut Object) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<NonNullObject<'_>> for f64 {type Error = FfiError; fn try_from(object: NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&NonNullObject<'_>> for f64 {type Error = FfiError; fn try_from(object: &NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&mut NonNullObject<'_>> for f64 {type Error = FfiError; fn try_from(object: &mut NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}

impl TryFrom<Object<'_>> for bool {
    type Error = FfiError;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let mut value = MaybeUninit::<u32>::uninit();
        let r = unsafe {FREGetObjectAsBool(object.0, value.as_mut_ptr())};
        if let Ok(e) = r.try_into() {return Err(e);}
        let value = unsafe {value.assume_init()};
        Ok(value != 0)
    }
}
impl TryFrom<&Object<'_>> for bool {type Error = FfiError; fn try_from(object: &Object) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&mut Object<'_>> for bool {type Error = FfiError; fn try_from(object: &mut Object) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<NonNullObject<'_>> for bool {type Error = FfiError; fn try_from(object: NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&NonNullObject<'_>> for bool {type Error = FfiError; fn try_from(object: &NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl TryFrom<&mut NonNullObject<'_>> for bool {type Error = FfiError; fn try_from(object: &mut NonNullObject) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}

impl<'a> TryFrom<Object<'a>> for &'a str {
    type Error = FfiError;
    fn try_from(object: Object<'a>) -> Result<Self, Self::Error> {
        let mut len = MaybeUninit::<u32>::uninit();
        let mut ptr = MaybeUninit::<FREStr>::uninit();
        let r = unsafe {FREGetObjectAsUTF8(object.0, len.as_mut_ptr(), ptr.as_mut_ptr())};
        if let Ok(e) = r.try_into() {return Err(e);}
        let len = unsafe {len.assume_init()};
        let ptr = unsafe {ptr.assume_init()};
        let bytes = unsafe {std::slice::from_raw_parts(ptr, len as usize)};
        let s = unsafe {str::from_utf8_unchecked(bytes)};
        Ok(s)
    }
}
impl<'a> TryFrom<&Object<'a>> for &'a str {type Error = FfiError; fn try_from(object: &Object<'a>) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl<'a> TryFrom<&mut Object<'a>> for &'a str {type Error = FfiError; fn try_from(object: &mut Object<'a>) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl<'a> TryFrom<NonNullObject<'a>> for &'a str {type Error = FfiError; fn try_from(object: NonNullObject<'a>) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl<'a> TryFrom<&NonNullObject<'a>> for &'a str {type Error = FfiError; fn try_from(object: &NonNullObject<'a>) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}
impl<'a> TryFrom<&mut NonNullObject<'a>> for &'a str {type Error = FfiError; fn try_from(object: &mut NonNullObject<'a>) -> Result<Self, Self::Error> {Self::try_from(object.as_object())}}

impl Display for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(obj) = NonNullObject::from_object(*self) {
            Display::fmt(&obj, f)
        } else {write!(f, "null")}
    }
}
impl Display for NonNullObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.toString() {
            Ok(s) => Display::fmt(s.value(), f),

            // May mutually call with `<ExternalError as Display>::fmt`.
            // Assumes thrown object's `toString` does not re-enter and cause an infinite call loop.
            Err(ref e) => Display::fmt(e, f),
        }
    }
}

impl Default for Object<'_> {
    fn default() -> Self {as3::null}
}

unsafe impl Sealed for Object<'_> {}
unsafe impl Sealed for NonNullObject<'_> {}
unsafe impl<'a> AsObject<'a> for Object<'a> {const TYPE: Type = Type::Named("Object");
    #[inline]
    fn is_null(self) -> bool {self.0.is_null() || self.get_type() == Type::null}
}
unsafe impl<'a> AsObject<'a> for NonNullObject<'a> {const TYPE: Type = Type::NonNullObject;}
impl<'a> AsNonNullObject<'a> for NonNullObject<'a> {fn as_non_null_object(self) -> NonNullObject<'a> {self}}

impl From<()> for Object<'_> {fn from(_: ()) -> Self {Self::default()}}
impl<'a, O: AsObject<'a>> From<Option<O>> for Object<'a> {
    fn from(value: Option<O>) -> Self {
        if let Some(obj) = value {
            obj.as_object()
        } else {as3::null}
    }
}

impl From<Object<'_>> for FREObject {fn from(object: Object) -> Self {object.as_ptr()}}
impl From<NonNullObject<'_>> for FREObject {fn from(object: NonNullObject) -> Self {object.as_ptr()}}


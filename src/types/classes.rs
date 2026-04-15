use std::i32;

use super::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Array <'a> (NonNullFREObject, PhantomData<&'a()>);
impl<'a> Array<'a> {
    const CLASS: UCStr = unsafe {UCStr::from_literal_unchecked(c"Array")};
    pub fn get_length (self) -> u32 {
        let mut value = u32::default();
        let r = unsafe {FREGetArrayLength(self.as_ptr(), &mut value)};
        debug_assert!(r.is_ok());
        value
    }
    /// [`Err`]=> [FfiError::InsufficientMemory];
    pub fn set_length (self, value: u32) -> Result<(), FfiError> {
        let r = unsafe {FRESetArrayLength(self.as_ptr(), value)};
        if let Ok(e) = FfiError::try_from(r) {
            Err(e)
        } else {Ok(())}
    }
    pub fn get (self, index: u32) -> Object<'a> {
        let mut obj = std::ptr::null_mut();
        let r = unsafe {FREGetArrayElementAt(self.as_ptr(), index, &mut obj)};
        debug_assert!(r.is_ok());
        debug_assert!(!obj.is_null());
        unsafe {transmute(obj)}
    }
    pub fn set <O: AsObject<'a>> (self, index: u32, value: O) {
        let r = unsafe {FRESetArrayElementAt(self.as_ptr(), index, value.as_ptr())};
        debug_assert!(r.is_ok());
    }
    pub fn new (ctx: &CurrentContext<'a>, num_elements: Option<NonNegativeInt>) -> Self {
        let mut arg = as3::null;
        let num_elements = num_elements.map(|v|{
            arg = int::new(ctx, v.get()).as_object();
            std::slice::from_ref(&arg)
        });
        let obj = Object::new(ctx, Self::CLASS, num_elements).unwrap();
        assert!(!obj.is_null());
        unsafe {transmute(obj)}
    }
    pub fn from_slice (frt: &CurrentContext<'a>, elements: &[Object<'a>]) -> Self {
        debug_assert!(elements.len() <= i32::MAX as usize);
        if elements.len() == 1 && elements[0].get_type() == Type::Number {
            let arr = Self::new(frt, NonNegativeInt::new(1));
            arr.set(0, *unsafe {elements.get_unchecked(0)});
            arr
        } else {
            let obj = Object::new(frt, Self::CLASS, Some(elements)).unwrap();
            debug_assert!(!obj.is_null());
            unsafe {transmute(obj)}
        }
    }
    pub fn extend_from_slice (self, elements: &[Object]) -> u32 {
        const PUSH: UCStr = unsafe {UCStr::from_literal_unchecked(c"push")};
        self.call_method(PUSH, Some(elements))
            .unwrap()
            .try_into()
            .unwrap()
    }
    pub fn push <O: AsObject<'a>> (self, element: O) -> u32 {
        let element = element.as_object();
        let elements = std::slice::from_ref(&element);
        self.extend_from_slice(elements)
    }
    pub fn iter(self) -> ArrayIter<'a> {ArrayIter::new(self)}
}
impl<'a> IntoIterator for Array<'a> {
    type Item = Object<'a>;
    type IntoIter = ArrayIter<'a>;
    fn into_iter(self) -> Self::IntoIter {self.iter()}
}
unsafe impl<'a> AsObject<'a> for Array<'a> {const TYPE: Type = Type::Array;}
unsafe impl<'a> TryAs<'a, Array<'a>> for Object<'a> {}
impl From<Array<'_>> for FREObject {fn from(value: Array) -> Self {value.as_ptr()}}
impl<'a> From<Array<'a>> for Object<'a> {fn from(value: Array<'a>) -> Self {value.as_object()}}
impl<'a> TryFrom<Object<'a>> for Array<'a> {type Error = Type; fn try_from (value: Object<'a>) -> Result<Self, Type> {value.try_as()}}
impl Display for Array<'_> {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {Display::fmt(&self.as_object(), f)}}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Vector <'a> (NonNullFREObject, PhantomData<&'a()>);
impl<'a> Vector<'a> {
    pub fn get_length (self) -> u32 {
        let mut value = u32::default();
        let r = unsafe {FREGetArrayLength(self.as_ptr(), &mut value)};
        debug_assert!(r.is_ok());
        value
    }
    /// [`Err`]=> [FfiError::InsufficientMemory], [FfiError::ReadOnly];
    pub fn set_length (self, value: u32) -> Result<(), FfiError> {
        let r = unsafe {FRESetArrayLength(self.as_ptr(), value)};
        if let Ok(e) = FfiError::try_from(r) {
            Err(e)
        } else {Ok(())}
    }
    /// [`Err`]=> [FfiError::InvalidArgument];
    pub fn get (self, index: u32) -> Result<Object<'a>, FfiError> {
        let mut obj = std::ptr::null_mut();
        let r = unsafe {FREGetArrayElementAt(self.as_ptr(), index, &mut obj)};
        if let Ok(e) = FfiError::try_from(r) {
            Err(e)
        } else {
            debug_assert!(!obj.is_null());
            Ok(unsafe {transmute(obj)})
        }
    }
    /// [`Err`]=> [FfiError::TypeMismatch];
    pub fn set <O: AsObject<'a>> (self, index: u32, value: O) -> Result<(), FfiError> {
        let r = unsafe {FRESetArrayElementAt(self.as_ptr(), index, value.as_ptr())};
        if let Ok(e) = FfiError::try_from(r) {
            Err(e)
        } else {Ok(())}
    }
    pub fn iter(self) -> VectorIter<'a> {VectorIter::new(self)}
}
impl<'a> IntoIterator for Vector<'a> {
    type Item = Object<'a>;
    type IntoIter = VectorIter<'a>;
    fn into_iter(self) -> Self::IntoIter {self.iter()}
}
unsafe impl<'a> AsObject<'a> for Vector<'a> {const TYPE: Type = Type::Vector;}
unsafe impl<'a> TryAs<'a, Vector<'a>> for Object<'a> {}
impl From<Vector<'_>> for FREObject {fn from(value: Vector) -> Self {value.as_ptr()}}
impl<'a> From<Vector<'a>> for Object<'a> {fn from(value: Vector<'a>) -> Self {value.as_object()}}
impl<'a> TryFrom<Object<'a>> for Vector<'a> {type Error = Type; fn try_from (value: Object<'a>) -> Result<Self, Type> {value.try_as()}}
impl Display for Vector<'_> {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {Display::fmt(&self.as_object(), f)}}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ByteArray <'a> (NonNullFREObject, PhantomData<&'a()>);
impl<'a> ByteArray<'a> {
    #[allow(unused_variables)]
    pub fn new (frt: &CurrentContext<'a>, length: u32) -> Self {
        let ptr = FREByteArray {length, bytes: std::ptr::null_mut()};
        let mut obj = std::ptr::null_mut();
        let r = unsafe {FRENewByteArray(transmute(&ptr), &mut obj)};
        debug_assert!(r.is_ok());
        assert!(!obj.is_null());
        unsafe {transmute(obj)}
    }
    #[allow(unused_variables)]
    pub fn from_bytes (frt: &CurrentContext<'a>, bytes: impl AsRef<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        debug_assert!(bytes.len() <= u32::MAX as usize);
        let ptr = FREByteArray {length: bytes.len() as u32, bytes: bytes.as_ptr() as FREBytes};
        let mut obj = std::ptr::null_mut();
        let r = unsafe {FRENewByteArray(transmute(&ptr), &mut obj)};
        debug_assert!(r.is_ok());
        debug_assert!(!obj.is_null());
        unsafe {transmute(obj)}
    }
    pub fn with <F, R> (self, f: F) -> R
    where F: FnOnce (&mut [u8]) -> R + Sync
    {
        let mut fptr = FREByteArray {length: 0, bytes: std::ptr::null_mut()};
        let result = unsafe {FREAcquireByteArray(self.as_ptr(), &mut fptr)};
        debug_assert!(result.is_ok());
        let bytes = unsafe {std::slice::from_raw_parts_mut(fptr.bytes, fptr.length as usize)};
        let r = f(bytes);
        let result = unsafe {FREReleaseByteArray(self.as_ptr())};
        debug_assert!(result.is_ok());
        r
    }
}
unsafe impl<'a> AsObject<'a> for ByteArray<'a> {const TYPE: Type = Type::ByteArray;}
unsafe impl<'a> TryAs<'a, ByteArray<'a>> for Object<'a> {}
impl From<ByteArray<'_>> for FREObject {fn from(value: ByteArray) -> Self {value.as_ptr()}}
impl<'a> From<ByteArray<'a>> for Object<'a> {fn from(value: ByteArray<'a>) -> Self {value.as_object()}}
impl<'a> TryFrom<Object<'a>> for ByteArray<'a> {type Error = Type; fn try_from (value: Object<'a>) -> Result<Self, Type> {value.try_as()}}
impl Display for ByteArray<'_> {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {Display::fmt(&self.as_object(), f)}}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct BitmapData <'a> (NonNullFREObject, PhantomData<&'a()>);
impl<'a> BitmapData<'a> {
    /// Using [`null`] inside the closure is meaningless and may lead to unintended FFI call ordering.
    pub fn with <F, R> (self, f: F) -> R
    where F: FnOnce (BitmapDataAdapter) -> R + Sync
    {
        let mut descriptor = FREBitmapData2::default();
        let result = unsafe {FREAcquireBitmapData2(self.as_ptr(), &mut descriptor)};
        assert!(result.is_ok());
        let r = f(unsafe {BitmapDataAdapter::new(self.as_ptr(), descriptor)});
        let result = unsafe {FREReleaseBitmapData(self.as_ptr())};
        debug_assert!(result.is_ok());
        r
    }
}
unsafe impl<'a> AsObject<'a> for BitmapData<'a> {const TYPE: Type = Type::BitmapData;}
unsafe impl<'a> TryAs<'a, BitmapData<'a>> for Object<'a> {}
impl From<BitmapData<'_>> for FREObject {fn from(value: BitmapData) -> Self {value.as_ptr()}}
impl<'a> From<BitmapData<'a>> for Object<'a> {fn from(value: BitmapData<'a>) -> Self {value.as_object()}}
impl<'a> TryFrom<Object<'a>> for BitmapData<'a> {type Error = Type; fn try_from (value: Object<'a>) -> Result<Self, Type> {value.try_as()}}
impl Display for BitmapData<'_> {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {Display::fmt(&self.as_object(), f)}}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Context3D <'a> (NonNullFREObject, PhantomData<&'a()>);
impl<'a> Context3D<'a> {
    pub fn value (self) -> FREHandle {
        let mut handle = FREHandle::default();
        let r = unsafe {FREGetNativeContext3DHandle(self.as_ptr(), &mut handle)};
        debug_assert!(r.is_ok());
        assert!(!handle.is_null());
        handle
    }
}
unsafe impl<'a> AsObject<'a> for Context3D<'a> {const TYPE: Type = Type::Context3D;}
// unsafe impl<'a> TryAs<'a, Context3D<'a>> for Object<'a> {}
impl From<Context3D<'_>> for FREObject {fn from(value: Context3D) -> Self {value.as_ptr()}}
impl<'a> From<Context3D<'a>> for Object<'a> {fn from(value: Context3D<'a>) -> Self {value.as_object()}}
// impl<'a> TryFrom<Object<'a>> for Context3D<'a> {type Error = Type; fn try_from (value: Object<'a>) -> Result<Self, Type> {value.try_as()}}
impl Display for Context3D<'_> {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {Display::fmt(&self.as_object(), f)}}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ErrorObject <'a> (NonNullFREObject, PhantomData<&'a()>);
impl<'a> ErrorObject<'a> {
    #[allow(non_snake_case)]
    pub fn get_errorID(self) -> i32 {
        const ERROR_ID: UCStr = unsafe {UCStr::from_literal_unchecked(c"errorID")};
        let value = self.get_property(ERROR_ID).unwrap().try_into().unwrap();
        value
    }
    const MESSAGE: UCStr = unsafe {UCStr::from_literal_unchecked(c"message")};
    pub fn get_message(self) -> Option<as3::String<'a>> {
        let value = self.get_property(Self::MESSAGE).unwrap().try_as().ok();
        value
    }
    pub fn set_message(self, value: Option<as3::String>) {
        let r = self.set_property(Self::MESSAGE, Object::from(value));
        debug_assert!(r.is_ok());
    }
    const NAME: UCStr = unsafe {UCStr::from_literal_unchecked(c"name")};
    pub fn get_name(self) -> Option<as3::String<'a>> {
        let value = self.get_property(Self::NAME).unwrap().try_as().ok();
        value
    }
    pub fn set_name(self, value: Option<as3::String>) {
        let r = self.set_property(Self::NAME, Object::from(value));
        debug_assert!(r.is_ok());
    }
    const CLASS: UCStr = unsafe {UCStr::from_literal_unchecked(c"Error")};
    pub fn new (frt: &CurrentContext<'a>, message: Option<&str>, id: i32) -> Self {
        let message = message.map(|s|as3::String::new(frt, s));
        let id = int::new(frt, id);
        let args = vec![message.into(), id.as_object()].into_boxed_slice();
        let err_obj= Object::new(frt, Self::CLASS, Some(args.as_ref())).unwrap();
        assert!(!err_obj.is_null());
        unsafe {transmute(err_obj)}
    }
}
unsafe impl<'a> AsObject<'a> for ErrorObject<'a> {const TYPE: Type = Type::Error;}
// unsafe impl<'a> TryAs<'a, ErrorObject<'a>> for Object<'a> {}
impl From<ErrorObject<'_>> for FREObject {fn from(value: ErrorObject) -> Self {value.as_ptr()}}
impl<'a> From<ErrorObject<'a>> for Object<'a> {fn from(value: ErrorObject<'a>) -> Self {value.as_object()}}
// impl<'a> TryFrom<Object<'a>> for ErrorObject<'a> {type Error = Type; fn try_from (value: Object<'a>) -> Result<Self, Type> {value.try_as()}}
impl Display for ErrorObject<'_> {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {Display::fmt(&self.as_object(), f)}}
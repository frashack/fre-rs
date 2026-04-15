use super::*;


#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct int <'a> (NonNullFREObject, PhantomData<&'a()>);
impl<'a> int<'a> {
    #[allow(unused_variables)]
    pub fn new (frt: &CurrentContext<'a>, value: i32) -> Self {
        let mut obj = std::ptr::null_mut();
        let r = unsafe {FRENewObjectFromInt32(value, &mut obj)};
        debug_assert!(r.is_ok());
        assert!(!obj.is_null());
        unsafe {transmute(obj)}
    }
    fn value(self) -> i32 {
        let mut val = i32::default();
        let r = unsafe {FREGetObjectAsInt32(self.as_ptr(), &mut val)};
        debug_assert!(r.is_ok());
        val
    }
}
unsafe impl<'a> AsObject<'a> for int<'a> {const TYPE: Type = Type::Named("int");}
impl From<int<'_>> for FREObject {fn from(value: int) -> Self {value.as_ptr()}}
impl<'a> From<int<'a>> for Object<'a> {fn from(value: int<'a>) -> Self {value.as_object()}}
impl Display for int<'_> {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {Display::fmt(&(self.value()), f)}}


#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct uint <'a> (NonNullFREObject, PhantomData<&'a()>);
impl<'a> uint<'a> {
    #[allow(unused_variables)]
    pub fn new (frt: &CurrentContext<'a>, value: u32) -> Self {
        let mut obj = std::ptr::null_mut();
        let r = unsafe {FRENewObjectFromUint32(value, &mut obj)};
        debug_assert!(r.is_ok());
        assert!(!obj.is_null());
        unsafe {transmute(obj)}
    }
    pub fn value (self) -> u32 {
        let mut val = u32::default();
        let r = unsafe {FREGetObjectAsUint32(self.as_ptr(), &mut val)};
        debug_assert!(r.is_ok());
        val
    }
}
unsafe impl<'a> AsObject<'a> for uint<'a> {const TYPE: Type = Type::Named("uint");}
impl From<uint<'_>> for FREObject {fn from(value: uint) -> Self {value.as_ptr()}}
impl<'a> From<uint<'a>> for Object<'a> {fn from(value: uint<'a>) -> Self {value.as_object()}}
impl Display for uint<'_> {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {Display::fmt(&(self.value()), f)}}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Number <'a> (NonNullFREObject, PhantomData<&'a()>);
impl<'a> Number<'a> {
    #[allow(unused_variables)]
    pub fn new (frt: &CurrentContext<'a>, value: f64) -> Self {
        let mut obj = std::ptr::null_mut();
        let r = unsafe {FRENewObjectFromDouble(value, &mut obj)};
        debug_assert!(r.is_ok());
        assert!(!obj.is_null());
        unsafe {transmute(obj)}
    }
    pub fn value (self) -> f64 {
        let mut val = f64::default();
        let r = unsafe {FREGetObjectAsDouble(self.as_ptr(), &mut val)};
        debug_assert!(r.is_ok());
        val
    }
}
unsafe impl<'a> AsObject<'a> for Number<'a> {const TYPE: Type = Type::Number;}
unsafe impl<'a> TryAs<'a, Number<'a>> for Object<'a> {}
impl From<Number<'_>> for FREObject {fn from(value: Number) -> Self {value.as_ptr()}}
impl<'a> From<Number<'a>> for Object<'a> {fn from(value: Number<'a>) -> Self {value.as_object()}}
impl<'a> TryFrom<Object<'a>> for Number<'a> {type Error = Type; fn try_from (value: Object<'a>) -> Result<Self, Type> {value.try_as()}}
impl Display for Number<'_> {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {Display::fmt(&self.as_object(), f)}}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Boolean <'a> (NonNullFREObject, PhantomData<&'a()>);
impl<'a> Boolean<'a> {
    #[allow(unused_variables)]
    pub fn new (frt: &CurrentContext<'a>, value: bool) -> Self {
        let value = if value {1} else {0};
        let mut obj = std::ptr::null_mut();
        let r = unsafe {FRENewObjectFromBool(value, &mut obj)};
        debug_assert!(r.is_ok());
        assert!(!obj.is_null());
        unsafe {transmute(obj)}
    }
    pub fn value (self) -> bool {
        let mut val = u32::default();
        let r = unsafe {FREGetObjectAsBool(self.as_ptr(), &mut val)};
        debug_assert!(r.is_ok());
        val != 0
    }
}
unsafe impl<'a> AsObject<'a> for Boolean<'a> {const TYPE: Type = Type::Boolean;}
unsafe impl<'a> TryAs<'a, Boolean<'a>> for Object<'a> {}
impl From<Boolean<'_>> for FREObject {fn from(value: Boolean) -> Self {value.as_ptr()}}
impl<'a> From<Boolean<'a>> for Object<'a> {fn from(value: Boolean<'a>) -> Self {value.as_object()}}
impl<'a> TryFrom<Object<'a>> for Boolean<'a> {type Error = Type; fn try_from (value: Object<'a>) -> Result<Self, Type> {value.try_as()}}
impl Display for Boolean<'_> {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {Display::fmt(&(self.value()), f)}}


/// Represents an ActionScript `String` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct StringObject <'a> (NonNullFREObject, PhantomData<&'a()>);
impl<'a> StringObject<'a> {
    #[allow(unused_variables)]
    pub fn new (frt: &CurrentContext<'a>, value: &str) -> Self {
        let value = value.as_bytes();
        debug_assert!(value.len() <= u32::MAX as usize);
        let mut obj = std::ptr::null_mut();
        let r = unsafe {FRENewObjectFromUTF8(value.len() as u32, value.as_ptr(), &mut obj)};
        debug_assert!(r.is_ok());
        assert!(!obj.is_null());
        unsafe {transmute(obj)}
    }
    pub fn value (self) -> &'a str {
        let mut len = u32::default();
        let mut ptr = std::ptr::null();
        let r = unsafe {FREGetObjectAsUTF8(self.as_ptr(), &mut len, &mut ptr)};
        debug_assert!(r.is_ok());
        let bytes = unsafe {std::slice::from_raw_parts(ptr, len as usize)};
        unsafe {str::from_utf8_unchecked(bytes)}
    }
}
unsafe impl<'a> AsObject<'a> for StringObject<'a> {const TYPE: Type = Type::String;}
unsafe impl<'a> TryAs<'a, StringObject<'a>> for Object<'a> {}
impl From<StringObject<'_>> for FREObject {fn from(value: StringObject) -> Self {value.as_ptr()}}
impl<'a> From<StringObject<'a>> for Object<'a> {fn from(value: StringObject<'a>) -> Self {value.as_object()}}
impl<'a> TryFrom<Object<'a>> for StringObject<'a> {type Error = Type; fn try_from (value: Object<'a>) -> Result<Self, Type> {value.try_as()}}
impl Display for StringObject<'_> {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {Display::fmt(self.value(), f)}}
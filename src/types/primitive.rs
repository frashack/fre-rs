use super::*;
use std::string::String as StdString;


crate::class! {
    /// A reference to the AS3 object `int`.
    /// 
    /// Some methods are not yet implemented.
    /// 
    #[allow(non_camel_case_types)]
    int !PartialEq
}
impl PartialEq for int<'_> {fn eq(&self, other: &Self) -> bool {self.value() == other.value()}}
impl PartialEq<i32> for int<'_> {fn eq(&self, other: &i32) -> bool {self.value() == *other}}
impl PartialEq<int<'_>> for i32 {fn eq(&self, other: &int<'_>) -> bool {*self == other.value()}}
impl Eq for int<'_> {}
impl<'a> int<'a> {
    pub fn new (_: &CurrentContext<'a>, value: i32) -> Self {
        let mut object = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FRENewObjectFromInt32(value, object.as_mut_ptr())};
        debug_assert!(r.is_ok());
        let object = unsafe {object.assume_init()};
        assert!(!object.is_null());
        unsafe {transmute(object)}
    }
    pub fn value(self) -> i32 {
        let mut value = MaybeUninit::<i32>::uninit();
        let r = unsafe {FREGetObjectAsInt32(self.as_ptr(), value.as_mut_ptr())};
        debug_assert!(r.is_ok());
        unsafe {value.assume_init()}
    }
}


crate::class! {
    /// A reference to the AS3 object `uint`.
    /// 
    /// Some methods are not yet implemented.
    /// 
    #[allow(non_camel_case_types)]
    uint !PartialEq
}
impl PartialEq for uint<'_> {fn eq(&self, other: &Self) -> bool {self.value() == other.value()}}
impl PartialEq<u32> for uint<'_> {fn eq(&self, other: &u32) -> bool {self.value() == *other}}
impl PartialEq<uint<'_>> for u32 {fn eq(&self, other: &uint<'_>) -> bool {*self == other.value()}}
impl Eq for uint<'_> {}
impl<'a> uint<'a> {
    pub fn new (_: &CurrentContext<'a>, value: u32) -> Self {
        let mut object = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FRENewObjectFromUint32(value, object.as_mut_ptr())};
        debug_assert!(r.is_ok());
        let object = unsafe {object.assume_init()};
        assert!(!object.is_null());
        unsafe {transmute(object)}
    }
    pub fn value (self) -> u32 {
        let mut value = MaybeUninit::<u32>::uninit();
        let r = unsafe {FREGetObjectAsUint32(self.as_ptr(), value.as_mut_ptr())};
        debug_assert!(r.is_ok());
        unsafe {value.assume_init()}
    }
}


crate::class! {@Typeof
    /// A reference to the AS3 object `Number`.
    /// 
    /// Some methods are not yet implemented.
    /// 
    Number !PartialEq
}
impl PartialEq for Number<'_> {fn eq(&self, other: &Self) -> bool {self.value() == other.value()}}
impl PartialEq<f64> for Number<'_> {fn eq(&self, other: &f64) -> bool {self.value() == *other}}
impl PartialEq<Number<'_>> for f64 {fn eq(&self, other: &Number<'_>) -> bool {*self == other.value()}}
impl<'a> Number<'a> {
    pub fn new (_: &CurrentContext<'a>, value: f64) -> Self {
        let mut object = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FRENewObjectFromDouble(value, object.as_mut_ptr())};
        debug_assert!(r.is_ok());
        let object = unsafe {object.assume_init()};
        assert!(!object.is_null());
        unsafe {transmute(object)}
    }
    pub fn value (self) -> f64 {
        let mut value = MaybeUninit::<f64>::uninit();
        let r = unsafe {FREGetObjectAsDouble(self.as_ptr(), value.as_mut_ptr())};
        debug_assert!(r.is_ok());
        unsafe {value.assume_init()}
    }
}


crate::class! {@Typeof
    /// A reference to the AS3 object `Boolean`.
    /// 
    Boolean !PartialEq
}
impl PartialEq for Boolean<'_> {fn eq(&self, other: &Self) -> bool {self.value() == other.value()}}
impl PartialEq<bool> for Boolean<'_> {fn eq(&self, other: &bool) -> bool {self.value() == *other}}
impl PartialEq<Boolean<'_>> for bool {fn eq(&self, other: &Boolean<'_>) -> bool {*self == other.value()}}
impl Eq for Boolean<'_> {}
impl<'a> Boolean<'a> {
    pub fn new (_: &CurrentContext<'a>, value: bool) -> Self {
        let value = if value {1} else {0};
        let mut object = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FRENewObjectFromBool(value, object.as_mut_ptr())};
        debug_assert!(r.is_ok());
        let object = unsafe {object.assume_init()};
        assert!(!object.is_null());
        unsafe {transmute(object)}
    }
    pub fn value (self) -> bool {
        let mut value = MaybeUninit::<u32>::uninit();
        let r = unsafe {FREGetObjectAsBool(self.as_ptr(), value.as_mut_ptr())};
        debug_assert!(r.is_ok());
        let value: u32 = unsafe {value.assume_init()};
        value != 0
    }
}


//
// TODO: Implement methods of `String`.
//
crate::class! {@Typeof
    /// A reference to the AS3 object `String`.
    /// 
    /// Some methods are not yet implemented.
    /// 
    String !PartialEq
}
impl PartialEq for String<'_> {fn eq(&self, other: &Self) -> bool {self.value() == other.value()}}
impl PartialEq<str> for String<'_> {fn eq(&self, other: &str) -> bool {self.value() == other}}
impl PartialEq<StdString> for String<'_> {fn eq(&self, other: &StdString) -> bool {self.value() == other.as_str()}}
impl PartialEq<Cow<'_, str>> for String<'_> {fn eq(&self, other: &Cow<'_, str>) -> bool {self.value() == other.as_ref()}}
impl PartialEq<Box<str>> for String<'_> {fn eq(&self, other: &Box<str>) -> bool {self.value() == other.as_ref()}}
impl PartialEq<Rc<str>> for String<'_> {fn eq(&self, other: &Rc<str>) -> bool {self.value() == other.as_ref()}}
impl PartialEq<Arc<str>> for String<'_> {fn eq(&self, other: &Arc<str>) -> bool {self.value() == other.as_ref()}}
impl PartialEq<UCStr> for String<'_> {fn eq(&self, other: &UCStr) -> bool {self.value() == other.as_str()}}
impl PartialEq<String<'_>> for str {fn eq(&self, other: &String<'_>) -> bool {self == other.value()}}
impl PartialEq<String<'_>> for StdString {fn eq(&self, other: &String<'_>) -> bool {PartialEq::eq(other, self)}}
impl PartialEq<String<'_>> for Cow<'_, str> {fn eq(&self, other: &String<'_>) -> bool {PartialEq::eq(other, self)}}
impl PartialEq<String<'_>> for Box<str> {fn eq(&self, other: &String<'_>) -> bool {PartialEq::eq(other, self)}}
impl PartialEq<String<'_>> for Rc<str> {fn eq(&self, other: &String<'_>) -> bool {PartialEq::eq(other, self)}}
impl PartialEq<String<'_>> for Arc<str> {fn eq(&self, other: &String<'_>) -> bool {PartialEq::eq(other, self)}}
impl PartialEq<String<'_>> for UCStr {fn eq(&self, other: &String<'_>) -> bool {PartialEq::eq(other, self)}}
impl Eq for String<'_> {}
impl<'a> String<'a> {
    pub fn new (_: &CurrentContext<'a>, value: &str) -> Self {
        let value = value.as_bytes();
        debug_assert!(value.len() <= u32::MAX as usize);
        let mut object = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FRENewObjectFromUTF8(value.len() as u32, value.as_ptr(), object.as_mut_ptr())};
        debug_assert!(r.is_ok());
        let object = unsafe {object.assume_init()};
        assert!(!object.is_null());
        unsafe {transmute(object)}
    }
    pub fn value (self) -> &'a str {
        let mut len = MaybeUninit::<u32>::uninit();
        let mut ptr = MaybeUninit::<FREStr>::uninit();
        let r = unsafe {FREGetObjectAsUTF8(self.as_ptr(), len.as_mut_ptr(), ptr.as_mut_ptr())};
        debug_assert!(r.is_ok());
        let len = unsafe {len.assume_init()};
        let ptr = unsafe {ptr.assume_init()};
        assert!(!ptr.is_null());
        let bytes = unsafe {std::slice::from_raw_parts(ptr, len as usize)};
        unsafe {str::from_utf8_unchecked(bytes)}
    }
}
impl<'a> AsRef<str> for String<'a> {fn as_ref(&self) -> &'a str {self.value()}}


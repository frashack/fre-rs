//! 
//! Validated types for safe interaction with the C API and the Flash Runtime,
//! ensuring data passed across the FFI boundary is well-formed and valid.
//! 


use super::*;


pub type NonNullFREData = NonNull<c_void>;
pub type NonNullHandle = NonNull<c_void>;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct NonNegativeInt(i32);
impl NonNegativeInt {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(i32::MAX);
    pub fn new (value: i32) -> Option<Self> {
        if value >= 0 {
            Some(Self(value))
        } else {
            None
        }
    }
    pub fn get (self) -> i32 {self.0}
}
impl Default for NonNegativeInt {
    fn default() -> Self {Self::MIN}
}


/// A UTF-8 string stored as a NUL-terminated [`CStr`].
/// 
/// Use the [`ucstringify!`] macro to construct [`UCStr`] constants.
/// 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UCStr(UCStrValue);
#[derive(Debug, Clone, PartialEq, Eq)]
enum UCStrValue {
    Static(&'static CStr),
    Heap(Arc<CStr>),
}
impl UCStr {
    pub const fn from_literal (literal: &'static CStr) -> Result<Self, Utf8Error> {
        match literal.to_str() {
            Ok(_) => Ok(Self(UCStrValue::Static(literal))),
            Err(e) => Err(e),
        }
    }
    pub const unsafe fn from_literal_unchecked (literal: &'static CStr) -> Self {
        debug_assert!(literal.to_str().is_ok());
        Self(UCStrValue::Static(literal))
    }
    #[inline]
    pub fn as_str (&self) -> &str {
        let s = match self.0 {
            UCStrValue::Static(s) => s.to_bytes(),
            UCStrValue::Heap(ref s) => s.to_bytes(),
        };
        unsafe {str::from_utf8_unchecked(s)}
    }
    pub fn as_c_str (&self) -> &CStr {
        match self.0 {
            UCStrValue::Static(s) => s,
            UCStrValue::Heap(ref s) => s.as_ref(),
        }
    }
    /// # Borrow
    pub fn as_ptr (&self) -> FREStr {
        match self.0 {
            UCStrValue::Static(s) => s.as_ptr() as FREStr,
            UCStrValue::Heap(ref s) => s.as_ptr() as FREStr,
        }
    }
    pub fn to_c_string (&self) -> CString {
        match self.0 {
            UCStrValue::Static(s) => s.to_owned(),
            UCStrValue::Heap(ref s) => s.as_ref().to_owned(),
        }
    }
}
impl From<UCStr> for CString {
    fn from(value: UCStr) -> Self {value.to_c_string()}
}
impl From<UCStr> for String {
    fn from(value: UCStr) -> Self {value.to_string()}
}
impl TryFrom<String> for UCStr {
    type Error = NulError;
    fn try_from(value: String) -> Result<Self, NulError> {
        CString::new(value).map(|s|{
            let s = s.into();
            Self(UCStrValue::Heap(s))
        })
    }
}
impl TryFrom<&str> for UCStr {
    type Error = NulError;
    fn try_from(value: &str) -> Result<Self, NulError> {
        CString::new(value).map(|s|{
            let s = s.into();
            Self(UCStrValue::Heap(s))
        })
    }
}
impl TryFrom<CString> for UCStr {
    type Error = Utf8Error;
    fn try_from(value: CString) -> Result<Self, Utf8Error> {
        value.to_str()?;
        let s = value.into();
        Ok(Self(UCStrValue::Heap(s)))
    }
}
impl TryFrom<&CStr> for UCStr {
    type Error = Utf8Error;
    fn try_from(value: &CStr) -> Result<Self, Utf8Error> {
        value.to_str().map(|_|{
            let s = value.to_owned().into();
            Self(UCStrValue::Heap(s))
        })
    }
}
impl PartialEq<str> for UCStr {
    #[inline]
    fn eq(&self, other: &str) -> bool { self.as_str() == other }
}
impl PartialEq<UCStr> for str {
    #[inline]
    fn eq(&self, other: &UCStr) -> bool { self == other.as_str() }
}
impl PartialOrd for UCStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { self.as_str().partial_cmp(other.as_str()) }
}
impl Ord for UCStr {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.as_str().cmp(other.as_str()) }
}
impl std::hash::Hash for UCStr {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.as_str().hash(state) }
}
impl AsRef<str> for UCStr {
    fn as_ref(&self) -> &str { self.as_str() }
}
impl AsRef<CStr> for UCStr {
    fn as_ref(&self) -> &CStr { self.as_c_str() }
}
impl std::borrow::Borrow<str> for UCStr {
    #[inline]
    fn borrow(&self) -> &str { self.as_str() }
}
impl Display for UCStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Display::fmt(self.as_str(), f) }
}
impl Default for UCStr {
    fn default() -> Self {Self(UCStrValue::Static(c""))}
}


pub trait ToUcstrLossy {
    fn to_ucstr_lossy(&self) -> UCStr;
}
macro_rules! impl_to_ucstr_lossy {
    {// #0
        ref $self:ident: $ty:ty
        $body:block
        $(<$($gps:tt)+)? 
    } => {
        impl $(<$($gps)+)? ToUcstrLossy for & $ty {fn to_ucstr_lossy(&self) -> UCStr {<$ty as ToUcstrLossy>::to_ucstr_lossy(*self)}}
        impl $(<$($gps)+)? ToUcstrLossy for &mut $ty {fn to_ucstr_lossy(&self) -> UCStr {<$ty as ToUcstrLossy>::to_ucstr_lossy(*self)}}
        impl $(<$($gps)+)? ToUcstrLossy for $ty {fn to_ucstr_lossy(&self) -> UCStr {let $self = self; $body}}
    };
}
impl_to_ucstr_lossy! {ref this: UCStr {
    this.clone()
}}
impl_to_ucstr_lossy! {ref this: str {
    this.replace('\0', "�")
        .try_into()
        .unwrap()
}}
impl_to_ucstr_lossy! {ref this: String {
    this.as_str().to_ucstr_lossy()
}}
impl_to_ucstr_lossy! {ref this: CStr {
    let s = this.to_string_lossy();
    let bytes = s.as_bytes();
    let mut v = Vec::with_capacity(bytes.len()+1);
    v.extend_from_slice(bytes);
    v.push(0);
    let s = unsafe {CString::from_vec_unchecked(v)};
    UCStr(UCStrValue::Heap(s.into()))
}}
impl_to_ucstr_lossy! {ref this: CString {
    this.as_c_str().to_ucstr_lossy()
}}
impl<'a, O: AsObject<'a>> ToUcstrLossy for O {
    fn to_ucstr_lossy(&self) -> UCStr {
        self.to_string()
            .as_str()
            .to_ucstr_lossy()
    }
}
impl<'a> ToUcstrLossy for & as3::Object<'a> {fn to_ucstr_lossy(&self) -> UCStr {<as3::Object as ToUcstrLossy>::to_ucstr_lossy(*self)}}
impl<'a> ToUcstrLossy for &mut as3::Object<'a> {fn to_ucstr_lossy(&self) -> UCStr {<as3::Object as ToUcstrLossy>::to_ucstr_lossy(*self)}}
// crate::class! (...);
impl_to_ucstr_lossy! {ref this: Option<O> {
    if let Some(object) = this {
        object.to_ucstr_lossy()
    } else {crate::ucstringify!(null)}
} <'a, O: AsObject<'a>> }
impl_to_ucstr_lossy! {ref this: [O] {
    this.iter()
        .map(|object|object.to_string())
        .collect::<Vec<String>>()
        .join(", ")
        .as_str()
        .to_ucstr_lossy()
} <'a, O: AsObject<'a>> }
impl_to_ucstr_lossy! {ref this: [Option<O>] {
    this.iter()
        .map(|object|{
            if let Some(object) = object {
                object.to_string()
            } else {String::from("null")}
        })
        .collect::<Vec<String>>()
        .join(", ")
        .as_str()
        .to_ucstr_lossy()
} <'a, O: AsObject<'a>> }
impl_to_ucstr_lossy! {ref this: [O; LEN] {
    this.as_slice().to_ucstr_lossy()
} <'a, O: AsObject<'a>,
const LEN: usize> }
impl_to_ucstr_lossy! {ref this: [Option<O>; LEN] {
    this.as_slice().to_ucstr_lossy()
} <'a, O: AsObject<'a>,
const LEN: usize> }
impl_to_ucstr_lossy! {ref this: Vec<O> {
    this.as_slice().to_ucstr_lossy()
} <'a, O: AsObject<'a>> }
impl_to_ucstr_lossy! {ref this: Vec<Option<O>> {
    this.as_slice().to_ucstr_lossy()
} <'a, O: AsObject<'a>> }


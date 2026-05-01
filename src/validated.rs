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
impl<T: ?Sized> ToUcstrLossy for &T
where T: ToUcstrLossy
{fn to_ucstr_lossy(&self) -> UCStr {T::to_ucstr_lossy(self)}}
impl<T: ?Sized> ToUcstrLossy for &mut T
where for<'a> &'a T: ToUcstrLossy
{fn to_ucstr_lossy(&self) -> UCStr {((*self) as &T).to_ucstr_lossy()}}
//
//
impl ToUcstrLossy for UCStr {fn to_ucstr_lossy(&self) -> UCStr {self.clone()}}
impl ToUcstrLossy for str {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.replace('\0', "�");
    let buf = unsafe {CString::from_vec_unchecked(buf.into_bytes())};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for String {fn to_ucstr_lossy(&self) -> UCStr {self.as_str().to_ucstr_lossy()}}
impl ToUcstrLossy for CStr {fn to_ucstr_lossy(&self) -> UCStr {
    let s = self.to_string_lossy();
    let bytes = s.as_bytes();
    let mut v = Vec::with_capacity(bytes.len()+1);
    v.extend_from_slice(bytes);
    v.push(0);
    let s = unsafe {CString::from_vec_with_nul_unchecked(v)};
    UCStr(UCStrValue::Heap(s.into()))
}}
impl ToUcstrLossy for CString {fn to_ucstr_lossy(&self) -> UCStr {self.as_c_str().to_ucstr_lossy()}}
impl ToUcstrLossy for Object<'_> {fn to_ucstr_lossy(&self) -> UCStr {self.to_string().to_ucstr_lossy()}}
impl ToUcstrLossy for NonNullObject<'_> {fn to_ucstr_lossy(&self) -> UCStr {self.as_object().to_ucstr_lossy()}}
// crate::class! (...) => impl ToUcstrLossy for ...
impl ToUcstrLossy for () {fn to_ucstr_lossy(&self) -> UCStr {crate::ucstringify!(())}}
impl ToUcstrLossy for char {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = format!("'{}'(U+{:08X})\0", self.escape_default(),*self as u32);
    let buf = unsafe {CString::from_vec_with_nul_unchecked(buf.into_bytes())};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for bool {fn to_ucstr_lossy(&self) -> UCStr {
    if *self {crate::ucstringify!(true)} else {crate::ucstringify!(false)}
}}
impl ToUcstrLossy for i8 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for i16 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for i32 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for i64 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for i128 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for isize {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for u8 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for u16 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for u32 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for u64 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for u128 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for usize {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for f32 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl ToUcstrLossy for f64 {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = self.to_string().into_bytes();
    let buf = unsafe {CString::from_vec_unchecked(buf)};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl<T> ToUcstrLossy for *const T {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = format!("*const({:p})\0", self);
    let buf = unsafe {CString::from_vec_with_nul_unchecked(buf.into_bytes())};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl<T> ToUcstrLossy for *mut T {fn to_ucstr_lossy(&self) -> UCStr {
    let buf = format!("*mut({:p})\0", self);
    let buf = unsafe {CString::from_vec_with_nul_unchecked(buf.into_bytes())};
    UCStr(UCStrValue::Heap(buf.into()))
}}
//
//
impl<T: ToUcstrLossy> ToUcstrLossy for Option<T> {fn to_ucstr_lossy(&self) -> UCStr {
    if let Some(inner) = self {
        let inner = inner.to_ucstr_lossy();
        let mut buf = String::with_capacity(4 + 1 + inner.as_str().len() + 1 + 1);
        buf.push_str("Some(");
        buf.push_str(inner.as_str());
        buf.push_str(")\0");
        let buf = unsafe {CString::from_vec_with_nul_unchecked(buf.into_bytes())};
        UCStr(UCStrValue::Heap(buf.into()))
    } else {crate::ucstringify!(None)}
}}
impl<T: ToUcstrLossy> ToUcstrLossy for [T] {fn to_ucstr_lossy(&self) -> UCStr {
    const SEPARATOR: &str = ", ";
    let elems = self.iter()
        .map(|i|i.to_ucstr_lossy())
        .collect::<Box<[UCStr]>>();
    let len = elems.iter()
        .map(|s|s.as_str().len())
        .sum::<usize>()
        + (elems.len().saturating_sub(1))*SEPARATOR.len()
        + 2
        + 1;
    let mut buf = String::with_capacity(len);
    buf.push('[');
    for (i, s) in elems.iter().enumerate() {
        if i != 0 {buf.push_str(SEPARATOR);}
        buf.push_str(s.as_str());
    }
    buf.push(']');
    buf.push('\0');
    let buf = unsafe {CString::from_vec_with_nul_unchecked(buf.into_bytes())};
    UCStr(UCStrValue::Heap(buf.into()))
}}
impl<T: ToUcstrLossy, const LEN: usize> ToUcstrLossy for [T; LEN] {fn to_ucstr_lossy(&self) -> UCStr {self.as_slice().to_ucstr_lossy()}}
impl<T: ToUcstrLossy> ToUcstrLossy for Vec<T> {fn to_ucstr_lossy(&self) -> UCStr {self.as_slice().to_ucstr_lossy()} }
impl<T: ToUcstrLossy>
ToUcstrLossy for (T,) {fn to_ucstr_lossy(&self) -> UCStr {
    let s = self.0.to_ucstr_lossy();
    let mut buf = String::with_capacity(s.as_str().len() + 3);
    buf.push('(');
    buf.push_str(s.as_str());
    buf.push(')');
    buf.push('\0');
    let buf = unsafe {CString::from_vec_with_nul_unchecked(buf.into_bytes())};
    UCStr(UCStrValue::Heap(buf.into()))
}}
macro_rules! tuple_to_ucstr {
    ($elements:expr_2021) => {{
        const SEPARATOR: &'static str = ", ";
        let len = $elements.iter()
            .map(|s|s.as_str().len())
            .sum::<usize>()
            + ($elements.len().saturating_sub(1))*SEPARATOR.len()
            + 2
            + 1;
        let mut buf = String::with_capacity(len);
        buf.push('(');
        for (i, s) in $elements.iter().enumerate() {
            if i != 0 {buf.push_str(SEPARATOR);}
            buf.push_str(s.as_str());
        }
        buf.push(')');
        buf.push('\0');
        let buf = unsafe {CString::from_vec_with_nul_unchecked(buf.into_bytes())};
        UCStr(UCStrValue::Heap(buf.into()))
    }};
}
impl<T: ToUcstrLossy, U: ToUcstrLossy>
ToUcstrLossy for (T, U) {fn to_ucstr_lossy(&self) -> UCStr {
    let elements =  [self.0.to_ucstr_lossy(), self.1.to_ucstr_lossy()];
    tuple_to_ucstr! (elements)
}}
impl<T: ToUcstrLossy, U: ToUcstrLossy, V: ToUcstrLossy>
ToUcstrLossy for (T, U, V) {fn to_ucstr_lossy(&self) -> UCStr {
    let elements = [self.0.to_ucstr_lossy(), self.1.to_ucstr_lossy(), self.2.to_ucstr_lossy()];
    tuple_to_ucstr! (elements)
}}
impl<T: ToUcstrLossy, U: ToUcstrLossy, V: ToUcstrLossy, W: ToUcstrLossy>
ToUcstrLossy for (T, U, V, W) {fn to_ucstr_lossy(&self) -> UCStr {
    let elements = [self.0.to_ucstr_lossy(), self.1.to_ucstr_lossy(), self.2.to_ucstr_lossy(), self.3.to_ucstr_lossy()];
    tuple_to_ucstr! (elements)
}}
impl<T: ToUcstrLossy, U: ToUcstrLossy, V: ToUcstrLossy, W: ToUcstrLossy, X: ToUcstrLossy>
ToUcstrLossy for (T, U, V, W, X) {fn to_ucstr_lossy(&self) -> UCStr {
    let elements = [self.0.to_ucstr_lossy(), self.1.to_ucstr_lossy(), self.2.to_ucstr_lossy(), self.3.to_ucstr_lossy(), self.4.to_ucstr_lossy()];
    tuple_to_ucstr! (elements)
}}
impl<T: ToUcstrLossy, U: ToUcstrLossy, V: ToUcstrLossy, W: ToUcstrLossy, X: ToUcstrLossy, Y: ToUcstrLossy>
ToUcstrLossy for (T, U, V, W, X, Y) {fn to_ucstr_lossy(&self) -> UCStr {
    let elements = [self.0.to_ucstr_lossy(), self.1.to_ucstr_lossy(), self.2.to_ucstr_lossy(), self.3.to_ucstr_lossy(), self.4.to_ucstr_lossy(), self.5.to_ucstr_lossy()];
    tuple_to_ucstr! (elements)
}}
impl<T: ToUcstrLossy, U: ToUcstrLossy, V: ToUcstrLossy, W: ToUcstrLossy, X: ToUcstrLossy, Y: ToUcstrLossy, Z: ToUcstrLossy>
ToUcstrLossy for (T, U, V, W, X, Y, Z) {fn to_ucstr_lossy(&self) -> UCStr {
    let elements = [self.0.to_ucstr_lossy(), self.1.to_ucstr_lossy(), self.2.to_ucstr_lossy(), self.3.to_ucstr_lossy(), self.4.to_ucstr_lossy(), self.5.to_ucstr_lossy(), self.6.to_ucstr_lossy()];
    tuple_to_ucstr! (elements)
}}
impl<T: ToUcstrLossy, U: ToUcstrLossy, V: ToUcstrLossy, W: ToUcstrLossy, X: ToUcstrLossy, Y: ToUcstrLossy, Z: ToUcstrLossy, A: ToUcstrLossy>
ToUcstrLossy for (T, U, V, W, X, Y, Z, A) {fn to_ucstr_lossy(&self) -> UCStr {
    let elements = [self.0.to_ucstr_lossy(), self.1.to_ucstr_lossy(), self.2.to_ucstr_lossy(), self.3.to_ucstr_lossy(), self.4.to_ucstr_lossy(), self.5.to_ucstr_lossy(), self.6.to_ucstr_lossy(), self.7.to_ucstr_lossy()];
    tuple_to_ucstr! (elements)
}}


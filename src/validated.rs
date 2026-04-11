use super::*;


pub type NonNullFREData = NonNull<c_void>;
pub type NonNullFREObject = NonNull<c_void>;


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
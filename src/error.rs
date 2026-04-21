use super::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalError<'a> {
    C(FfiError),
    ActionScript(ActionScriptError<'a>),
}
impl<'a> ExternalError<'a> {
    /// Attempts to convert a [`FREResult`] into [`ExternalError`].
    ///
    /// If the result represents an ActionScript error and `thrown` is [`Some`],
    /// the provided object is used as the error value.
    ///
    /// If `thrown` is [`None`], the thrown object is ignored or assumed unavailable.
    ///
    /// When an ActionScript throw occurs, the provided object is assumed to be a valid [`FREObject`].
    /// However, ActionScript may throw `null`.
    /// 
    pub fn try_from (result: FREResult, thrown: Option<as3::Object<'a>>) -> Result<Self, ()> {
        let r = result.try_into();
        if let (Ok(Self::ActionScript(_)), Some(obj)) = (r, thrown) {
            return Ok(ActionScriptError(obj).into());
        }
        r
    }
}
impl From<FfiError> for ExternalError<'static> {
    fn from(value: FfiError) -> Self {Self::C(value)}
}
impl<'a> From<ActionScriptError<'a>> for ExternalError<'a> {
    fn from(value: ActionScriptError<'a>) -> Self {Self::ActionScript(value)}
}
impl TryFrom<FREResult> for ExternalError<'static> {
    type Error = ();
    fn try_from(value: FREResult) -> Result<Self, ()> {
        FfiError::try_from(value).map(|e|{
            if let FfiError::UnexpectedResult(v)=e && v==FREResult::FRE_ACTIONSCRIPT_ERROR {
                ActionScriptError::IGNORED.into()
            }else{e.into()}
        })
    }
}
impl Display for ExternalError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::C(e) => Display::fmt(e, f),
            Self::ActionScript(e) => Display::fmt(e, f),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiError {
    NoSuchName,
    InvalidObject,
    TypeMismatch,
    InvalidArgument,
    ReadOnly,
    WrongThread,
    IllegalState,
    InsufficientMemory,
    UnexpectedResult(FREResult),
}
impl Display for FfiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const PREFIX: &str = "[FfiError]";
        match *self {
            FfiError::NoSuchName => write!(f, "{PREFIX} The name of a class, property, or method passed as a parameter does not match an ActionScript class name, property, or method."),
            FfiError::InvalidObject => write!(f, "{PREFIX} An FREObject parameter is invalid. For examples of invalid FREObject variables, see 'FREObject validity'. https://help.adobe.com/en_US/air/extensions/WS460ee381960520ad-866f9c112aa6e1ad46-7ff9.html"),
            FfiError::TypeMismatch => write!(f, "{PREFIX} An FREObject parameter does not represent an object of the ActionScript class expected by the called function."),
            FfiError::InvalidArgument => write!(f, "{PREFIX} A pointer parameter is `NULL`."),
            FfiError::ReadOnly => write!(f, "{PREFIX} The function attempted to modify a read-only property of an ActionScript object."),
            FfiError::WrongThread => write!(f, "{PREFIX} The method was called from a thread other than the one on which the runtime has an outstanding call to a native extension function."),
            FfiError::IllegalState => write!(f, "{PREFIX} A call was made to a native extensions C API function when the extension context was in an illegal state for that call. This return value occurs in the following situation. The context has acquired access to an ActionScript BitmapData or ByteArray class object. With one exception, the context can call no other C API functions until it releases the BitmapData or ByteArray object. The one exception is that the context can call `FREInvalidateBitmapDataRect()` after calling `FREAcquireBitmapData()` or `FREAcquireBitmapData2()`."),
            FfiError::InsufficientMemory => write!(f, "{PREFIX} The runtime could not allocate enough memory to change the size of an Array or Vector object."),
            FfiError::UnexpectedResult(code) => write!(f, "{PREFIX} Unexpected FREResult. ({code:#08X})"),
        }
    }
}
impl Error for FfiError {}
impl TryFrom<FREResult> for FfiError {
    type Error = ();

    /// Converts a [`FREResult`] into [`FfiError`].
    ///
    /// Assumes `value` is **not** [`FREResult::FRE_ACTIONSCRIPT_ERROR`].
    /// If it is, it will be treated as an unexpected result.
    /// 
    fn try_from(value: FREResult) -> Result<Self, ()> {
        Ok(match value {
            FREResult::FRE_OK => return Err(()),
            FREResult::FRE_INVALID_OBJECT => Self::InvalidObject,
            FREResult::FRE_TYPE_MISMATCH => Self::TypeMismatch,
            FREResult::FRE_INVALID_ARGUMENT => Self::InvalidArgument,
            FREResult::FRE_READ_ONLY => Self::ReadOnly,
            FREResult::FRE_WRONG_THREAD => Self::WrongThread,
            FREResult::FRE_ILLEGAL_STATE => Self::IllegalState,
            FREResult::FRE_INSUFFICIENT_MEMORY => Self::InsufficientMemory,
            _ => Self::UnexpectedResult(value),
        })
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ActionScriptError<'a> (as3::Object<'a>);
impl<'a> ActionScriptError<'a> {
    const IGNORED: ActionScriptError<'static> = ActionScriptError(as3::null);

    /// May be `null` if the caller does not want to receive this handle.
    pub fn thrown (self) -> as3::Object<'a> {self.0}
}
impl Display for ActionScriptError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ActionScriptError] An ActionScript error occurred, and an exception was thrown. The C API functions that can result in this error allow you to specify an FREObject to receive information about the exception.")
    }
}
impl Error for ActionScriptError<'_> {}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextError {
    InvalidContext,
    NullRegistry,
    InvalidRegistry,
    MethodsNotRegistered,
    MethodNotFound,
    FfiCallFailed(FfiError),
    BorrowRegistryConflict,
}
impl Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ContextError] {self:?}")
    }
}
impl Error for ContextError {}
impl From<FfiError> for ContextError {
    fn from(value: FfiError) -> Self {Self::FfiCallFailed(value)}
}
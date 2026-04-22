//! 
//! Asynchronous event dispatching utilities for cross-thread scenarios,
//! providing a safe interface for interacting with the Flash Runtime.
//! 


use super::*;


/// A handle for dispatching events that may become invalid under the same
/// conditions as [`Context`].
///
/// In asynchronous scenarios, its validity is difficult to predict.
/// Therefore, all APIs on this type are safe to call, but their execution
/// results cannot be guaranteed or reliably observed.
/// 
#[derive(Debug, Clone, Copy)]
pub struct EventDispatcher (pub(crate) crate::context::ContextHandle);
impl EventDispatcher {

    /// Attempts to dispatch a event to the associated `ExtensionContext`.
    ///
    /// If the underlying context has already been disposed, this call has no effect.
    /// 
    pub fn dispatch (self, event: Event) {
        let r = unsafe {FREDispatchStatusEventAsync(self.0.as_ptr(), event.code.as_ptr(), event.level.as_ptr())};
        debug_assert!(r.is_ok());
    }

    /// Sends a debug message as a event through the associated `ExtensionContext`.
    /// 
    /// If the underlying context has already been disposed, this call has no effect.
    /// 
    pub fn debug (self, message: impl AsRef<str>) {
        let s = message.as_ref();
        let code = match UCStr::try_from(s) {
            Ok(s) => s,
            Err(e) => format!(
                "String contains an unexpected NUL byte at index {} (length: {})\n{}",
                e.nul_position(),
                s.len(),
                std::backtrace::Backtrace::capture(),
            ).try_into().unwrap(),
        };
        let evt = Event { level: EventLevel::Debug, code };
        self.dispatch(evt);
    }
}
unsafe impl Send for EventDispatcher {}


/// `flash.events.StatusEvent`
/// 
#[derive(Debug, Clone)]
pub struct Event {
    pub code: UCStr,
    pub level: EventLevel,
}
impl Event {
    pub fn new (code: UCStr, level: Option<EventLevel>) -> Self {
        let level = level.unwrap_or_default();
        Self { code, level }
    }
}


#[derive(Debug, Clone)]
pub enum EventLevel {
    Status,
    Warning,
    Error,
    Debug,
    Custom(UCStr)
}
impl EventLevel {
    pub fn as_ptr (&self) -> FREStr {
        const STATUS: FREStr = c"status".as_ptr() as FREStr;
        const WARNING: FREStr = c"warning".as_ptr() as FREStr;
        const ERROR: FREStr = c"error".as_ptr() as FREStr;
        const DEBUG: FREStr = c"debug".as_ptr() as FREStr;
        match self {
            Self::Status => STATUS,
            Self::Warning => WARNING,
            Self::Error => ERROR,
            Self::Debug => DEBUG,
            Self::Custom(s) => s.as_ptr(),
        }
    }
}
impl Default for EventLevel {
    fn default() -> Self {Self::Status}
}
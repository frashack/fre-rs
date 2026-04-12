//! 
//! Internal implementation details of the crate. Not intended for public use.
//!  



thread_local! {
    /// Thread-local storage for caching panic information,
    /// which will be propagated to the Flash Runtime.
    pub static LAST_PANIC_INFO: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

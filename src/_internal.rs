


thread_local! {
    pub static LAST_PANIC_INFO: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

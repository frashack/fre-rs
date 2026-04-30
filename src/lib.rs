//!
//! Safe, ergonomic Rust abstraction over the AIR Native Extension (ANE) C API ([`fre-sys`](https://crates.io/crates/fre-sys)) for native-side development.
//!
//! ## Getting Started
//!
//! The primary entry points of this crate are the macros [`extension!`] and [`function!`].
//! Refer to their documentation for details and examples.
//!
//! # Flash Runtime Extension Lifecycle
//!
//! ```text
//!                                   Flash-runtime ━━━━┓
//!                                                     ┃
//!          ExtensionContext.loadExtension ━━━━━━━━━━━━┫
//!                ↓                                    ┃
//!    ┏━━━━ Extension-Load ━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
//!    ┃           ↓                                    ┃
//!    ┃     Initializer → Extension-Data               ┃
//!    ┃           ↓                                    ┃
//!    ┃  ┏━━ ExtensionContext.createExtensionContext ━━┫
//!    ┃  ┃        ↓                                    ┃
//!    ┃  ┃    Context-Initializer → Context-Data       ┃
//!    ┃  ┃        ↓            ↓    ↓         ↓        ┃
//!    ┃  ┃    Function-Data → Function  Extension-Data ┃
//!    ┃  ┃                     ↑                       ┃
//!    ┃  ┣━━ ExtensionContext.call ━━━━━━━━━━━━━━━━━━━━┫
//!    ┃  ┃                                             ┃
//!    ┃  ┃                        Extension-Data       ┃
//!    ┃  ┃                              ↑              ┃
//!    ┃  ┃    Context-Data ≈ `ContextRegistry`         ┃
//!    ┃  ┃        ↓                     ↑              ┃
//!    ┃  ┃    Context-Finalizer    Function-Data       ┃
//!    ┃  ┃        ↑                                    ┃
//!    ┃  ┗━━ ExtensionContext.dispose ━━━━━━━━━━━━━━━━━┫
//!    ┃                                                ┃
//!    ┃            Extension-Data → Finalizer          ┃
//!    ┃                                 ↑              ┃
//!    ┗━━━━━━━━━━━━━━━━━━━━━━━━━━ Extension-Unload ━━━━┛
//! ```
//!



/// Namespace for ActionScript 3 classes and objects.
/// 
pub mod as3 {
    use super::*;
    pub use crate::types::{
        display::*,
        misc::*,
        object::{Object},
        primitive::*,
    };

    /// Although `'static`, using [`null`] outside the Flash runtime main thread,
    /// or within certain restricted closure call stacks (when an object is acquired
    /// and the runtime is constrained) is unsupported. Related APIs may return errors in such cases.
    /// 
    #[allow(non_upper_case_globals)]
    // 
    // ALL APIS THAT MAY BE USED IN UNSUPPORTED CASES MUST HANDLE ERRORS CORRECTLY AND MUST NOT PANIC. (`AsObject`, `Object`)
    // 
    pub const null: Object<'static> = unsafe {transmute(std::ptr::null_mut::<FREObject>())};
}

/// [`fre-sys`](https://crates.io/crates/fre-sys)
/// 
pub mod c {pub use fre_sys::*;}
pub mod prelude {
    pub use crate::{
        as3,
        types::{Type, object::{Object, NonNullObject, AsObject, AsNonNullObject, TryAs}},
        context::{Context, CurrentContext},
        data::Data,
        event::*,
        function::{FunctionSet, trace},
        validated::*,
    };
    pub use std::any::Any;
}
pub mod context;
pub mod data;
pub mod error;
pub mod event;
pub mod function;
mod macros;
pub mod misc;
pub mod types;
pub mod validated;
pub mod utils;


use {
    prelude::*,
    c::*,
    data::ExtensionData,
    error::*,
    function::*,
    misc::*,
    utils::*,
};
use std::{
    borrow::Cow,
    cell::{RefCell},
    collections::HashMap,
    ffi::{CStr, CString, NulError, c_void, c_char},
    fmt::{self, Debug, Display},
    marker::PhantomData,
    mem::{MaybeUninit, transmute},
    ptr::{NonNull},
    rc::Rc,
    str::Utf8Error,
    sync::{Arc, Mutex},
};


/// Internal implementation details of the crate. Not intended for public use.
/// 
#[doc(hidden)]
pub mod __private {
    pub unsafe trait Sealed {}
    pub(crate) const SEALED: () = ();
    pub mod context {
        pub use crate::context::stack::{with, with_initializer, with_method};
    }
}


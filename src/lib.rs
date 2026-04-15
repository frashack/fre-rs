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
//!                                   Flash Runtime ━━━━┓
//!                                                     ┃
//!          ExtensionContext.loadExtension ━━━━━━━━━━━━┫
//!                ↓                                    ┃
//!    ┏━━━━ Extension Load ━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
//!    ┃           ↓                                    ┃
//!    ┃     Initializer → Extension Data               ┃
//!    ┃           ↓                                    ┃
//!    ┃  ┏━━ ExtensionContext.createExtensionContext ━━┫
//!    ┃  ┃        ↓                                    ┃
//!    ┃  ┃    Context Initializer → Context Data       ┃
//!    ┃  ┃        ↓            ↓    ↓         ↓        ┃
//!    ┃  ┃    Function Data → Function  Extension Data ┃
//!    ┃  ┃                     ↑                       ┃
//!    ┃  ┣━━ ExtensionContext.call ━━━━━━━━━━━━━━━━━━━━┫
//!    ┃  ┃                                             ┃
//!    ┃  ┃                        Extension Data       ┃
//!    ┃  ┃                              ↑              ┃
//!    ┃  ┃    Context Data ≈ `ContextRegistry`         ┃
//!    ┃  ┃        ↓                     ↑              ┃
//!    ┃  ┃    Context Finalizer    Function Data       ┃
//!    ┃  ┃        ↑                                    ┃
//!    ┃  ┗━━ ExtensionContext.dispose ━━━━━━━━━━━━━━━━━┫
//!    ┃                                                ┃
//!    ┃            Extension Data → Finalizer          ┃
//!    ┃                                 ↑              ┃
//!    ┗━━━━━━━━━━━━━━━━━━━━━━━━━━ Extension Unload ━━━━┛
//! ```
//!



pub mod as3 {
    use super::*;
    pub use crate::types::{
        classes::{Array, Vector, ByteArray, BitmapData, Context3D, ErrorObject as Error},
        object::{Object},
        primitive::{int, uint, Number, Boolean, StringObject as String}
    };
    
    /// Although `'static`, it must not be used outside the Flash runtime main thread,
    /// or related APIs may return errors or panic due to failed assertions.
    #[allow(non_upper_case_globals)]
    pub const null: Object = unsafe {transmute(std::ptr::null_mut::<FREObject>())};
}
/// [`fre-sys`](https://crates.io/crates/fre-sys)
pub mod c {pub use fre_sys::*;}
pub mod prelude {
    pub use crate::{
        as3,
        types::{Type, object::{AsObject, TryAs}},
        context::{Context, CurrentContext},
        data::Data,
        event::*,
        function::FunctionSet,
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
pub mod _internal;

use {
    prelude::*,
    c::prelude::*,
    data::ExtensionData,
    error::*,
    function::*,
    utils::*,
    _internal::Sealed,
};
use std::{
    cell::{RefCell},
    collections::HashMap,
    error::Error,
    ffi::{CStr, CString, NulError, c_void, c_char},
    fmt::{self, Debug, Display},
    marker::PhantomData,
    mem::{transmute, size_of, size_of_val},
    ptr::{NonNull},
    str::Utf8Error,
    sync::{Arc, Mutex},
};
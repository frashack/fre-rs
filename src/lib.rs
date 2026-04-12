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
//!    ┃  ┃    Context Data = `ContextRegistry`         ┃
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


pub mod c {
    pub use fre_sys::*;
}
pub mod prelude {
    pub use crate::{
        context::*,
        data::Data,
        event::*,
        function::FunctionSet,
        types::*,
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
    error::*,
    function::*,
    utils::*,
};
use std::{
    cell::Cell,
    collections::HashMap,
    error::Error,
    ffi::{CStr, CString, NulError, c_void, c_char},
    fmt::{self, Debug, Display},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
    str::Utf8Error,
    sync::Arc,
};
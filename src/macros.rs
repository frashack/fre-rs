#[allow(unused_imports)]
use super::*;


/// 
/// Generates and links the required Flash Runtime Extension entry points and lifecycle hooks,
/// bridging the C ABI with safe Rust abstractions.
/// 
/// This macro accepts up to four optional function arguments:
/// [`Initializer`], [`Finalizer`], [`ContextInitializer`], and [`ContextFinalizer`],
/// along with two external symbols for `extern "C"` functions.
/// 
/// ## Full Example
/// ```
/// mod lib {
///     use fre_rs::prelude::*;
///     fre_rs::extension! {
///         extern symbol_initializer, symbol_finalizer;
///         move initializer, final finalizer;
///         gen context_initializer, final context_finalizer;
///     }
///     struct ExtensionData (i32);
///     impl Data for ExtensionData {}
///     fn initializer() -> Option<Box<dyn Any>> {Some(ExtensionData(0).into_boxed())}
///     fn finalizer(ext_data: Option<Box<dyn Any>>) {ExtensionData::from_boxed(ext_data.unwrap()).unwrap().0 = -1;}
///     fn context_initializer(frt: &FlashRuntime) -> FunctionSet {
///         let mut funcs = FunctionSet::new();
///         let ctx_type = unsafe {frt.current_context().with(|ctx_reg| {
///             ctx_reg.context_type()
///         })}.unwrap();
///         if ctx_type.is_some() {
///             funcs.add(None, None::<()>, method_name);
///         } else {
///             funcs.add(None, None::<()>, method_name2);
///         }
///         return funcs;
///     }
///     fn context_finalizer(frt: &FlashRuntime) {frt.current_context().set_actionscript_data(null).unwrap()}
///     fn method_implementation <'a> (frt: &FlashRuntime<'a>, data: Option<&mut dyn Any>, args: &[Object<'a>]) -> Object<'a> {null}
///     fre_rs::function! (method_name use method_implementation);
///     fre_rs::function! {
///         method_name2 (_, _, _) -> Option<Array> {None}
///     }
/// }
/// ```
/// ## Minimal Example
/// ```
/// mod lib {
///     use fre_rs::prelude::*;
///     fre_rs::extension! {
///         extern symbol_initializer;
///         gen context_initializer, final;
///     }
///     fn context_initializer(frt: &FlashRuntime) -> FunctionSet {
///         let mut funcs = FunctionSet::new();
///         funcs.add(None, None::<()>, method_name);
///         funcs
///     }
///     fre_rs::function! {
///         method_name (frt, _, args) -> StringObject {
///             frt.trace(args);
///             StringObject::new(frt, "Hello! Flash Runtime")
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! extension {
    {// #0
        extern $symbol_initializer:ident $(, $symbol_finalizer:ident;
        move $initializer:path, final $($finalizer:path)?)?;
        gen $context_initializer:path, final $($context_finalizer:path)?;
    } => {
        const _: () = {
        mod _flash_runtime_extension {
            use super::*;
            $crate::extension! {@Extern [$symbol_initializer $(, $symbol_finalizer, $initializer $(, $finalizer)?)?]}
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe extern "C" fn ctx_initializer (
                ext_data: $crate::c::markers::FREData,
                ctx_type: $crate::c::markers::FREStr,
                ctx: $crate::c::markers::FREContext,
                num_funcs_to_set: *mut u32,
                funcs_to_set: *mut *const $crate::c::ffi::FRENamedFunction,
            ) {
                let context_initializer: $crate::function::ContextInitializer = $context_initializer;
                $crate::context::FlashRuntime::with_context_initializer(ext_data, ctx_type, &ctx, num_funcs_to_set, funcs_to_set, $context_initializer);
            }
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe extern "C" fn ctx_finalizer (ctx: $crate::c::markers::FREContext) {
                $crate::context::FlashRuntime::with(&ctx, |frt| {
                    $(
                        let context_finalizer: $crate::function::ContextFinalizer = $context_finalizer;
                        context_finalizer(frt);
                    )?
                    let raw = $crate::validated::NonNullFREData::new(frt.current_context().get_native_data().expect("Failed to retrieve native data from FFI.")).expect("`ContextRegistry` is expected in native data but is missing.");
                    assert!(<$crate::context::ContextRegistry as $crate::data::Data>::ref_from(raw).is_ok());
                    $crate::data::drop_from(raw);
                });

            }
        }
        };
    };
    {// #1``
        @Extern [$symbol_initializer:ident, $symbol_finalizer:ident, $initializer:path $(, $finalizer:path)?]
    } => {
        #[allow(unsafe_op_in_unsafe_fn, non_snake_case)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $symbol_initializer (
            ext_data_to_set: *mut $crate::c::markers::FREData,
            ctx_initializer_to_set: *mut $crate::c::ffi::FREContextInitializer,
            ctx_finalizer_to_set: *mut $crate::c::ffi::FREContextFinalizer,
        ) {
            assert!(!ext_data_to_set.is_null());
            assert!(!ctx_initializer_to_set.is_null());
            assert!(!ctx_finalizer_to_set.is_null());
            let initializer: $crate::function::Initializer = $initializer;
            if let Some(ext_data) = initializer() {
                *ext_data_to_set = $crate::data::into_raw(ext_data).as_ptr();
            }
            *ctx_initializer_to_set = ctx_initializer;
            *ctx_finalizer_to_set = Some(ctx_finalizer);
            $crate::extension! (@Hook);
        }
        #[allow(unsafe_op_in_unsafe_fn, non_snake_case)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $symbol_finalizer (ext_data: $crate::c::markers::FREData) {
            let ext_data = $crate::validated::NonNullFREData::new(ext_data)
                .map(|raw| $crate::data::from_raw(raw));
            $(
                let finalizer: $crate::function::Finalizer = $finalizer;
                finalizer(ext_data);
            )?
        }
    };
    {// #2
        @Extern [$symbol_initializer:ident]
    } => {
        #[allow(unsafe_op_in_unsafe_fn, non_snake_case)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $symbol_initializer (
            ext_data_to_set: *mut $crate::c::markers::FREData,
            ctx_initializer_to_set: *mut $crate::c::ffi::FREContextInitializer,
            ctx_finalizer_to_set: *mut $crate::c::ffi::FREContextFinalizer,
        ) {
            assert!(!ext_data_to_set.is_null());
            assert!(!ctx_initializer_to_set.is_null());
            assert!(!ctx_finalizer_to_set.is_null());
            *ctx_initializer_to_set = ctx_initializer;
            *ctx_finalizer_to_set = Some(ctx_finalizer);
            $crate::extension! (@Hook);
        }
    };
    (// #3
        @Hook
    ) => {
        static INIT_HOOK: ::std::sync::Once = ::std::sync::Once::new();
        INIT_HOOK.call_once(|| {
            let default_hook = ::std::panic::take_hook();
            ::std::panic::set_hook(Box::new(move |info| {
                let payload = info.payload_as_str().unwrap_or_default();
                let location = info.location()
                    .map(|l| format!("at {}:{}:{}", l.file(), l.line(), l.column()))
                    .unwrap_or_default();
                let backtrace = ::std::backtrace::Backtrace::force_capture();
                let s = format!("{payload}\n{location}\n{backtrace}");
                $crate::_internal::LAST_PANIC_INFO.with(|i| {*i.borrow_mut() = Some(s);});
                default_hook(info);
            }));
        });
    }
}


/// 
/// Defines a function intended for context registration by generating its
/// ABI-compatible wrapper and binding it to a Rust implementation.
///
/// Expands to a `&'static` constant of type [`FunctionDefinition`],
/// intended to be added to a [`FunctionSet`].
/// 
/// # Panic Handling
/// Any [`panic`] occurring within the function body is intercepted via
/// [`std::panic::catch_unwind`]. Instead of unwinding across the FFI boundary,
/// an [`ErrorObject`] containing the captured panic details is constructed and
/// returned to the Flash Runtime.
///
/// This fallback return value is **NOT** constrained by the return type
/// declared in the macro invocation. On the ActionScript side, the result may
/// either be expected and handled as an `Error`, or not. In the latter case,
/// if an [`ErrorObject`] is returned, casting it to a non-error type yields
/// `null` and may lead to runtime exceptions.
///
/// When the [`ErrorObject`] is properly handled, the Flash Runtime remains
/// stable. However, care must be taken to avoid leaving the native extension
/// in an inconsistent state; resources should be managed reliably even in the
/// presence of panics.
/// 
/// ## Full Example
/// ```
/// mod lib {
///     use fre_rs::prelude::*;
///     fre_rs::function! {
///         method_name (frt, data, args) -> Option<Object> {
///             return frt.current_context().get_actionscript_data().ok();
///         }
///     }
///     fre_rs::function! (method_name2 use method_implementation);
///     fn method_implementation <'a> (frt: &FlashRuntime<'a>, data: Option<&mut dyn Any>, args: &[Object<'a>]) -> Object<'a> {null}
/// }
/// ```
/// ## Minimal Example
/// ```
/// mod lib {
///     fre_rs::function! {
///         method_name (_, _, _) {}
///     }
/// }
/// ```
#[macro_export]
macro_rules! function {
    {// #0
        $name:ident ($($arguments:tt)+) $(-> $return_type:ty)? $body:block
    } => {
        #[allow(non_upper_case_globals)]
        pub const $name: &'static $crate::function::FunctionDefinition = & $crate::function::FunctionDefinition::new(
            $crate::function!(@Name $name), {
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe extern "C" fn abi(
                ctx: $crate::c::markers::FREContext,
                func_data: $crate::c::markers::FREData,
                argc: u32,
                argv: *const $crate::c::markers::FREObject,
            ) -> $crate::c::markers::FREObject {
                fn func <'a> (
                    frt: &$crate::context::FlashRuntime<'a>,
                    func_data: Option<&mut dyn ::std::any::Any>,
                    args: &[$crate::types::Object<'a>],
                ) -> $crate::types::Object<'a> {
                    $crate::function! {@Parameters [frt, func_data, args] $($arguments)+}
                    let r = ::std::panic::catch_unwind(|| -> $crate::function!(@Return $($return_type)?) {
                        $body
                    });
                    $crate::function! (@Unwind [frt, r])
                }
                $crate::context::FlashRuntime::with_method(&ctx, func_data, argc, argv, func)
            }
            abi},
        );
    };
    (// #1
        $name:ident use $func:path
    ) => {
        #[allow(non_upper_case_globals)]
        pub const $name: &'static $crate::function::FunctionDefinition = & $crate::function::FunctionDefinition::new(
            $crate::function!(@Name $name), {
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe extern "C" fn abi(
                ctx: $crate::c::markers::FREContext,
                func_data: $crate::c::markers::FREData,
                argc: u32,
                argv: *const $crate::c::markers::FREObject,
            ) -> $crate::c::markers::FREObject {
                let func: $crate::function::Function = $func;
                let r = ::std::panic::catch_unwind(|| -> $crate::c::markers::FREObject {
                    $crate::context::FlashRuntime::with_method(&ctx, func_data, argc, argv, func)
                });
                let frt: $crate::context::FlashRuntime = ::std::mem::transmute(ctx);
                $crate::function! (@Unwind [&frt, r])
            }
            abi},
        );
    };
    (// #2
        @Name $name:ident
    ) => {
        unsafe {
            let s: &'static str = concat!(stringify!($name), "\0");
            let s: &'static ::std::ffi::CStr = ::std::ffi::CStr::from_bytes_with_nul_unchecked(s.as_bytes());
            let s = $crate::validated::UCStr::from_literal_unchecked(s);
            s
        }
    };
    (// #3
        @Return $return_type:ty
    ) => ($return_type); 
    (// #4
        @Return
    ) => (());
    {// #5
        @Parameters [$f:ident, $d:ident, $a:ident $(,)?]
        $frt:ident, $data:ident, $args:ident $(,)?
    } => {
        let $frt: &$crate::context::FlashRuntime<'a> = $f;
        let $data: Option<&mut dyn ::std::any::Any> = $d;
        let $args: &[$crate::types::Object<'a>] = $a;
    };
    {// #6
        @Parameters [$f:ident, $d:ident, $a:ident $(,)?]
        $frt:ident, $data:ident, _ $(,)?
    } => {
        $crate::function! {@Parameters [$f, $d, $a]
            $frt, $data, _args
        }
    };
    {// #7
        @Parameters [$f:ident, $d:ident, $a:ident $(,)?]
        $frt:ident, _, $args:ident $(,)?
    } => {
        $crate::function! {@Parameters [$f, $d, $a]
            $frt, _data, $args
        }
    };
    {// #8
        @Parameters [$f:ident, $d:ident, $a:ident $(,)?]
        _, $data:ident, $args:ident $(,)?
    } => {
        $crate::function! {@Parameters [$f, $d, $a]
            _frt, $data, $args
        }
    };
    {// #9
        @Parameters [$f:ident, $d:ident, $a:ident $(,)?]
        _, _, $args:ident $(,)?
    } => {
        $crate::function! {@Parameters [$f, $d, $a]
            _frt, _data, $args
        }
    };
    {// #10
        @Parameters [$f:ident, $d:ident, $a:ident $(,)?]
        _, $data:ident, _ $(,)?
    } => {
        $crate::function! {@Parameters [$f, $d, $a]
            _frt, $data, _args
        }
    };
    {// #11
        @Parameters [$f:ident, $d:ident, $a:ident $(,)?]
        $frt:ident, _, _ $(,)?
    } => {
        $crate::function! {@Parameters [$f, $d, $a]
            $frt, _data, _args
        }
    };
    {// #12
        @Parameters [$f:ident, $d:ident, $a:ident $(,)?]
        _, _, _ $(,)?
    } => {
        $crate::function! {@Parameters [$f, $d, $a]
            _frt, _data, _args
        }
    };
    (// #13
        @Unwind [$frt:expr_2021, $catched:expr_2021]
    ) => {
        match $catched {
            Ok(r) => r.into(),
            Err(_) => {
                let info = $crate::_internal::LAST_PANIC_INFO.with(|i| {i.borrow_mut().take()});
                let msg = info.as_ref()
                    .map(|s|s.as_str())
                    .unwrap_or("Panic occurred but no details were captured.");
                let err = $crate::types::ErrorObject::new($frt, Some(msg), i32::MIN);
                err.set_name(Some($crate::types::StringObject::new($frt, "Native Extension Panic Error")));
                err.into()
            },
        }
    }
}


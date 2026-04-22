#[allow(unused_imports)]
use super::*;


/// Generates and links the required Flash Runtime Extension entry points and lifecycle hooks,
/// bridging the C ABI with safe Rust abstractions.
/// 
/// This macro accepts two external symbols as arguments for `extern "C"` functions,
/// and four functions: [`Initializer`], [`Finalizer`], [`ContextInitializer`], [`ContextFinalizer`].
/// Some of these arguments are optional.
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
///     struct NativeData (i32);
///     impl Data for NativeData {}
///     fn initializer() -> Option<Box<dyn Any>> {
///         let extension_data = NativeData(-3).into_boxed();
///         return Some(extension_data);
///     }
///     fn finalizer(ext_data: Option<Box<dyn Any>>) {
///         assert_eq!(NativeData::from_boxed(ext_data.unwrap()).unwrap().0, -3);
///     }
///     fn context_initializer(ctx: &CurrentContext) -> (Option<Box<dyn Any>>, FunctionSet) {
///         let context_data = NativeData(-2).into_boxed();
///         let mut funcs = FunctionSet::with_capacity(1);
///         let function_data = NativeData(-1).into_boxed();
///         if ctx.ty().is_some() {
///             funcs.add(None, Some(function_data), method_name);
///         } else {
///             funcs.add(None, Some(function_data), method_name2);
///         }
///         return (Some(context_data), funcs);
///     }
///     fn context_finalizer(ctx: &CurrentContext) {
///         let context_data: &NativeData = ctx.data().unwrap().downcast_ref().unwrap();
///         assert_eq!(context_data.0, -2);
///         ctx.set_actionscript_data(as3::null)
///     }
///     fn method_implementation <'a> (ctx: &CurrentContext<'a>, data: Option<&mut dyn Any>, args: &[as3::Object<'a>]) -> as3::Object<'a> {as3::null}
///     fre_rs::function! (method_name use method_implementation);
///     fre_rs::function! {
///         method_name2 (ctx, data, args) -> Option<as3::Array> {
///             let function_data: &mut NativeData = data.unwrap().downcast_mut().unwrap();
///             assert_eq!(function_data.0, -1);
///             return None;
///         }
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
///     fn context_initializer(_: &CurrentContext) -> (Option<Box<dyn Any>>, FunctionSet) {
///         let mut funcs = FunctionSet::new();
///         funcs.add(None, None, method_name);
///         (None, funcs)
///     }
///     fre_rs::function! {
///         method_name (ctx, _, args) -> as3::String {
///             ctx.trace(args);
///             as3::String::new(ctx, "Hello! Flash Runtime.")
///         }
///     }
/// }
/// ```
/// 
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
                $crate::context::CurrentContext::with_context_initializer(ext_data, ctx_type, &ctx, num_funcs_to_set, funcs_to_set, $context_initializer);
            }
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe extern "C" fn ctx_finalizer (ctx: $crate::c::markers::FREContext) {
                $crate::context::CurrentContext::with(&ctx, |ctx| {
                    $(
                        let context_finalizer: $crate::function::ContextFinalizer = $context_finalizer;
                        context_finalizer(ctx);
                    )?
                    let ctx = *(ctx as *const $crate::context::CurrentContext as *const $crate::context::ForeignContext);
                    let raw = ctx
                        .get_native_data()
                        .expect("Failed to retrieve native data from FFI.")
                        .expect("`ContextRegistry` is expected in native data but is missing.");
                    assert!(<::std::cell::RefCell<$crate::context::ContextRegistry> as $crate::data::Data>::ref_from(raw).is_ok());
                    $crate::data::drop_from(raw);
                });

            }
        }
        };
    };
    {// #1
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
                let ext_data = ::std::sync::Arc::new(::std::sync::Mutex::new(ext_data));
                *ext_data_to_set = <::std::sync::Arc<::std::sync::Mutex<Box<dyn ::std::any::Any>>> as $crate::data::Data>::into_raw(ext_data).as_ptr();
            }
            *ctx_initializer_to_set = ctx_initializer;
            *ctx_finalizer_to_set = Some(ctx_finalizer);
        }
        #[allow(unsafe_op_in_unsafe_fn, non_snake_case)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $symbol_finalizer (ext_data: $crate::c::markers::FREData) {
            let ext_data = $crate::validated::NonNullFREData::new(ext_data)
                .map(|raw| {
                    let arc_mutex_boxed = <::std::sync::Arc<::std::sync::Mutex<Box<dyn ::std::any::Any>>> as $crate::data::Data>::from_raw(raw);
                    let mutex = ::std::sync::Arc::try_unwrap(arc_mutex_boxed).expect("INVARIANT: No context exists.");
                    let boxed = mutex.into_inner().expect("The mutex is poisoned.");
                    boxed
                });
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
        }
    };
}


/// Defines a function intended for context registration by generating its
/// ABI-compatible wrapper and binding it to a Rust implementation.
///
/// Expands to a `&'static` constant of type [`FunctionImplementation`],
/// intended to be added to a [`FunctionSet`].
/// 
/// This macro supports two forms:
/// one that defines a function inline, and another that binds to an existing
/// [`Function`] using the `use` keyword.
///
/// For larger or more complex implementations, the latter can provide better
/// IDE support. However, it introduces two distinct items with the same
/// semantics, so naming should be chosen carefully to avoid confusion.
/// 
/// # Panics and Debugging
/// 
/// If a panic occurs on the call stack, the process will abort.
/// The runtime may print a backtrace to `stderr`, but it may lack sufficient
/// symbol information (such as function names and source locations)
/// for meaningful debugging.
///
/// For example, when using the MSVC toolchain on Windows, including
/// the generated `.pdb` file alongside the `.dll` (in the `.ane` package)
/// allows debuggers and crash reporting tools to resolve symbols
/// and produce human-readable stack traces.
/// 
/// ## Full Example
/// ```
/// mod lib {
///     use fre_rs::prelude::*;
///     fre_rs::function! {
///         method_name (ctx, data, args) -> as3::Object {
///             return ctx.get_actionscript_data();
///         }
///     }
///     fre_rs::function! (method_name2 use method_implementation);
///     fn method_implementation <'a> (ctx: &CurrentContext<'a>, data: Option<&mut dyn Any>, args: &[as3::Object<'a>]) -> as3::Object<'a> {as3::null}
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
/// 
#[macro_export]
macro_rules! function {
    {// #0
        $name:ident ($($arguments:tt)+) $(-> $return_type:ty)? $body:block
    } => {
        #[allow(non_upper_case_globals)]
        pub const $name: &'static $crate::function::FunctionImplementation = & $crate::function::FunctionImplementation::new(
            $crate::function!(@Name $name), {
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe extern "C" fn abi(
                ctx: $crate::c::markers::FREContext,
                func_data: $crate::c::markers::FREData,
                argc: u32,
                argv: *const $crate::c::markers::FREObject,
            ) -> $crate::c::markers::FREObject {
                fn func <'a> (
                    ctx: &$crate::context::CurrentContext<'a>,
                    func_data: Option<&mut dyn ::std::any::Any>,
                    args: &[$crate::as3::Object<'a>],
                ) -> $crate::as3::Object<'a> {
                    $crate::function! {@Parameters [ctx, func_data, args] $($arguments)+}
                    (|| -> $crate::function!(@Return $($return_type)?) {
                        $body
                    })().into()
                }
                $crate::context::CurrentContext::with_method(&ctx, func_data, argc, argv, func)
            }
            abi},
        );
    };
    (// #1
        $name:ident use $func:path
    ) => {
        #[allow(non_upper_case_globals)]
        pub const $name: &'static $crate::function::FunctionImplementation = & $crate::function::FunctionImplementation::new(
            $crate::function!(@Name $name), {
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe extern "C" fn abi(
                ctx: $crate::c::markers::FREContext,
                func_data: $crate::c::markers::FREData,
                argc: u32,
                argv: *const $crate::c::markers::FREObject,
            ) -> $crate::c::markers::FREObject {
                $crate::context::CurrentContext::with_method(&ctx, func_data, argc, argv, $func)
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
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        $ctx:ident, $data:ident, $args:ident $(,)?
    } => {
        let $ctx: &$crate::context::CurrentContext<'a> = $c;
        let $data: Option<&mut dyn ::std::any::Any> = $d;
        let $args: &[$crate::as3::Object<'a>] = $a;
    };
    {// #6
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        $ctx:ident, $data:ident, _ $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            $ctx, $data, _args
        }
    };
    {// #7
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        $ctx:ident, _, $args:ident $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            $ctx, _data, $args
        }
    };
    {// #8
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        _, $data:ident, $args:ident $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            _ctx, $data, $args
        }
    };
    {// #9
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        _, _, $args:ident $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            _ctx, _data, $args
        }
    };
    {// #10
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        _, $data:ident, _ $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            _ctx, $data, _args
        }
    };
    {// #11
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        $ctx:ident, _, _ $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            $ctx, _data, _args
        }
    };
    {// #12
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        _, _, _ $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            _ctx, _data, _args
        }
    };
}


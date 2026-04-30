#[allow(unused_imports)]
use super::*;


/// Generates and links the required Flash Runtime Extension entry points and lifecycle hooks,
/// bridging the C ABI with safe Rust abstractions.
/// 
/// This macro accepts two external symbols to be exported as part of the public ABI,
/// and four functions: [`Initializer`], [`Finalizer`], [`ContextInitializer`], [`ContextFinalizer`].
/// Some of these arguments are optional.
/// 
/// # Full Examples
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
///     fn method_implementation <'a> (ctx: &CurrentContext<'a>, data: Option<&mut dyn Any>, args: &[Object<'a>]) -> Object<'a> {as3::null}
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
/// # Minimal Examples
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
///             trace(args);
///             as3::String::new(ctx, "Hello, Flash runtime!")
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
        mod __flash_runtime_extension {
            use super::*;
            $crate::extension! {@Extern [$symbol_initializer $(, $symbol_finalizer, $initializer $(, $finalizer)?)?]}
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe extern "C" fn ctx_initializer (
                ext_data: $crate::c::FREData,
                ctx_type: $crate::c::FREStr,
                ctx: $crate::c::FREContext,
                num_funcs_to_set: *mut u32,
                funcs_to_set: *mut *const $crate::c::FRENamedFunction,
            ) {
                let context_initializer: $crate::function::ContextInitializer = $context_initializer;
                $crate::__private::context::with_initializer(ext_data, ctx_type, &ctx, num_funcs_to_set, funcs_to_set, $context_initializer);
            }
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe extern "C" fn ctx_finalizer (ctx: $crate::c::FREContext) {
                $crate::__private::context::with(&ctx, |ctx| {
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
            ext_data_to_set: *mut $crate::c::FREData,
            ctx_initializer_to_set: *mut $crate::c::FREContextInitializer,
            ctx_finalizer_to_set: *mut $crate::c::FREContextFinalizer,
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
            *ctx_finalizer_to_set = ctx_finalizer;
        }
        #[allow(unsafe_op_in_unsafe_fn, non_snake_case)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $symbol_finalizer (ext_data: $crate::c::FREData) {
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
            ext_data_to_set: *mut $crate::c::FREData,
            ctx_initializer_to_set: *mut $crate::c::FREContextInitializer,
            ctx_finalizer_to_set: *mut $crate::c::FREContextFinalizer,
        ) {
            assert!(!ctx_initializer_to_set.is_null());
            assert!(!ctx_finalizer_to_set.is_null());
            *ctx_initializer_to_set = ctx_initializer;
            *ctx_finalizer_to_set = ctx_finalizer;
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
/// # Full Examples
/// ```
/// mod lib {
///     use fre_rs::prelude::*;
///     fre_rs::function! {
///         method_name (ctx, data, args) -> Object {
///             return ctx.get_actionscript_data();
///         }
///     }
///     fre_rs::function! (method_name2 use method_implementation);
///     fn method_implementation <'a> (ctx: &CurrentContext<'a>, data: Option<&mut dyn Any>, args: &[Object<'a>]) -> Object<'a> {as3::null}
/// }
/// ```
/// # Minimal Examples
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
            $crate::ucstringify! ($name), {
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe extern "C" fn abi(
                ctx: $crate::c::FREContext,
                func_data: $crate::c::FREData,
                argc: u32,
                argv: *const $crate::c::FREObject,
            ) -> $crate::c::FREObject {
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
                $crate::__private::context::with_method(&ctx, func_data, argc, argv, func)
            }
            abi},
        );
    };
    (// #1
        $name:ident use $func:path
    ) => {
        #[allow(non_upper_case_globals)]
        pub const $name: &'static $crate::function::FunctionImplementation = & $crate::function::FunctionImplementation::new(
            $crate::ucstringify! ($name), {
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe extern "C" fn abi(
                ctx: $crate::c::FREContext,
                func_data: $crate::c::FREData,
                argc: u32,
                argv: *const $crate::c::FREObject,
            ) -> $crate::c::FREObject {
                $crate::__private::context::with_method(&ctx, func_data, argc, argv, $func)
            }
            abi},
        );
    };
    (// #2
        @Return $return_type:ty
    ) => ($return_type); 
    (// #3
        @Return
    ) => (());
    {// #4
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        $ctx:ident, $data:ident, $args:ident $(,)?
    } => {
        let $ctx: &$crate::context::CurrentContext<'a> = $c;
        let $data: Option<&mut dyn ::std::any::Any> = $d;
        let $args: &[$crate::as3::Object<'a>] = $a;
    };
    {// #5
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        $ctx:ident, $data:ident, _ $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            $ctx, $data, _args
        }
    };
    {// #6
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        $ctx:ident, _, $args:ident $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            $ctx, _data, $args
        }
    };
    {// #7
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        _, $data:ident, $args:ident $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            _ctx, $data, $args
        }
    };
    {// #8
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        _, _, $args:ident $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            _ctx, _data, $args
        }
    };
    {// #9
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        _, $data:ident, _ $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            _ctx, $data, _args
        }
    };
    {// #10
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        $ctx:ident, _, _ $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            $ctx, _data, _args
        }
    };
    {// #11
        @Parameters [$c:ident, $d:ident, $a:ident $(,)?]
        _, _, _ $(,)?
    } => {
        $crate::function! {@Parameters [$c, $d, $a]
            _ctx, _data, _args
        }
    };
}


/// Defines a new AS3 class.
/// 
/// Accepts a unique class name as an argument.
/// 
/// By default, the generated class implements [`PartialEq<Self>`] and [`Eq`]
/// using pointer equality. This can be disabled by adding the `!PartialEq` modifier.
/// 
/// # Examples
/// 
/// ```
/// use fre_rs::prelude::*;
/// fre_rs::class! (EventDispatcher);
/// impl<'a> EventDispatcher<'a> {
///     pub fn new (ctx: &CurrentContext<'a>) -> Self {
///        unsafe {ctx.construct(fre_rs::ucstringify!(flash.events.EventDispatcher), None)
///             .expect("Object construction failed.")
///             .as_unchecked()}
///     }
/// }
/// fre_rs::class! (XML !PartialEq);
/// impl PartialEq for XML<'_> {
///     fn eq(&self, other: &Self) -> bool {todo!()}
/// }
/// impl Eq for XML<'_> {}
/// ```
/// 
#[macro_export]
macro_rules! class {

    // #0
    {
        $(#[$meta:meta])*
        $name:ident $($modifier:tt)*
    } => {
        $crate::class! {@Define
            $(#[$meta])*
            $name $($modifier)*
        }
        unsafe impl<'a> $crate::types::object::AsObject<'a> for $name<'a> {
            const TYPE: $crate::types::Type = $crate::types::Type::Named(stringify!($name));
        }
    };

    // #1
    {@Typeof
        $(#[$meta:meta])*
        $name:ident $($modifier:tt)*
    } => {                                                          const _: () = $crate::__private::SEALED;
        $crate::class! {@Define
            $(#[$meta])*
            $name $($modifier)*
        }
        unsafe impl<'a> $crate::types::object::AsObject<'a> for $name<'a> {
            const TYPE: $crate::types::Type = $crate::types::Type::$name;
        }
        impl<'a> TryFrom<$crate::as3::Object<'a>> for $name<'a> {
            type Error = $crate::types::Type;
            fn try_from (object: $crate::as3::Object<'a>) -> Result<Self, Self::Error> {
                let ty = <$crate::as3::Object as $crate::types::object::AsObject>::get_type(object);
                if ty == <Self as $crate::types::object::AsObject>::TYPE {
                    Ok(unsafe {<$crate::as3::Object as $crate::types::object::AsObject>::as_unchecked(object)})
                }else{Err(ty)}
            }
        }
        impl<'a> TryFrom<$crate::types::object::NonNullObject<'a>> for $name<'a> {
            type Error = $crate::types::Type;
            fn try_from (object: $crate::types::object::NonNullObject<'a>) -> Result<Self, Self::Error> {
                <$crate::types::object::NonNullObject as $crate::types::object::AsObject>::as_object(object).try_into()
            }
        }
    };

    // #2
    {@Define
        $(#[$meta:meta])*
        $name:ident
    } => {
        $(#[$meta])*
        #[derive(::std::fmt::Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct $name <'a> (::std::ptr::NonNull<::std::ffi::c_void>, ::std::marker::PhantomData<&'a ()>);
        $crate::class!(@Implement $name);
    };

    // #3
    {@Define
        $(#[$meta:meta])*
        $name:ident !PartialEq
    } => {
        $(#[$meta])*
        #[derive(::std::fmt::Debug, Clone, Copy)]
        #[repr(transparent)]
        pub struct $name <'a> (::std::ptr::NonNull<::std::ffi::c_void>, ::std::marker::PhantomData<&'a ()>);
        $crate::class!(@Implement $name);
    };

    // #4
    {@Implement
        $name:ident
    } => {
        #[cfg(debug_assertions)]
        const _: () = {
            #[used]
            #[unsafe(export_name = concat!("__class_", stringify!($name)))]
            pub static CLASS_NAME_MUST_UNIQUE: u8 = 0;
        };

        unsafe impl $crate::__private::Sealed for $name<'_> {}

        impl<'a> $crate::types::object::AsNonNullObject<'a> for $name<'a> {}
        impl<'a> From<$name<'a>> for $crate::types::object::NonNullObject<'a> {fn from(object: $name<'a>) -> Self {<$name as $crate::types::object::AsNonNullObject>::as_non_null_object(object)}}
        
        impl From<$name<'_>> for $crate::c::FREObject {fn from(object: $name) -> Self {<$name as $crate::types::object::AsObject>::as_ptr(object)}}
        
        impl ::std::fmt::Display for $name<'_> {fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {::std::fmt::Display::fmt(&(<Self as $crate::types::object::AsNonNullObject>::as_non_null_object(*self)), f)}}
        
        impl<'a> $crate::validated::ToUcstrLossy for & $name<'a> {fn to_ucstr_lossy(&self) -> $crate::validated::UCStr {<$name as $crate::validated::ToUcstrLossy>::to_ucstr_lossy(*self)}}
        impl<'a> $crate::validated::ToUcstrLossy for &mut $name<'a> {fn to_ucstr_lossy(&self) -> $crate::validated::UCStr {<$name as $crate::validated::ToUcstrLossy>::to_ucstr_lossy(*self)}}
    };
}


/// Interprets the input tokens as a string literal and yields a const [`UCStr`].
///
/// Whitespace between tokens is normalized in the same way as [`stringify!`].
/// 
/// Note that the expanded results of the input tokens may change in the future.
/// You should be careful if you rely on the output.
/// 
/// # Examples
/// 
/// ```
/// use fre_rs::prelude::*;
/// const METHOD_NAME: UCStr = fre_rs::ucstringify! (addChild);
/// const ERROR_MESSAGE: UCStr = fre_rs::ucstringify! {Invalid argument    type.    };
/// assert_eq! (METHOD_NAME.as_str(), "addChild");
/// assert_eq! (ERROR_MESSAGE.as_str(), "Invalid argument type.");
/// ```
/// 
#[macro_export]
macro_rules! ucstringify {

    // #0
    {$($tokens:tt)*} => {
        {
            const STR: &'static str = concat!(stringify!($($tokens)*), "\0");
            const CSTR: &'static ::std::ffi::CStr = unsafe {::std::ffi::CStr::from_bytes_with_nul_unchecked(STR.as_bytes())};
            const UCSTR: $crate::validated::UCStr = unsafe {$crate::validated::UCStr::from_literal_unchecked(CSTR)};
            UCSTR
        }
    };
}


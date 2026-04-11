use super::*;


/// The primary entry point of this crate, providing access to its public APIs.
#[derive(Debug)]
#[repr(transparent)]
pub struct FlashRuntime<'a> (FREContext, PhantomData<&'a()>);
impl<'a> FlashRuntime<'a> {
    pub fn current_context(&self) -> Context<'a> {Context(self.0, PhantomData)}
    pub fn event_dispatcher(&self) -> EventDispatcher {self.current_context().event_dispatcher()}

    /// A wrapper used by [`FREContextInitializer`], [`FREContextFinalizer`], and
    /// [`FREFunction`] that provides a safe stack-level execution environment.
    ///
    /// **In typical usage of this crate, this function should not be called directly.**
    /// 
    /// # Safety
    /// While all operations performed within this function are safe at the stack level,
    /// calling this function itself is unsafe and requires that all input arguments
    /// are valid. In particular, this function assumes it is invoked directly with
    /// arguments provided by the Flash runtime.
    ///
    /// Violating these assumptions may lead to undefined behavior.
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn with <F, R> (ctx: &'a FREContext, f: F) -> R
    where
        F: FnOnce (&FlashRuntime<'a>) -> R,
        R: 'a,
    {
        assert!(!ctx.is_null());
        assert!(!IS_FRT_BORROWED.get());
        IS_FRT_BORROWED.set(true);
        let frt: FlashRuntime<'a> = Self(*ctx, PhantomData);
        let r = f(&frt);
        IS_FRT_BORROWED.set(false);
        r
    }
    
    /// A wrapper around [`FREContextInitializer`] that provides a safe stack-level
    /// execution environment for context initialization.
    /// 
    /// **In typical usage of this crate, this function should not be called directly.**
    ///
    /// # Safety
    /// While all operations performed within this function are safe at the stack level,
    /// calling this function itself is unsafe and requires the following conditions:
    ///
    /// - The native data associated with [`Context`] must not be accessed or managed
    ///   by external code.
    /// - This function will construct a [`ContextRegistry`] and assign it as the native data.
    ///   The constructed [`ContextRegistry`] must be properly disposed in
    ///   [`FREContextFinalizer`] to ensure its lifecycle is correctly terminated.
    /// - This function assumes it is invoked directly with arguments provided by the
    ///   Flash runtime, meaning all arguments must be valid and consistent.
    ///
    /// Violating these assumptions may lead to undefined behavior.
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn with_context_initializer <F> (
        ext_data: FREData,// &'extension mut
        ctx_type: FREStr,// &'function
        ctx: &'a FREContext,// &'function
        num_funcs_to_set: *mut u32,// return
        funcs_to_set: *mut *const FRENamedFunction,// return &'context mut
        f: F
    )
    where F: FnOnce (&FlashRuntime<'a>) -> FunctionSet
    {
        assert!(!num_funcs_to_set.is_null());
        assert!(!funcs_to_set.is_null());
        Self::with(ctx, |frt|{
            let ctx_type = if ctx_type.is_null() {None} else {
                let ctx_type = CStr::from_ptr(ctx_type as *const c_char);
                let ctx_type = UCStr::try_from(ctx_type).expect("Input string is not valid UTF-8.");
                Some(ctx_type)
            };
            let ctx_data = ContextRegistry::new(ext_data, ctx_type).into_raw();
            let r= frt.current_context().set_native_data(ctx_data);// <'context> move
            assert!(r.is_ok());
            let r = f(frt);
            let methods = MethodSet::from(r);
            let r = methods.as_ref();
            assert!(r.len() <= u32::MAX as usize);
            *num_funcs_to_set = r.len() as u32;
            *funcs_to_set = r.as_ptr();
            let ctx_data = ContextRegistry::mut_from(ctx_data).unwrap();
            ctx_data.methods = Some(methods);
        })
    }

    /// A wrapper around [`FREFunction`] that provides a safe stack-level execution
    /// environment for the given closure.
    /// 
    /// **In typical usage of this crate, this function should not be called directly.**
    ///
    /// # Safety
    /// While all operations performed within this function are safe at the stack level,
    /// calling this function itself is unsafe and requires the following conditions:
    ///
    /// - `func_data` must either be constructed via [`Data::into_raw`] before
    ///   [`ContextInitializer`] returns, or be a null pointer.
    /// - This function assumes it is invoked directly with arguments provided by the
    ///   Flash runtime, meaning all arguments must be valid and consistent.
    ///
    /// Violating these assumptions may lead to undefined behavior.
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn with_method <F> (
        ctx: &'a FREContext,// &'function
        func_data: FREData,// &'function mut
        argc: u32,
        argv: *const FREObject,// &'function
        f: F
    ) -> FREObject
    where F: FnOnce (&FlashRuntime<'a>, Option<&mut dyn Any>, &[Object<'a>]) -> Object<'a>
    {
        assert!(!argv.is_null() || argc==0);
        Self::with(ctx, |frt| {
            let func_data = NonNullFREData::new(func_data)
                .map(|raw| crate::data::mut_from(raw));
            let args = std::slice::from_raw_parts(argv as *const Object, argc as usize);
            f(frt, func_data, args).as_ptr()
        })
    }
    
    pub fn trace (&self, msg: impl Into<UCStr>) {_ = self.current_context().trace(msg);}

}
impl Drop for FlashRuntime<'_> {
    fn drop(&mut self) {IS_FRT_BORROWED.set(false);}
}
thread_local! {static IS_FRT_BORROWED: Cell<bool> = Cell::new(false)}


/// A handle to a context that may become invalid under specific conditions.
///
/// Invalidity only occurs after the associated `ExtensionContext` AS3 object
/// has been disposed. Therefore, callers should be prepared for operations
/// on [`Context`] to fail at appropriate points.
///
/// This crate leverages [`FREGetFREContextFromExtensionContext`] to enable
/// more advanced use cases, but doing so also increases overall complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Context <'a> (FREContext, PhantomData<&'a()>);
impl<'a> Context<'a> {
    pub fn is_valid(self) -> bool {self.get_actionscript_data().is_ok()}
    pub fn event_dispatcher(self) -> EventDispatcher {EventDispatcher(self.0)}

    /// Provides exclusive access to the [`ContextRegistry`] in a constrained manner.
    /// 
    /// # Safety
    /// This method assumes that the native data associated with the [`Context`]
    /// was created and is exclusively managed by [`crate::extension!`] as a [`ContextRegistry`].
    ///
    /// To call this method safely, you must guarantee that the native data
    /// attached to the [`Context`] has NOT been manually managed, replaced,
    /// or altered by external code. Violating this assumption may lead to
    /// undefined behavior.
    ///
    /// The use of a closure and the [`Sync`] bound is intentional: it constrains
    /// the ordering of FFI interactions, while helping prevent data races,
    /// preserving memory safety, and avoiding uncontrolled complexity growth
    /// in cross-boundary usage.
    /// 
    /// [`Err`]=> [`ContextError::InvalidContext`], [`ContextError::NullData`], [`ContextError::UnexpectedData`], [`ContextError::FfiCallFailed`];
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn with <F, R> (self, f: F) -> Result<R, ContextError>
    where F: FnOnce (&mut ContextRegistry) -> R + Sync {
        if !self.is_valid() {return Err(ContextError::InvalidContext)}
        let raw = NonNullFREData::new(self.get_native_data()?)
            .ok_or(ContextError::NullData)?;
        let cr = ContextRegistry::mut_from(raw)
            .map_err(|_| ContextError::UnexpectedData)?;
        let r = f(cr);
        Ok(r)
    }
    
    /// **In typical usage of this crate, this function should not be called directly.**
    ///
    /// This is because the native data is reserved for [`ContextRegistry`] and is
    /// automatically managed by [`crate::extension!`].
    pub fn get_native_data (self) -> Result<FREData, FfiError> {
        let mut data = FREData::default();
        let r = unsafe {FREGetContextNativeData(self.0, &mut data)};
        if let Ok(e) = FfiError::try_from(r) {Err(e)} else {Ok(data)}
    }
    
    /// **In typical usage of this crate, this function should not be called directly.**
    ///
    /// This is because the native data is reserved for [`ContextRegistry`] and is
    /// automatically managed by [`crate::extension!`].
    /// 
    /// # Safety
    ///
    /// This function sets the native data pointer associated with the underlying [`FREContext`].
    /// Calling this function is unsafe because the caller must uphold the following invariants:
    ///
    /// 1. **The native data must not have been previously set**
    ///    - The context must currently be in an uninitialized state with respect to native data.
    ///    - In other words, [`Self::get_native_data`] must return null pointer.
    ///    - Violating this will overwrite an existing pointer, causing a memory leak or double-free.
    ///
    /// 2. **The ownership and lifetime of `data` must be correctly managed**
    ///    - If `data` represents a moved (owned) allocation, the caller is responsible for ensuring
    ///      that it is eventually freed.
    ///    - The memory must remain valid for the entire lifetime of the [`FREContext`].
    ///    - The allocation must be released no later than in the [`FREContextFinalizer`] callback,
    ///      where the native data is expected to be cleaned up manually.
    ///
    /// Failure to uphold these guarantees may result in undefined behavior, including memory leaks,
    /// use-after-free, or double-free errors.
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn set_native_data (self, data: NonNullFREData) -> Result<(), FfiError> {
        debug_assert!(self.get_native_data()?.is_null());
        let r = FRESetContextNativeData(self.0, data.as_ptr());
        if let Ok(e) = FfiError::try_from(r) {Err(e)} else {Ok(())}
    }
    /// `flash.external.ExtensionContext.actionScriptData`
    pub fn get_actionscript_data (self) -> Result<Object<'a>, FfiError> {
        let mut obj = FREObject::default();
        let r = unsafe {FREGetContextActionScriptData(self.0, &mut obj)};
        if let Ok(e) = FfiError::try_from(r) {Err(e)} else {Ok(unsafe {transmute(obj)})}
    }
    /// `flash.external.ExtensionContext.actionScriptData`
    pub fn set_actionscript_data (self, object: Object<'_>) -> Result<(), FfiError> {
        let r = unsafe {FRESetContextActionScriptData(self.0, object.as_ptr())};
        if let Ok(e) = FfiError::try_from(r) {Err(e)} else {Ok(())}
    }

    /// Calls a registered function by name through this crate's internal API.
    ///
    /// # Safety
    /// To call this method safely, all safety requirements of [`Self::with`]
    /// must be upheld. In particular, the native data associated with the
    /// [`Context`] must be a valid [`ContextRegistry`] created and exclusively
    /// managed by [`crate::extension!`], and must not be manually modified or
    /// replaced by external code.
    ///
    /// Violating these conditions may lead to undefined behavior.
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn call_method (self, name: &str, args: &[Object<'a>] ) -> Result<Object<'a>, ContextError> {
        self.with(|registry| {
            registry.methods.as_mut()
            .ok_or(ContextError::MethodsNotRegistered)
            .map(|ms| ms.get(name))
        })??
        .ok_or(ContextError::MethodNotFound)
        .map(|(func, data)| {
            let r = func(self.0, data, args.len() as u32, args.as_ptr() as *mut FREObject);
            transmute(r)
        })
    }
    
    pub fn trace (self, msg: impl Into<UCStr>) -> Result<(), FfiError> {
        let r = unsafe {FRETrace(self.0, msg.into().as_ptr())};
        if let Ok(e) = FfiError::try_from(r) {Err(e)} else {Ok(())}
    }

    /// Return [`Err`] if `stage` is non-null but not a `Stage` object
    /// 
    /// This is a minimal safety wrapper around the underlying FFI. Its current placement,
    /// shape, and usage are not ideal, and it is hoped that it can be refactored
    /// if a more flexible C API becomes available in the AIR SDK.
    pub fn get_render_mode (self, stage: Option<Object<'a>>) -> Result<crate::misc::RenderMode, FfiError> {
        let stage = stage.unwrap_or_default();
        let mut rm = u8::default();
        let r = unsafe {FREGetRenderMode(self.0, stage.as_ptr(), &mut rm)};
        if let Ok(e) = FfiError::try_from(r) {
            Err(e)
        } else {
            let rm: FRERenderMode = FRERenderMode(rm as i32);
            Ok(crate::misc::RenderMode::from(rm))
        }
    }
    
    /// `air.media.MediaBuffer` (AIR SDK 51)
    /// 
    /// `AIR-5963: Add ANE capabilities to render a Sprite using a MediaBuffer - initial support via BitmapData`
    /// 
    /// This is a minimal safety wrapper around the underlying FFI. Its current placement,
    /// shape, and usage are not ideal, and it is hoped that it can be refactored
    /// if a more flexible C API becomes available in the AIR SDK.
    pub fn set_render_source (self, media_buffer: Object<'a>, sprite: Object<'a>) -> Result<(), FfiError> {
        let r = unsafe {FRESetRenderSource(self.0, media_buffer.as_ptr(), sprite.as_ptr())};
        if let Ok(e) = FfiError::try_from(r) {Err(e)} else {Ok(())}
    }

    /// This is a minimal safety wrapper around the underlying FFI. Its current placement,
    /// shape, and usage are not ideal, and it is hoped that it can be refactored
    /// if a more flexible C API becomes available in the AIR SDK.
    pub fn with_media_buffer <F, R> (self, media_buffer: Object<'a>, f: F) -> Result<R, FfiError> 
    where F: FnOnce (MediaBufferDataAdapter) -> R {
        let mut bytes = FREBytes::default();
        let mut width = u32::default();
        let mut height = u32::default();
        let mut stride = u32::default();
        let mut format = u32::default();
        let result = unsafe {FREMediaBufferLock(self.0, media_buffer.as_ptr(), &mut bytes, &mut width, &mut height, &mut stride, &mut format)};
        if let Ok(e) = FfiError::try_from(result) {return Err(e);}
        let adapter = unsafe {MediaBufferDataAdapter::new(bytes, width, height, stride, format)};
        let r = f(adapter);
        let result = unsafe {FREMediaBufferUnlock(self.0, media_buffer.as_ptr(), u32::default())};
        debug_assert!(result.is_ok());
        Ok(r)
    }

}


/// The extension-side concrete representation of a [`Context`].
///
/// This can be considered the actual context instance, while [`Context`]
/// serves as an abstract handle or outer wrapper around it.
/// 
/// This struct is only constructed via [`FlashRuntime::with_context_initializer`].
#[derive(Debug)]
pub struct ContextRegistry {
    ctx_type: Option<UCStr>,
    ctx_data: Option<Box<dyn Any>>,
    ext_data: Option<NonNullFREData>,// &'extension mut
    methods: Option<MethodSet>,
}
impl ContextRegistry {
    pub fn context_type(&self) -> Option<UCStr> {(self.ctx_type).clone()}
    pub fn context_data(&self) -> Option<&dyn Any> {self.ctx_data.as_ref().map(|b| b.as_ref())}
    pub fn context_data_mut(&mut self) -> &mut Option<Box<dyn Any>> {&mut self.ctx_data}

    /// Returns a reference to the extension data.
    ///
    /// # Safety
    /// This method assumes that `ext_data` was either constructed via
    /// [`Data::into_raw`] or is a null pointer.
    ///
    /// Violating these assumptions may lead to undefined behavior.
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn extension_data(&self) -> Option<&dyn Any> {
        self.ext_data
            .as_ref()
            .map(|raw| crate::data::ref_from(*raw))
    }
    /// Returns a mutable reference to the extension data.
    ///
    /// # Safety
    /// This method assumes that `ext_data` was either constructed via
    /// [`Data::into_raw`] or is a null pointer.
    ///
    /// Violating these assumptions may lead to undefined behavior.
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn extension_data_mut(&mut self) -> Option<&mut dyn Any> {
        self.ext_data
            .as_ref()
            .map(|raw| crate::data::mut_from(*raw))
    }
    fn new (ext_data: FREData, ctx_type: Option<UCStr>) -> Self {
        let ext_data = NonNullFREData::new(ext_data);
        Self {
            ctx_type,
            ctx_data: None,
            ext_data,
            methods: None,
        }
    }
}
impl Data for ContextRegistry {}


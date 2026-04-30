//! 
//! Core implementation of the extension abstraction.
//! 


use super::*;


unsafe trait Sealed {}
pub(crate) type ContextHandle = NonNull<c_void>;


/// The current ANE context, on which most APIs in this crate depend.
///
/// The lifetime is strictly tied to the function call stack.
///
/// # Invariant
/// 
/// During any context-related call, the current context is guaranteed to be
/// valid, and its associated `ExtensionContext` object must not call `dispose`.
///
/// Violating this invariant may cause subsequent API calls to fail and lead to
/// rapidly increasing complexity in error handling. This crate treats such
/// situations as invalid state and does not attempt to recover from them.
/// 
#[derive(Debug)]
#[repr(transparent)]
pub struct CurrentContext<'a> (ContextHandle, PhantomData<&'a()>);
impl<'a> CurrentContext<'a> {

    /// Returns the context type associated with the current context.
    ///
    /// This corresponds to the `contextType` argument passed to
    /// `ExtensionContext.createExtensionContext`.
    ///
    /// Returns [`None`] if that argument was `null`.
    /// 
    pub fn ty(&self) -> Option<UCStr> {self.ctx_reg().context_type()}
    
    /// Returns an immutable reference to the Context Data associated with the current context.
    ///
    /// Context Data is user-defined data bound to the context, sharing the same
    /// lifetime as the context itself.
    ///
    /// It can only be set via the first return value of [`ContextInitializer`].
    /// 
    pub fn data(&self) -> Option<&dyn Any> {self.ctx_reg().context_data()}

    /// Returns a mutable reference to the Context Data associated with the current context.
    ///
    /// Context Data is user-defined data bound to the context, sharing the same
    /// lifetime as the context itself.
    ///
    /// It can only be set via the first return value of [`ContextInitializer`].
    /// 
    pub fn data_mut(&mut self) -> Option<&mut dyn Any> {self.ctx_reg_mut().context_data_mut()}

    /// Calls the constructor for the specified class.
    /// 
    pub fn construct (&self, class: UCStr, args: Option<&[Object]>) -> Result<NonNullObject<'a>, ExternalError<'a>> {
        let args = args.unwrap_or_default();
        debug_assert!(args.len() <= u32::MAX as usize);
        let mut object = MaybeUninit::<FREObject>::uninit();
        let mut thrown = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FRENewObject(class.as_ptr(), args.len() as u32, transmute(args.as_ptr()), object.as_mut_ptr(), thrown.as_mut_ptr())};
        if let Some(e) = ExternalError::try_from(r, Some(unsafe {transmute(thrown)})) {
            Err(e)
        }else{
            let object = unsafe {object.assume_init()};
            assert!(!object.is_null());
            Ok(unsafe {transmute(object)})
        }
    }

    /// Calls a method associated with the current context.
    ///
    /// Methods can only be set via the second return value of [`ContextInitializer`].
    /// 
    /// [`Err`]=> [`ContextError::MethodsNotRegistered`], [`ContextError::MethodNotFound`];
    /// 
    pub fn call_method (&mut self, name: &str, args: Option<&[Object]> ) -> Result<Object<'a>, ContextError> {
        self.ctx_reg_mut().methods.as_ref()
            .ok_or(ContextError::MethodsNotRegistered)
            .map(|ms| ms.get(name))?
            .ok_or(ContextError::MethodNotFound)
            .map(|(func, data)| {
                let args = args.unwrap_or_default();
                let argc = args.len() as u32;
                let argv = args.as_ptr() as *const FREObject;
                let r = unsafe {func(self.as_ptr(), data, argc, argv)};
                unsafe {transmute(r)}
            })
    }

    /// Returns the ActionScript-side Context Data associated with the current context.
    /// 
    /// `ExtensionContext.actionScriptData`
    /// 
    pub fn get_actionscript_data (&self) -> Object<'a>
    {self.as_cooperative_ctx().get_actionscript_data().expect("INVARIANT: `CurrentContext` is always valid.")}

    /// Sets the ActionScript-side Context Data associated with the current context.
    /// 
    /// `ExtensionContext.actionScriptData`
    /// 
    pub fn set_actionscript_data (&self, object: Object<'_>)
    {self.as_cooperative_ctx().set_actionscript_data(object).expect("INVARIANT: `CurrentContext` is always valid.");}

    /// Returns the associated context from an `ExtensionContext` object.
    /// 
    /// # Safety
    ///
    /// `context` must be another context constructed by the current
    /// extension via [`ContextInitializer`]. Otherwise, the invariants
    /// of [`CooperativeContext`] are violated, and its internal APIs
    /// for accessing native data will result in undefined behavior.
    /// 
    /// # Panics
    /// 
    /// Panics if `context` is not an `ExtensionContext` object,
    /// or if it is associated with the current context.
    /// 
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn cooperative_context_from_object(&self, context: Object<'a>) -> CooperativeContext<'a> {
        transmute(self.foreign_context_from_object(context))
    }

    /// Returns the associated context from an `ExtensionContext` object.
    /// 
    /// # Panics
    /// 
    /// Panics if `context` is not an `ExtensionContext` object,
    /// or if it is associated with the current context.
    /// 
    pub fn foreign_context_from_object(&self, context: Object<'a>) -> ForeignContext<'a> {
        let mut handle = MaybeUninit::<FREContext>::uninit();
        let r = unsafe {FREGetFREContextFromExtensionContext(context.as_ptr(), handle.as_mut_ptr())};
        let handle = unsafe {handle.assume_init()};
        assert!(r.is_ok(), "{}", FfiError::try_from(r).expect("The result must be error."));
        assert!(!handle.is_null());
        assert_ne!(handle, self.as_ptr(), "INVARIANT: `CurrentContext` is unique.");
        unsafe {transmute(handle)}
    }

}
impl<'a> CurrentContext<'a> {
    fn new(ctx: &'a FREContext) -> Self {
        Self(ContextHandle::new(*ctx).expect("INVARIANT: `CurrentContext` is always valid."), PhantomData)
    }
    fn as_cooperative_ctx(&self) -> CooperativeContext<'a> {unsafe {transmute(self.0)}}
    fn ctx_reg(&self) -> &ContextRegistry {
        let ptr = self.as_cooperative_ctx()
            .with(|ctx_reg|ctx_reg as *const ContextRegistry)
            .expect("INVARIANT: `CurrentContext` is unique.");
        unsafe {&*(ptr)}
    }
    fn ctx_reg_mut(&mut self) -> &mut ContextRegistry {
        let ptr = self.as_cooperative_ctx()
            .with_mut(|ctx_reg|ctx_reg as *mut ContextRegistry)
            .expect("INVARIANT: `CurrentContext` is unique.");
        unsafe {&mut *(ptr)}
    }
}
unsafe impl Sealed for CurrentContext<'_> {}
impl<'a> Context<'a> for CurrentContext<'a> {
    fn as_handle (&self) -> ContextHandle {self.0}
    fn is_valid(&self) -> bool {
        debug_assert!(self.as_cooperative_ctx().is_valid());
        true
    }
}


/// A handle to a context created by the current extension,
/// which may become invalid under specific conditions.
/// 
/// Can only be obtained through [`CurrentContext::cooperative_context_from_object`].
///
/// Invalidity only occurs after the associated `ExtensionContext` object
/// has been disposed. Therefore, callers should be prepared for operations
/// on the context to fail at appropriate points.
///
/// This crate leverages [`FREGetFREContextFromExtensionContext`] to enable
/// more advanced use cases, but doing so also increases overall complexity.
/// 
/// # Invariant
/// 
/// The context must have been constructed by the current extension via
/// [`ContextInitializer`], and it must not be the [`CurrentContext`].
/// Violating these invariants may results in undefined behavior.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct CooperativeContext <'a> (ContextHandle, PhantomData<&'a()>);
impl<'a> CooperativeContext<'a> {

    /// Provides immutable access to the [`ContextRegistry`] within a closure.
    /// 
    /// [`Err`]=> [`ContextError::InvalidContext`], [`ContextError::NullRegistry`], [`ContextError::InvalidRegistry`], [`ContextError::FfiCallFailed`], [`ContextError::BorrowRegistryConflict`];
    /// 
    pub fn with <F, R> (self, f: F) -> Result<R, ContextError>
    where F: FnOnce (&ContextRegistry) -> R {
        if !self.is_valid() {return Err(ContextError::InvalidContext)}
        let handle = self.as_foreign_ctx().get_native_data()?
            .ok_or(ContextError::NullRegistry)?;
        let cell = unsafe {RefCell::<ContextRegistry>::ref_from(handle)}
            .map_err(|_| ContextError::InvalidRegistry)?;
        let ctx_reg = cell.try_borrow()
            .map_err(|_| ContextError::BorrowRegistryConflict)?;
        let r = f(&ctx_reg);
        Ok(r)
    }

    /// Provides mutable access to the [`ContextRegistry`] within a closure.
    /// 
    /// [`Err`]=> [`Self::with`];
    /// 
    pub fn with_mut <F, R> (self, f: F) -> Result<R, ContextError>
    where F: FnOnce (&mut ContextRegistry) -> R {
        if !self.is_valid() {return Err(ContextError::InvalidContext)}
        let handle = self.as_foreign_ctx().get_native_data()?
            .ok_or(ContextError::NullRegistry)?;
        let cell = unsafe {RefCell::<ContextRegistry>::ref_from(handle)}
            .map_err(|_| ContextError::InvalidRegistry)?;
        let mut ctx_reg = cell.try_borrow_mut()
            .map_err(|_| ContextError::BorrowRegistryConflict)?;
        let r = f(&mut ctx_reg);
        Ok(r)
    }

    /// Calls a registered function by name through an internal call within the current extension.
    /// 
    /// Nested calls increase complexity. Callers must consider borrowing rules and context validity.
    /// 
    /// [`Err`]=> [`Self::with`], [`ContextError::MethodNotFound`];
    /// 
    pub fn call_method (self, name: &str, args: Option<&[Object<'a>]> ) -> Result<Object<'a>, ContextError> {
        let (func, data) = self.with(|ctx_reg| {
            ctx_reg.methods.as_ref()
            .expect("INVARIANT: `CooperativeContext` is not current context and has been initialized with `MethodSet`.")
            .get(name)
            .ok_or(ContextError::MethodNotFound)
        })??;
        let args = args.unwrap_or_default();
        let argc = args.len() as u32;
        let argv = args.as_ptr() as *const FREObject;
        let r = unsafe {func(self.as_ptr(), data, argc, argv)};
        Ok(unsafe {transmute(r)})
    }
    
    /// Returns the ActionScript-side Context Data associated with the context.
    /// 
    /// `ExtensionContext.actionScriptData`
    /// 
    /// Only fails if the context is invalid.
    /// 
    pub fn get_actionscript_data (self) -> Result<Object<'a>, FfiError>
    {self.as_foreign_ctx().get_actionscript_data()}
    
    
    /// Sets the ActionScript-side Context Data associated with the context.
    /// 
    /// `ExtensionContext.actionScriptData`
    /// 
    /// Only fails if the context is invalid.
    /// 
    pub fn set_actionscript_data (self, object: Object<'_>) -> Result<(), FfiError>
    {self.as_foreign_ctx().set_actionscript_data(object)}


    fn as_foreign_ctx(self) -> ForeignContext<'a> {unsafe {transmute(self)}}
}
unsafe impl Sealed for CooperativeContext<'_> {}
impl<'a> Context<'a> for CooperativeContext<'a> {
    fn as_handle (&self) -> ContextHandle {self.0}
    fn is_valid(&self) -> bool {self.get_actionscript_data().is_ok()}
}


/// A handle to a context that may become invalid under specific conditions.
/// 
/// Can only be obtained through [`CurrentContext::foreign_context_from_object`].
/// 
/// Assumes the context was NOT constructed by the current extension.
/// Accessing its associated native data is therefore unsafe.
///
/// Invalidity only occurs after the associated `ExtensionContext` object
/// has been disposed. Therefore, callers should be prepared for operations
/// on the context to fail at appropriate points.
///
/// This crate leverages [`FREGetFREContextFromExtensionContext`] to enable
/// more advanced use cases, but doing so also increases overall complexity.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ForeignContext <'a> (ContextHandle, PhantomData<&'a()>);
impl<'a> ForeignContext<'a> {
    
    /// Returns a pointer to the native data. Callers must understand its memory
    /// layout and explicitly treat it as either a borrow or a move.
    /// 
    /// Only fails if the context is invalid.
    /// 
    pub fn get_native_data (self) -> Result<Option<NonNullFREData>, FfiError> {
        let mut data = MaybeUninit::<FREData>::uninit();
        let r = unsafe {FREGetContextNativeData(self.as_ptr(), data.as_mut_ptr())};
        let data = unsafe {data.assume_init()};
        if let Ok(e) = FfiError::try_from(r) {Err(e)} else {Ok(NonNullFREData::new(data))}
    }

    /// Sets the native data pointer for the context.
    ///
    /// Only fails if the context is invalid.
    /// 
    /// # Safety
    ///
    /// Callers must ensure that no valid native data is currently set,
    /// or that any previously associated native data has become invalid
    /// and its memory has been properly released.
    ///
    /// If `data` is a non-null pointer, it must have well-defined ownership:
    /// it must be treated explicitly as either a borrow or a move.
    ///
    /// If treated as a move, callers must ensure that the memory is properly
    /// released before [`FREContextFinalizer`] completes.
    /// 
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn set_native_data (self, data: Option<NonNullFREData>) -> Result<(), FfiError> {
        let data: *mut c_void = data.map(|ptr|ptr.as_ptr())
            .unwrap_or_default();
        let r = FRESetContextNativeData(self.as_ptr(), data);
        if let Ok(e) = FfiError::try_from(r) {Err(e)} else {Ok(())}
    }

    /// Returns the ActionScript-side Context Data associated with the context.
    /// 
    /// `ExtensionContext.actionScriptData`
    /// 
    /// Only fails if the context is invalid.
    /// 
    pub fn get_actionscript_data (self) -> Result<Object<'a>, FfiError> {
        let mut object = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FREGetContextActionScriptData(self.as_ptr(), object.as_mut_ptr())};
        if let Ok(e) = FfiError::try_from(r) {Err(e)} else {Ok(unsafe {transmute(object)})}
    }
    
    /// Sets the ActionScript-side Context Data associated with the context.
    /// 
    /// `ExtensionContext.actionScriptData`
    /// 
    /// Only fails if the context is invalid.
    /// 
    pub fn set_actionscript_data (self, object: Object<'_>) -> Result<(), FfiError> {
        let r = unsafe {FRESetContextActionScriptData(self.as_ptr(), object.as_ptr())};
        if let Ok(e) = FfiError::try_from(r) {Err(e)} else {Ok(())}
    }

}
unsafe impl Sealed for ForeignContext<'_> {}
impl<'a> Context<'a> for ForeignContext<'a> {
    fn as_handle (&self) -> ContextHandle {self.0}
    fn is_valid(&self) -> bool {self.get_actionscript_data().is_ok()}
}


#[allow(private_bounds)]
pub trait Context<'a>: Sealed {
    fn as_handle (&self) -> ContextHandle;
    fn as_ptr (&self) -> FREContext {self.as_handle().as_ptr()}
    
    /// Returns whether the context is valid.
    ///
    /// The context remains valid until [`FREContextFinalizer`] has completed.
    /// Invalidity only occurs when the associated `ExtensionContext` object
    /// is destructed or its `dispose` method is explicitly called.
    /// 
    fn is_valid(&self) -> bool;

    /// Sends a message to the debugger output.
    /// 
    /// Delivery is not guaranteed; the `message` may not be presented.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use fre_rs::prelude::*;
    /// fn func <'a> (ctx: CurrentContext<'a>, args: &[Object<'a>]) {
    ///     ctx.trace("Hello, Flash runtime!");
    ///     ctx.trace(args);
    ///     ctx.trace(args[0]);
    /// }
    /// ```
    /// 
    fn trace (&self, message: impl ToUcstrLossy) {
        let r = unsafe {FRETrace(self.as_ptr(), message.to_ucstr_lossy().as_ptr())};
        debug_assert!(r.is_ok(), "{}", FfiError::try_from(r).unwrap());
    }

    /// Returns an [`EventDispatcher`] used to perform asynchronous callbacks
    /// via the AS3 event system.
    /// 
    fn event_dispatcher(&self) -> EventDispatcher {EventDispatcher(self.as_handle())}

    /// Returns the render mode of the stage.
    /// 
    /// If `stage` is [`None`], the main/initial stage is used.
    /// 
    fn get_render_mode (&self, stage: Option<as3::Stage<'a>>) -> RenderMode {
        let stage: Object = stage.into();
        let mut mode = MaybeUninit::<FRERenderMode>::uninit();
        let r = unsafe {FREGetRenderMode(self.as_ptr(), stage.as_ptr(), mode.as_mut_ptr())};
        debug_assert!(r.is_ok(), "{}", FfiError::try_from(r).unwrap());
        let mode = unsafe {mode.assume_init()};
        RenderMode::from(mode)
    }

    /// [`FRESetRenderSource`]
    /// 
    /// Returns [`Err`] if `display_object` has an incorrect type.
    /// 
    fn set_render_source (&self, source: as3::MediaBuffer<'a>, display_object: NonNullObject<'a>) -> Result<(), FfiError> {
        let r = unsafe {FRESetRenderSource(self.as_ptr(), source.as_ptr(), display_object.as_ptr())};
        if let Ok(e) = FfiError::try_from(r) {Err(e)} else {Ok(())}
    }

    /// [`FREMediaBufferLock`] [`FREMediaBufferUnlock`]
    /// 
    fn with_media_buffer <F, R> (&self, media_buffer: as3::MediaBuffer<'a>, f: F) -> R 
    where F: FnOnce (MediaBufferDataAdapter) -> R {
        let mut bytes = MaybeUninit::<FREBytes>::uninit();
        let mut width = MaybeUninit::<u32>::uninit();
        let mut height = MaybeUninit::<u32>::uninit();
        let mut stride = MaybeUninit::<u32>::uninit();
        let mut format = MaybeUninit::<u32>::uninit();
        let result = unsafe {FREMediaBufferLock(self.as_ptr(), media_buffer.as_ptr(), bytes.as_mut_ptr(), width.as_mut_ptr(), height.as_mut_ptr(), stride.as_mut_ptr(), format.as_mut_ptr())};
        debug_assert!(result.is_ok(), "{}", FfiError::try_from(result).unwrap());
        let (bytes, width, height, stride, format) = unsafe {(
            bytes.assume_init(),
            width.assume_init(),
            height.assume_init(),
            stride.assume_init(),
            format.assume_init(),
        )};
        assert!(!bytes.is_null());
        let adapter = unsafe {MediaBufferDataAdapter::new(bytes, width, height, stride, format)};
        let r = f(adapter);
        let result = unsafe {FREMediaBufferUnlock(self.as_ptr(), media_buffer.as_ptr(), u32::default())};
        debug_assert!(result.is_ok(), "{}", FfiError::try_from(result).unwrap());
        r
    }
}


/// The extension-side concrete representation of a [`Context`].
///
/// This can be considered the actual context instance, while [`Context`]
/// serves as an abstract handle or outer wrapper around it.
/// 
/// Can only be constructed through [`CurrentContext::with_context_initializer`].
/// 
#[derive(Debug)]
pub struct ContextRegistry {
    ctx_type: Option<UCStr>,
    ctx_data: Option<Box<dyn Any>>,
    ext_data: Option<ExtensionData>,// &'extension mut
    methods: Option<MethodSet>,
}
impl ContextRegistry {

    /// Returns the context type associated with the context.
    ///
    /// This corresponds to the `contextType` argument passed to
    /// `ExtensionContext.createExtensionContext`.
    ///
    /// Returns [`None`] if that argument was `null`.
    /// 
    pub fn context_type(&self) -> Option<UCStr> {(self.ctx_type).clone()}

    
    /// Returns an immutable reference to the Context Data associated with the context.
    ///
    /// Context Data is user-defined data bound to the context, sharing the same
    /// lifetime as the context itself.
    ///
    /// It can only be set via the first return value of [`ContextInitializer`].
    /// 
    pub fn context_data(&self) -> Option<&dyn Any> {self.ctx_data.as_ref().map(|d|d.as_ref())}

    /// Returns a mutable reference to the Context Data associated with the context.
    ///
    /// Context Data is user-defined data bound to the context, sharing the same
    /// lifetime as the context itself.
    ///
    /// It can only be set via the first return value of [`ContextInitializer`].
    /// 
    pub fn context_data_mut(&mut self) -> Option<&mut dyn Any> {self.ctx_data.as_mut().map(|d|d.as_mut())}

    /// Provides access to the Extension Data.
    ///
    /// The Extension Data is set from the return value of [`Initializer`].
    /// It can be accessed across threads and is synchronized via a [`Mutex`],
    /// providing exclusive access on each call.
    ///
    /// Calling this method within nested [`Function`] invocations can lead
    /// to deadlocks. It is recommended to avoid accessing it within a
    /// [`Function`] call stack, and instead perform synchronization between
    /// Context Data and Extension Data in [`ContextInitializer`] and [`ContextFinalizer`].
    /// 
    pub fn with_extension_data <F, R> (&self, f: F) -> Option<R>
    where F: FnOnce (&mut dyn Any) -> R {
        let ext_data = self.ext_data.as_ref()?;
        let mut ext_data = ext_data.lock().expect("Mutex poisoned.");
        let r = f(ext_data.as_mut());
        Some(r)
    }
    
    fn new (ext_data: FREData, ctx_type: Option<UCStr>) -> RefCell<Self> {
        let ext_data = NonNullFREData::new(ext_data)
            .map(|raw| {
                Arc::clone(unsafe {
                    <ExtensionData as Data>::ref_from(raw)
                }.unwrap())
            });
        RefCell::new(Self {
            ctx_type,
            ctx_data: None,
            ext_data,
            methods: None,
        })
    }
}
impl Data for RefCell<ContextRegistry> {}


pub(crate) mod stack {
    use super::*;
    thread_local! {static STACK: RefCell<Vec<ContextHandle>> = RefCell::new(Vec::new());}
    pub(super) fn push(ctx: ContextHandle) {STACK.with_borrow_mut(|stack|stack.push(ctx));}
    pub(super) fn pop() -> Option<ContextHandle> {STACK.with_borrow_mut(|stack|stack.pop())}
    pub(crate) fn current_context<'a> () -> CurrentContext<'a> {
        let handle = STACK.with_borrow(|stack|stack.last().cloned())
            .expect("Outside the scope of a Flash runtime function call; no context available.");
        unsafe {transmute(handle)}
    }
    
    /// A wrapper used by [`FREContextInitializer`], [`FREContextFinalizer`], and
    /// [`FREFunction`] that provides a safe stack-level execution environment.
    ///
    /// **In typical usage of this crate, this function should not be called directly.**
    /// 
    /// # Safety
    /// 
    /// While all operations performed within this function are safe at the stack level,
    /// calling this function itself is unsafe and requires that all input arguments
    /// are valid. In particular, this function assumes it is invoked directly with
    /// arguments provided by the Flash runtime.
    ///
    /// Violating these assumptions may lead to undefined behavior.
    /// 
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn with <'a, F, R> (ctx: &'a FREContext, f: F) -> R
    where
        F: FnOnce (&CurrentContext<'a>) -> R,
        R: 'a,
    {
        let ctx = CurrentContext::new(ctx);
        stack::push(ctx.0);
        let r = f(&ctx);
        let popped = stack::pop();
        debug_assert_eq!(ctx.0, popped.expect("Context unexpectedly missing from stack."), "The context pushed at the beginning of this function must be popped at the end.");
        r
    }
    
    /// A wrapper around [`FREContextInitializer`] that provides a safe stack-level
    /// execution environment for context initialization.
    /// 
    /// **In typical usage of this crate, this function should not be called directly.**
    ///
    /// # Safety
    /// 
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
    /// 
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn with_initializer <F> (
        ext_data: FREData,// &'extension mut
        ctx_type: FREStr,// &'function
        ctx: &FREContext,// &'function
        num_funcs_to_set: *mut u32,// return
        funcs_to_set: *mut *const FRENamedFunction,// return &'context mut
        f: F
    )
    where F: for<'a> FnOnce (&'a CurrentContext<'a>) -> (Option<Box<dyn Any>>, FunctionSet)
    {
        assert!(!num_funcs_to_set.is_null());
        assert!(!funcs_to_set.is_null());
        stack::with(ctx, |ctx|{
            let ctx_type = if ctx_type.is_null() {None} else {
                let ctx_type = CStr::from_ptr(ctx_type as *const c_char);
                let ctx_type = UCStr::try_from(ctx_type).expect("Input string is not valid UTF-8.");
                Some(ctx_type)
            };
            let ctx_reg = ContextRegistry::new(ext_data, ctx_type).into_raw();
            let r = ForeignContext(ctx.0, PhantomData).set_native_data(Some(ctx_reg));// <'context> move
            assert!(r.is_ok());
            let (ctx_data, funcs) = f(ctx);
            let methods = MethodSet::from(funcs);
            let r = methods.as_ref();
            debug_assert!(r.len() <= u32::MAX as usize);
            *num_funcs_to_set = r.len() as u32;
            *funcs_to_set = r.as_ptr();
            let ctx_reg = RefCell::<ContextRegistry>::mut_from(ctx_reg).unwrap();
            let mut ctx_reg_mut = ctx_reg.borrow_mut();
            ctx_reg_mut.ctx_data = ctx_data;
            ctx_reg_mut.methods = Some(methods);
        })
    }

    /// A wrapper around [`FREFunction`] that provides a safe stack-level execution
    /// environment for the given closure.
    /// 
    /// **In typical usage of this crate, this function should not be called directly.**
    ///
    /// # Safety
    /// 
    /// While all operations performed within this function are safe at the stack level,
    /// calling this function itself is unsafe and requires the following conditions:
    ///
    /// - `func_data` must either be constructed via [`Data::into_raw`] before
    ///   [`ContextInitializer`] returns, or be a null pointer.
    /// - This function assumes it is invoked directly with arguments provided by the
    ///   Flash runtime, meaning all arguments must be valid and consistent.
    ///
    /// Violating these assumptions may lead to undefined behavior.
    /// 
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn with_method <'a, F, R> (
        ctx: &'a FREContext,// &'function
        func_data: FREData,// &'function mut
        argc: u32,
        argv: *const FREObject,// &'function
        f: F
    ) -> FREObject
    where
        F: FnOnce (&CurrentContext<'a>, Option<&mut dyn Any>, &[Object<'a>]) -> R,
        R: Into<Object<'a>> + 'a
    {
        assert!(!argv.is_null() || argc==0);
        stack::with(ctx, |ctx| {
            let func_data = NonNullFREData::new(func_data)
                .map(|raw| crate::data::mut_from(raw));
            let args = std::slice::from_raw_parts(argv as *const Object, argc as usize);
            f(ctx, func_data, args).into().as_ptr()
        })
    }
}


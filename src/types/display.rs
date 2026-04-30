use super::*;


crate::class! {@Typeof
    /// A reference to the AS3 object `flash.display.BitmapData`.
    /// 
    /// Some properties and methods are not yet implemented.
    /// 
    BitmapData
}
impl<'a> BitmapData<'a> {
    /// During the closure call stack, the Flash runtime is in a restricted state
    /// where most APIs are unavailable, and [`Sync`] is used to prevent illegal
    /// FFI call ordering.
    /// 
    pub fn with <F, R> (self, f: F) -> R
    where F: Sync + FnOnce (BitmapDataAdapter) -> R
    {
        let mut descriptor = MaybeUninit::<FREBitmapData2>::uninit();
        let result = unsafe {FREAcquireBitmapData2(self.as_ptr(), descriptor.as_mut_ptr())};
        debug_assert!(result.is_ok(), "{}", FfiError::try_from(result).unwrap());
        let descriptor = unsafe {descriptor.assume_init()};
        let r = f(unsafe {BitmapDataAdapter::new(self.as_ptr(), descriptor)});
        let result = unsafe {FREReleaseBitmapData(self.as_ptr())};
        debug_assert!(result.is_ok(), "{}", FfiError::try_from(result).unwrap());
        r
    }
}


crate::class! {
    /// A reference to the AS3 object `flash.display.NativeWindow`.
    /// 
    /// Some properties and methods are not yet implemented.
    /// 
    NativeWindow
}
impl<'a> NativeWindow<'a> {
    pub fn get_stage(self) -> Stage<'a> {
        let object = self.get_property(crate::ucstringify!(stage)).unwrap();
        assert!(!object.is_null());
        unsafe {object.as_unchecked()}
    }

    /// Passes the underlying native handle to the provided closure.
    /// 
    /// The [`NonNullHandle`] is only valid for the duration of this closure call.
    /// 
    /// During the closure call stack, the Flash runtime is in a restricted state
    /// where most APIs are unavailable, and [`Sync`] is used to prevent illegal
    /// FFI call ordering.
    /// 
    pub fn with <F, R> (self, f: F) -> R
    where F: Sync + FnOnce (NonNullHandle) -> R
    {
        let mut handle = MaybeUninit::<FRENativeWindow>::uninit();
        let result = unsafe {FREAcquireNativeWindowHandle(self.as_ptr(), handle.as_mut_ptr())};
        assert!(result.is_ok());
        let handle = unsafe {handle.assume_init()};
        let handle = NonNullHandle::new(handle).unwrap();
        let r = f(handle);
        let result = unsafe {FREReleaseNativeWindowHandle(self.as_ptr())};
        debug_assert!(result.is_ok());
        r
    }
}


crate::class! {
    /// A reference to the AS3 object `flash.display.Stage`.
    /// 
    /// Some properties and methods are not yet implemented.
    /// 
    Stage
}
impl<'a> Stage<'a> {

    /// The render mode of this stage.
    ///
    /// Use [`Context::get_render_mode`] with [`None`] for the main/initial stage.
    /// 
    pub fn render_mode(self) -> RenderMode {crate::context::stack::current_context().get_render_mode(Some(self))}

    #[allow(non_snake_case)]
    pub fn get_stage3Ds(self) -> Box<[Stage3D<'a>]> {
        let object = self.get_property(crate::ucstringify!(stage3Ds)).unwrap();
        assert!(!object.is_null());
        let vector: Vector = unsafe {object.as_unchecked()};
        let stage_3ds: Box<[Stage3D]> = vector.iter()
            .map(|stage_3d|unsafe {stage_3d.as_unchecked()})
            .collect();
        stage_3ds
    }
}


crate::class! {
    /// A reference to the AS3 object `flash.display.Stage3D`.
    /// 
    /// Some properties and methods are not yet implemented.
    /// 
    Stage3D
}
impl<'a> Stage3D<'a> {
    #[allow(non_snake_case)]
    pub fn get_context3D(self) -> Option<Context3D<'a>> {
        let object = self.get_property(crate::ucstringify!(context3D)).unwrap();
        if object.is_null() {None} else {Some(unsafe {object.as_unchecked()})}
    }
}


crate::class! {
    /// A reference to the AS3 object `flash.display3D.Context3D`.
    /// 
    /// Some properties and methods are not yet implemented.
    /// 
    Context3D
}
impl<'a> Context3D<'a> {
    /// Returns the underlying native handle.
    /// 
    /// See [`FREGetNativeContext3DHandle`] for details.
    /// 
    pub fn raw (self) -> NonNullHandle {
        let mut handle = MaybeUninit::<FREHandle>::uninit();
        let r = unsafe {FREGetNativeContext3DHandle(self.as_ptr(), handle.as_mut_ptr())};
        debug_assert!(r.is_ok());
        let handle = unsafe {handle.assume_init()};
        NonNullHandle::new(handle).unwrap()
    }
}


crate::class! {
    /// A reference to the AS3 object `air.media.MediaBuffer`.
    /// 
    /// Some properties and methods are not yet implemented.
    ///
    MediaBuffer
}
impl<'a> MediaBuffer<'a> {

    /// [`FRESetRenderSource`]
    /// 
    /// Returns [`Err`] if `display_object` has an incorrect type.
    /// 
    pub fn render_to (self, display_object: NonNullObject<'_>) -> Result<(), FfiError> {crate::context::stack::current_context().set_render_source(self, display_object)}

    /// [`FREMediaBufferLock`] [`FREMediaBufferUnlock`]
    /// 
    pub fn with <F, R> (self, f: F) -> R
    where F: FnOnce (MediaBufferDataAdapter) -> R {crate::context::stack::current_context().with_media_buffer(self, f)}
}


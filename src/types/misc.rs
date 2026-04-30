use super::*;


crate::class! {@Typeof
    /// A reference to the AS3 object `Array`.
    /// 
    /// Some methods are not yet implemented.
    /// 
    Array
}
impl<'a> Array<'a> {
    pub fn get_length (self) -> u32 {
        let mut value = MaybeUninit::<u32>::uninit();
        let r = unsafe {FREGetArrayLength(self.as_ptr(), value.as_mut_ptr())};
        debug_assert!(r.is_ok());
        unsafe {value.assume_init()}
    }
    /// [`Err`]=> [FfiError::InsufficientMemory];
    /// 
    pub fn set_length (self, value: u32) -> Result<(), FfiError> {
        let r = unsafe {FRESetArrayLength(self.as_ptr(), value)};
        if let Ok(e) = FfiError::try_from(r) {
            Err(e)
        } else {Ok(())}
    }
    pub fn get (self, index: u32) -> Object<'a> {
        let mut object = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FREGetArrayElementAt(self.as_ptr(), index, object.as_mut_ptr())};
        debug_assert!(r.is_ok());
        unsafe {transmute(object)}
    }
    pub fn set <O: AsObject<'a>> (self, index: u32, value: O) {
        let r = unsafe {FRESetArrayElementAt(self.as_ptr(), index, value.as_ptr())};
        debug_assert!(r.is_ok());
    }
    pub fn new (ctx: &CurrentContext<'a>, num_elements: Option<NonNegativeInt>) -> Self {
        let mut arg = as3::null;
        let num_elements = num_elements.map(|v|{
            arg = int::new(ctx, v.get()).as_object();
            std::slice::from_ref(&arg)
        });
        unsafe {ctx.construct(crate::ucstringify!(Array), num_elements)
            .unwrap()
            .as_unchecked()}
    }
    pub fn from_slice (ctx: &CurrentContext<'a>, elements: &[Object<'a>]) -> Self {
        debug_assert!(elements.len() <= i32::MAX as usize);
        if elements.len() == 1 && elements[0].get_type() == Type::Number {
            let arr = Self::new(ctx, NonNegativeInt::new(1));
            arr.set(0, *unsafe {elements.get_unchecked(0)});
            arr
        } else {
            unsafe {ctx.construct(crate::ucstringify!(Array), Some(elements))
                .unwrap()
                .as_unchecked()}
        }
    }
    pub fn extend_from_slice (self, elements: &[Object]) -> u32 {
        self.call_method(crate::ucstringify!(push), Some(elements))
            .unwrap()
            .try_into()
            .unwrap()
    }
    pub fn push <O: AsObject<'a>> (self, element: O) -> u32 {
        let element = element.as_object();
        let elements = std::slice::from_ref(&element);
        self.extend_from_slice(elements)
    }
    pub fn iter(self) -> ArrayIter<'a> {ArrayIter::new(self)}
}
impl<'a> IntoIterator for Array<'a> {
    type Item = Object<'a>;
    type IntoIter = ArrayIter<'a>;
    fn into_iter(self) -> Self::IntoIter {self.iter()}
}


crate::class! {@Typeof
    /// A reference to the AS3 object `Vector.<*>`.
    /// 
    /// Some properties and methods are not yet implemented.
    /// 
    Vector
}
impl<'a> Vector<'a> {
    pub fn get_length (self) -> u32 {
        let mut value = MaybeUninit::<u32>::uninit();
        let r = unsafe {FREGetArrayLength(self.as_ptr(), value.as_mut_ptr())};
        debug_assert!(r.is_ok());
        unsafe {value.assume_init()}
    }

    /// [`Err`]=> [FfiError::InsufficientMemory], [FfiError::ReadOnly];
    /// 
    pub fn set_length (self, value: u32) -> Result<(), FfiError> {
        let r = unsafe {FRESetArrayLength(self.as_ptr(), value)};
        if let Ok(e) = FfiError::try_from(r) {
            Err(e)
        } else {Ok(())}
    }

    /// [`Err`]=> [FfiError::InvalidArgument];
    /// 
    pub fn get (self, index: u32) -> Result<Object<'a>, FfiError> {
        let mut object = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FREGetArrayElementAt(self.as_ptr(), index, object.as_mut_ptr())};
        if let Ok(e) = FfiError::try_from(r) {
            Err(e)
        } else {
            Ok(unsafe {transmute(object)})
        }
    }

    /// [`Err`]=> [FfiError::TypeMismatch];
    /// 
    pub fn set <O: AsObject<'a>> (self, index: u32, value: O) -> Result<(), FfiError> {
        let r = unsafe {FRESetArrayElementAt(self.as_ptr(), index, value.as_ptr())};
        if let Ok(e) = FfiError::try_from(r) {
            Err(e)
        } else {Ok(())}
    }
    pub fn iter(self) -> VectorIter<'a> {VectorIter::new(self)}
}
impl<'a> IntoIterator for Vector<'a> {
    type Item = Object<'a>;
    type IntoIter = VectorIter<'a>;
    fn into_iter(self) -> Self::IntoIter {self.iter()}
}


crate::class! {@Typeof
    /// A reference to the AS3 object `flash.utils.ByteArray`.
    /// 
    /// Some properties and methods are not yet implemented.
    /// 
    ByteArray
}
impl<'a> ByteArray<'a> {
    pub fn new (_: &CurrentContext<'a>, length: u32) -> Self {
        let descriptor = FREByteArray {length, bytes: std::ptr::null_mut()};
        let mut object = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FRENewByteArray(transmute(&descriptor), object.as_mut_ptr())};
        debug_assert!(r.is_ok());
        let object = unsafe {object.assume_init()};
        assert!(!object.is_null());
        unsafe {transmute(object)}
    }
    pub fn from_bytes (_: &CurrentContext<'a>, bytes: impl AsRef<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        debug_assert!(bytes.len() <= u32::MAX as usize);
        let descriptor = FREByteArray {length: bytes.len() as u32, bytes: bytes.as_ptr() as FREBytes};
        let mut object = MaybeUninit::<FREObject>::uninit();
        let r = unsafe {FRENewByteArray(transmute(&descriptor), object.as_mut_ptr())};
        debug_assert!(r.is_ok());
        let object = unsafe {object.assume_init()};
        assert!(!object.is_null());
        unsafe {transmute(object)}
    }

    /// During the closure call stack, the Flash runtime is in a restricted state
    /// where most APIs are unavailable, and [`Sync`] is used to prevent illegal
    /// FFI call ordering.
    /// 
    pub fn with <F, R> (self, f: F) -> R
    where F: Sync + FnOnce (&mut [u8]) -> R
    {
        let mut descriptor = MaybeUninit::<FREByteArray>::uninit();
        let result = unsafe {FREAcquireByteArray(self.as_ptr(), descriptor.as_mut_ptr())};
        assert!(result.is_ok());
        let descriptor = unsafe {descriptor.assume_init()};
        assert!(!descriptor.bytes.is_null());
        let bytes = unsafe {std::slice::from_raw_parts_mut(descriptor.bytes, descriptor.length as usize)};
        let r = f(bytes);
        let result = unsafe {FREReleaseByteArray(self.as_ptr())};
        debug_assert!(result.is_ok());
        r
    }
}


crate::class! {
    /// A reference to the AS3 object `Error`.
    /// 
    Error
}
impl<'a> Error<'a> {
    #[allow(non_snake_case)]
    pub fn get_errorID(self) -> i32 {
        let value = self.get_property(crate::ucstringify!(errorID)).unwrap().try_into().unwrap();
        value
    }

    pub fn get_message(self) -> Option<as3::String<'a>> {
        let value = self.get_property(crate::ucstringify!(message)).unwrap().try_as().ok();
        value
    }
    pub fn set_message(self, value: Option<as3::String>) {
        let r = self.set_property(crate::ucstringify!(message), Object::from(value));
        debug_assert!(r.is_ok());
    }

    pub fn get_name(self) -> Option<as3::String<'a>> {
        let value = self.get_property(crate::ucstringify!(name)).unwrap().try_as().ok();
        value
    }
    pub fn set_name(self, value: Option<as3::String>) {
        let r = self.set_property(crate::ucstringify!(name), Object::from(value));
        debug_assert!(r.is_ok());
    }
    
    pub fn new(ctx: &CurrentContext<'a>, message: Option<&str>, id: i32) -> Self {
        let message = message.map(|s|as3::String::new(ctx, s));
        let id = int::new(ctx, id);
        let args = vec![message.into(), id.as_object()].into_boxed_slice();
        let object= ctx.construct(crate::ucstringify!(Error), Some(args.as_ref())).unwrap();
        unsafe {object.as_unchecked()}
    }
}


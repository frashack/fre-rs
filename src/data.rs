//! 
//! This module provides safe conversions between concrete typed instances
//! and raw pointers for native data.
//! 
//! [`Box<dyn Any>`] is a fat pointer stored on the stack. Casting it directly
//! to a raw pointer will truncate its metadata. To preserve type information,
//! the fat pointer must be stored intact on the heap, so it can be safely
//! reconstructed from a raw pointer.
//! 


use super::*;


pub(crate) type ExtensionData = Arc<Mutex<Box<dyn Any>>>;
impl Data for ExtensionData {}


/// Should be implemented for all native data types to ensure safe pointer passing
/// and correct memory management. Native data includes Extension Data, Context Data,
/// and Function data.
/// 
pub trait Data: 'static + Sized {
    fn into_boxed (self) -> Box<dyn Any> {
        Box::new(self) as Box<dyn Any>
    }

    fn from_boxed (boxed: Box<dyn Any>) -> Result<Self, Box<dyn Any>> {
        boxed.downcast()
            .map(|b| *b)
            .map_err(|b| b)
    }

    /// **In typical usage of this crate, this function should not be called directly.**
    /// 
    fn into_raw (self) -> NonNullFREData {
        super::data::into_raw(self.into_boxed())
    }

    /// **In typical usage of this crate, this function should not be called directly.**
    /// 
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn from_raw (raw: NonNullFREData) -> Self {
        let boxed = super::data::from_raw(raw);
        Self::from_boxed(boxed).unwrap()
    }

    /// **In typical usage of this crate, this function should not be called directly.**
    /// 
    /// # Safety
    /// 
    /// The returned reference has an unbounded lifetime and is not tied to any input.
    /// Correct and safe usage requires the caller to impose additional lifetime constraints.
    /// 
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn ref_from <'a> (raw: NonNullFREData) -> Result<&'a Self, &'a dyn Any> {
        let any = super::data::ref_from(raw);
        any.downcast_ref().ok_or(any)
    }

    /// **In typical usage of this crate, this function should not be called directly.**
    /// 
    /// # Safety
    /// 
    /// The returned reference has an unbounded lifetime and is not tied to any input.
    /// Correct and safe usage requires the caller to impose additional lifetime constraints.
    /// 
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn mut_from <'a> (raw: NonNullFREData) -> Result<&'a mut Self, &'a mut dyn Any> {
        let fat = super::data::mut_from(raw) as *mut dyn Any;
        (&mut (*fat)).downcast_mut().ok_or(&mut (*fat))
    }
}
impl Data for () {}


type DataPointer = *mut *mut (dyn Any + 'static);


/// **In typical usage of this crate, this function should not be called directly.**
/// 
pub fn into_raw (boxed: Box<dyn Any>) -> NonNullFREData {
    let fat = Box::into_raw(boxed);
    let raw: DataPointer = Box::into_raw(Box::new(fat));
    unsafe {NonNull::new_unchecked(raw as FREData)}
}

/// **In typical usage of this crate, this function should not be called directly.**
/// 
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn from_raw (raw: NonNullFREData) -> Box<dyn Any> {
    let raw = raw.as_ptr() as DataPointer;
    let fat = Box::from_raw(raw);
    let boxed = Box::from_raw(*fat);
    boxed
}

/// **In typical usage of this crate, this function should not be called directly.**
/// 
/// # Safety
/// 
/// The returned reference has an unbounded lifetime and is not tied to any input.
/// Correct and safe usage requires the caller to impose additional lifetime constraints.
/// 
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn ref_from <'a> (raw: NonNullFREData) -> &'a dyn Any {
    let raw = raw.as_ptr() as DataPointer;
    let any = &(**raw);
    any
}

/// **In typical usage of this crate, this function should not be called directly.**
/// 
/// # Safety
/// 
/// The returned reference has an unbounded lifetime and is not tied to any input.
/// Correct and safe usage requires the caller to impose additional lifetime constraints.
/// 
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn mut_from <'a> (raw: NonNullFREData) -> &'a mut dyn Any {
    let raw = raw.as_ptr() as DataPointer;
    let any = &mut (**raw);
    any
}

/// **In typical usage of this crate, this function should not be called directly.**
/// 
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn drop_from (raw: NonNullFREData) {
    drop(from_raw(raw));
}
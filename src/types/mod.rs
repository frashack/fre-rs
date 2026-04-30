//! 
//! Abstractions over AS3 objects for integration with its type system.
//! 
//! Use classes from this module via [`as3`] to avoid path dependencies.
//! 


pub mod display;
pub mod misc;
pub mod object;
pub mod primitive;


use super::*;
use misc::*;
use object::*;
use primitive::*;


#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    
    // Supported by `FREGetObjectType`.
    NonNullObject,
    Number,
    String,
    ByteArray,
    Array,
    Vector,
    BitmapData,
    Boolean,
    null,

    // Not supported by `FREGetObjectType`.
    Named(&'static str),
    Unexpected(FREObjectType),

}
impl Type {
    pub fn is_null(self) -> bool {self == Self::null}
}
impl From<FREObjectType> for Type {
    fn from(value: FREObjectType) -> Self {
        match value {
            FREObjectType::FRE_TYPE_OBJECT      => Self::NonNullObject,
            FREObjectType::FRE_TYPE_NUMBER      => Self::Number,
            FREObjectType::FRE_TYPE_STRING      => Self::String,
            FREObjectType::FRE_TYPE_BYTEARRAY   => Self::ByteArray,
            FREObjectType::FRE_TYPE_ARRAY       => Self::Array,
            FREObjectType::FRE_TYPE_VECTOR      => Self::Vector,
            FREObjectType::FRE_TYPE_BITMAPDATA  => Self::BitmapData,
            FREObjectType::FRE_TYPE_BOOLEAN     => Self::Boolean,
            FREObjectType::FRE_TYPE_NULL        => Self::null,
            _ => Self::Unexpected(value),
        }
    }
}
impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Named(name) => write!(f, "{name}"),
            Self::Unexpected(ty) => write!(f, "Unexpected FREObjectType({ty})."),
            _ => write!(f, "{self:?}"),
        }
    }
}


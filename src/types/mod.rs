mod classes;
mod object;
mod primitive;

pub use {
    classes::*,
    object::*,
    primitive::*,
};

use super::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Object,
    Number,
    String,
    ByteArray,
    Array,
    Vector,
    BitmapData,
    Boolean,
    Null,
    Named(&'static str),
    Context3D,
    Error,
    Unexpected(FREObjectType),
}
impl Type {
    pub fn is_null(self) -> bool {self == Self::Null}
}
impl From<FREObjectType> for Type {
    fn from(value: FREObjectType) -> Self {
        match value {
            FREObjectType::FRE_TYPE_OBJECT      => Self::Object,
            FREObjectType::FRE_TYPE_NUMBER      => Self::Number,
            FREObjectType::FRE_TYPE_STRING      => Self::String,
            FREObjectType::FRE_TYPE_BYTEARRAY   => Self::ByteArray,
            FREObjectType::FRE_TYPE_ARRAY       => Self::Array,
            FREObjectType::FRE_TYPE_VECTOR      => Self::Vector,
            FREObjectType::FRE_TYPE_BITMAPDATA  => Self::BitmapData,
            FREObjectType::FRE_TYPE_BOOLEAN     => Self::Boolean,
            FREObjectType::FRE_TYPE_NULL        => Self::Null,
            _ => Self::Unexpected(value),
        }
    }
}
impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Named(name) => write!(f, "{name}"),
            Self::Unexpected(ty) => write!(f, "Unexpected FREObjectType. ({ty:#08X})"),
            _ => write!(f, "{self:?}"),
        }
    }
}
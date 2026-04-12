//! 
//! Miscellaneous items that do not yet have a clear module placement.
//! Their current structure is not ideal and may be refactored as the crate
//! evolves or as the underlying ANE C API improves.
//! 


use super::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Unknown,
    None,
    Cpu,
    DirectOgles,
    DirectOgl,
    DirectD3d9,
    DirectD3d10,
    DirectD3d11,
    SoftwareGdi,
    GpuOgles,
    Unexpected(FRERenderMode),
}
impl From<FRERenderMode> for RenderMode {
    fn from(value: FRERenderMode) -> Self {
        match value {
            FRERenderMode::FRE_RENDERMODE_UNKNOWN           => Self::Unknown,
            FRERenderMode::FRE_RENDERMODE_NONE              => Self::None,
            FRERenderMode::FRE_RENDERMODE_CPU               => Self::Cpu,
            FRERenderMode::FRE_RENDERMODE_DIRECT_OGLES      => Self::DirectOgles,
            FRERenderMode::FRE_RENDERMODE_DIRECT_OGL        => Self::DirectOgl,
            FRERenderMode::FRE_RENDERMODE_DIRECT_D3D9       => Self::DirectD3d9,
            FRERenderMode::FRE_RENDERMODE_DIRECT_D3D10      => Self::DirectD3d10,
            FRERenderMode::FRE_RENDERMODE_DIRECT_D3D11      => Self::DirectD3d11,
            FRERenderMode::FRE_RENDERMODE_SOFTWARE_GDI      => Self::SoftwareGdi,
            FRERenderMode::FRE_RENDERMODE_GPU_OGLES         => Self::GpuOgles,
            _ => Self::Unexpected(value),
        }
    }
}
impl Display for RenderMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Unknown           => write!(f, "Unknown"),
            Self::None              => write!(f, "None"),
            Self::Cpu               => write!(f, "CPU"),
            Self::DirectOgles       => write!(f, "Direct (OpenGL ES)"),
            Self::DirectOgl         => write!(f, "Direct (OpenGL)"),
            Self::DirectD3d9        => write!(f, "Direct (Direct3D 9)"),
            Self::DirectD3d10       => write!(f, "Direct (Direct3D 10)"),
            Self::DirectD3d11       => write!(f, "Direct (Direct3D 11)"),
            Self::SoftwareGdi       => write!(f, "Software (GDI)"),
            Self::GpuOgles          => write!(f, "GPU (OpenGL ES)"),
            Self::Unexpected(rm) => write!(f, "Unexpected FRERenderMode. ({rm:#08X})"),
        }
    }
}
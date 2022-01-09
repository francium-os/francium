#[cfg(feature = "platform_virt")]
pub mod virt;
#[cfg(feature = "platform_virt")]
pub use virt::*;

#[cfg(feature = "platform_raspi4")]
pub mod raspi4;
#[cfg(feature = "platform_raspi4")]
pub use raspi4::*;
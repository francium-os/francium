#[cfg(feature = "platform_virt")]
pub mod virt;
#[cfg(feature = "platform_virt")]
pub use virt::*;

#[cfg(feature = "platform_raspi3")]
pub mod raspi3;
#[cfg(feature = "platform_raspi3")]
pub use raspi3::*;

#[cfg(feature = "platform_raspi4")]
pub mod raspi4;
#[cfg(feature = "platform_raspi4")]
pub use raspi4::*;

#[cfg(feature = "platform_pc")]
pub mod pc;
#[cfg(feature = "platform_pc")]
pub use pc::*;

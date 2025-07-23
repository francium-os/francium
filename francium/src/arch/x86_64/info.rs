use common::FramebufferInfo;

pub static mut SYSTEM_INFO_RSDP_ADDR: Option<u64> = None;
pub static mut FRAMEBUFFER_INFO: Option<FramebufferInfo> = None;

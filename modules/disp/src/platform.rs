use common::system_info::{SystemInfo, SystemInfoType};
use francium_common::types::{MapType, PagePermission, FramebufferInfo};
use process::ipc;
use process::syscalls;

#[derive(Debug)]
pub struct PlatformFramebuffer {
    framebuffer_virt: usize,
    pub info: FramebufferInfo,
}

impl<'a> PlatformFramebuffer {
    pub fn new() -> Option<PlatformFramebuffer> {
        let Ok(SystemInfo::FramebufferInfo(info)) = syscalls::get_system_info(SystemInfoType::FramebufferInfo, 0) else {
            panic!("No platform framebuffer!");
        };

        // TODO: Move this to be shared memory. But that requires the concept of shared memory.
        let fb_virt = syscalls::map_device_memory(
            info.phys_addr,
            0,
            info.size,
            MapType::NormalCachable,
            PagePermission::USER_READ_WRITE,
        )
        .unwrap();

        let adapter = PlatformFramebuffer {
            info: info,
            framebuffer_virt: fb_virt,
        };

        Some(adapter)
    }

    pub fn get_framebuffer(&self) -> &'a mut [u32] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.framebuffer_virt as *mut u32,
                self.info.stride * self.info.height,
            )
        }
    }
}

use common::os_error::{ResultCode, RESULT_OK};
use common::system_info::*;

pub fn svc_get_system_info(ty: SystemInfoType, _index: usize, out_ptr: *mut SystemInfo) -> ResultCode {
    match ty {
        SystemInfoType::Platform => {
            #[cfg(feature = "platform_pc")]
            {
                unsafe { *out_ptr = SystemInfo::Platform(Platform::Pc); }
                RESULT_OK
            }

            #[cfg(feature = "platform_virt")]
            {
                unsafe { *out_ptr = SystemInfo::Platform(Platform::Virt); }
                RESULT_OK
            }

            #[cfg(feature = "platform_raspi3")]
            {
                unsafe { *out_ptr = SystemInfo::Platform(Platform::Raspi3); }
                RESULT_OK
            }

            #[cfg(feature = "platform_raspi4")]
            {
                unsafe { *out_ptr = SystemInfo::Platform(Platform::Raspi4); }
                RESULT_OK
            }
        },
        _ => {
            unimplemented!();
        }
    }
}

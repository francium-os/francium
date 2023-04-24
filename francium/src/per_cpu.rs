use crate::arch;
use crate::process::Thread;
use alloc::sync::Arc;

#[cfg(target_arch = "x86_64")]
use francium_x86::gdt::*;

// The entirety of the layout constraints here is that per_cpu_ptr stays at 0. repr(Rust) doesn't even guarantee that.
// Align to page to try and make sure the GDT/TSS don't cross page boundaries.
#[repr(C)]
#[repr(align(4096))]
pub struct PerCpuData {
    pub per_cpu_ptr: usize,
    pub saved_kernel_stack: usize,
    #[cfg(target_arch = "x86_64")]
    pub gdt: [GDTEntry; 8],
    #[cfg(target_arch = "x86_64")]
    pub tss: TSS,
    pub current_thread: Option<Arc<Thread>>,
    pub idle_thread: Option<Arc<Thread>>,
}

const _: () = assert!(core::mem::size_of::<PerCpuData>() <= 0x1000);

pub fn get() -> &'static mut PerCpuData {
    unsafe {
        (arch::get_per_cpu_base() as *mut PerCpuData)
            .as_mut()
            .unwrap()
    }
}

pub unsafe fn get_base() -> usize {
    arch::get_per_cpu_base()
}

pub fn get_current_thread() -> Arc<Thread> {
    get().current_thread.as_ref().unwrap().clone()
}

pub fn set_current_thread(a: Arc<Thread>) {
    get().current_thread = Some(a);
}

use core::arch::asm;

pub fn get_vendor() -> [u8; 12] {
    let mut vendor: [u8; 12] = [0; 12];
    unsafe {
        asm!("push rbx

		  mov eax, 0
	      cpuid
		  mov [{vendor}], ebx
		  mov [{vendor} + 4], edx
		  mov [{vendor} + 8], ecx

		  pop rbx", vendor = in(reg) vendor.as_mut_ptr());
    }
    vendor
}

// Leaf 1 Processor Info and Feature Bits: ecx bit 31
pub fn is_hypervisor_present() -> bool {
    let mut ecx: u32;
    unsafe {
        asm!("
			push rbx

		  mov eax, 1
	      cpuid
		  mov {ecx:e}, ecx
		  pop rbx", ecx = out(reg) ecx);
    }

    (ecx & (1 << 31)) != 0
}

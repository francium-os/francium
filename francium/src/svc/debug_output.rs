use crate::aarch64::context::ExceptionContext;

pub fn svc_debug_output(ctx: &mut ExceptionContext) {
	let mut temp_buffer: [u8; 512] = [0; 512];
	unsafe {
		core::ptr::copy_nonoverlapping(ctx.regs[0] as *const u8, temp_buffer.as_mut_ptr(), ctx.regs[1]);
	}

	print!("{}", core::str::from_utf8(&temp_buffer[0..ctx.regs[1]]).unwrap());
}

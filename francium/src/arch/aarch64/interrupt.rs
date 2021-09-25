use super::context::ExceptionContext;
use super::gicv2;
use super::arch_timer;
use crate::scheduler;
use crate::process::get_elr_el1;

extern "C" {
	fn get_esr_el1() -> usize;
	fn get_far_el1() -> usize;
}

type SVCHandler = fn(&mut ExceptionContext);

fn svc_break(_: &mut ExceptionContext) {
	panic!("svcBreak called!");
}

fn svc_debug_output(ctx: &mut ExceptionContext) {
	let mut temp_buffer: [u8; 256] = [0; 256];

	unsafe {
		core::ptr::copy_nonoverlapping(ctx.regs[0] as *const u8, temp_buffer.as_mut_ptr(), ctx.regs[1]);
	}
	println!("[Debug] {}", core::str::from_utf8(&temp_buffer[0..ctx.regs[1]]).unwrap());
}

const SVC_HANDLERS: [SVCHandler; 2] = [
	svc_break,
	svc_debug_output
];

#[no_mangle]
pub extern "C" fn rust_curr_el_spx_sync(_ctx: &ExceptionContext) -> ! {
	unsafe {
		let esr = get_esr_el1();
		let ec = (esr & (0x3f << 26)) >> 26;
		let iss = esr & 0xffffff;

		if ec == 0b100101 {
			println!("Data abort!");
		}

		println!("Exception!!! rust_curr_el_spx_sync!\n");
		println!("lr: {:x}, ec: {:6b}, iss: {:x}", get_elr_el1(), ec, iss);
		println!("FAR: {:x}", get_far_el1());

	    loop {}
	}
}

#[no_mangle]
pub extern "C" fn rust_lower_el_spx_sync(ctx: &mut ExceptionContext) {
	unsafe {
		let esr = get_esr_el1();
		let ec = (esr & (0x3f << 26)) >> 26;
		let iss = esr & 0xffffff;

		// 0b010101 SVC instruction execution in AArch64 state.
		if ec == 0b010101 {
			if iss < SVC_HANDLERS.len() {
				SVC_HANDLERS[iss](ctx);
			}
		} else {
			println!("Exception!!! rust_lower_el_spx_sync!\n");
			println!("lr: {:x}, ec: {:6b}, iss: {:x}", get_elr_el1(), ec, iss);
			println!("FAR: {:x}", get_far_el1());
			
			loop {}
		}
	}
}

#[no_mangle]
pub extern "C" fn rust_lower_el_aarch64_irq(ctx: &mut ExceptionContext) {
	// we know it's an interrupt
	// which one?
	// fuck knows
	// for now, just ack timer

	println!("Tick!");

	arch_timer::reset_timer();

	let timer_irq = 16 + 14;
	gicv2::clear(timer_irq);

	scheduler::tick(ctx);
}

extern "C" {
	fn set_daif(daif: usize);
	fn get_daif() -> usize;
}

pub fn enable_interrupts() {
	unsafe {
		const DAIF_I: usize = 1<<7;
		set_daif(get_daif() & !DAIF_I);
	}
}

pub fn disable_interrupts() {
	unsafe {
		const DAIF_I: usize = 1<<7;
		set_daif(get_daif() | DAIF_I);
	}
}
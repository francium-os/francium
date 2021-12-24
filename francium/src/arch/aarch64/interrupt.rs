use super::context::ExceptionContext;
use super::gicv2;
use super::arch_timer;
use crate::scheduler;
use crate::svc;

extern "C" {
	fn get_esr_el1() -> usize;
	fn get_far_el1() -> usize;
}

type SVCHandler = fn(&mut ExceptionContext);

const SVC_HANDLERS: [SVCHandler; 9] = [
	svc::svc_break,
	svc::svc_debug_output,
	svc::svc_create_port,
	svc::svc_connect_to_port,
	svc::svc_exit_process,
	svc::svc_close_handle,
	svc::svc_ipc_request,
	svc::svc_ipc_reply,
	svc::svc_ipc_receive
];

#[no_mangle]
pub extern "C" fn rust_curr_el_spx_sync(ctx: &ExceptionContext) -> ! {
	unsafe {
		let esr = get_esr_el1();
		let ec = (esr & (0x3f << 26)) >> 26;
		let iss = esr & 0xffffff;

		if ec == 0b100101 {
			println!("Data abort!");
		}

		println!("Exception!!! rust_curr_el_spx_sync!\n");
		println!("lr: {:x}, ec: {:6b}, iss: {:x}", ctx.saved_pc, ec, iss);
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
			} else {
				panic!("Invalid SVC!");
			}
		} else {
			println!("Exception!!! rust_lower_el_spx_sync!\n");
			println!("pc: {:x}, ec: {:6b}, iss: {:x}", ctx.saved_pc, ec, iss);
			println!("FAR: {:x}", get_far_el1());
			
			println!("LR: {:x}", ctx.regs[30]);

			loop {}
		}
	}
}

#[no_mangle]
pub extern "C" fn rust_lower_el_aarch64_irq(_ctx: &mut ExceptionContext) {
	// we know it's an interrupt
	// which one?
	// fuck knows
	// for now, just ack timer

	println!("Tick!");

	arch_timer::reset_timer();

	let timer_irq = 16 + 14;
	gicv2::clear(timer_irq);

	scheduler::tick();
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

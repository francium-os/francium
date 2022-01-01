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

const SVC_HANDLERS: [SVCHandler; 10] = [
	svc::svc_break,
	svc::svc_debug_output,
	svc::svc_create_port,
	svc::svc_connect_to_port,
	svc::svc_exit_process,
	svc::svc_close_handle,
	svc::svc_ipc_request,
	svc::svc_ipc_reply,
	svc::svc_ipc_receive,
	svc::svc_ipc_accept
];

fn stringify_ec(ec: usize) -> &'static str {
	match ec {
		0b000000 => "unknown",
		0b000001 => "trapped wfi/wfe",
		0b000011 => "trapped mcr/mrc",
		0b000100 => "trapped mcrr/mrrc",
		0b000101 => "trapped mcr/mrc",
		0b000110 => "trapped ldc/stc",
		0b000111 => "floating point trap",
		0b001010 => "trapped 64 byte load",
		0b001100 => "trapped mrrc",
		0b001101 => "branch target exception",
		0b001110 => "illegal execution state",
		0b010001 => "aarch32 svc",
		0b010101 => "aarch64 svc",
		0b011000 => "trapped msr/mrs",
		0b011001 => "trapped sve instruction",
		0b011100 => "pac failure",
		0b100000 => "instruction abort from lower level",
		0b100001 => "instruction abort from same level",
		0b100010 => "pc alignment",
		0b100100 => "data abort from lower level",
		0b100101 => "data abort from same level",
		0b100110 => "sp alignment",
		0b101000 => "floating point exception (aarch32)",
		0b101100 => "floating point exception (aarch64)",
		0b101111 => "serror",
		0b110000 => "breakpoint from lower level",
		0b110001 => "breakpoint from same level",
		0b110010 => "software step from lower level",
		0b110011 => "software step from same level",
		0b110100 => "watchpoint from lower level",
		0b110101 => "watchpoint from same level",
		0b111000 => "aarch32 bkpt",
		0b111100 => "aarch64 brk",
		_ => "unknown ?"
	}
}

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
		println!("lr: {:x}, ec: {:}, iss: {:x}", ctx.saved_pc, stringify_ec(ec), iss);
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
			println!("pc: {:x}, ec: {:}, iss: {:x}", ctx.saved_pc, stringify_ec(ec), iss);
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

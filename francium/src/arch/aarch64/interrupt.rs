use super::context::ExceptionContext;
use crate::timer;
use crate::arch::aarch64::svc_wrappers;
use crate::platform::{DEFAULT_TIMER, GIC};
use crate::drivers::Timer;
use crate::drivers::InterruptController;
use core::arch::asm;

unsafe fn get_esr_el1() -> usize {
	let mut value: usize;
	asm!("mrs {esr_el1}, esr_el1", esr_el1 = out(reg) value);
	value
}

unsafe fn get_far_el1() -> usize {
	let mut value: usize;
	asm!("mrs {far_el1}, far_el1", far_el1 = out(reg) value);
	value
}

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

fn stringify_dfsc(dfsc: usize) -> &'static str {
	match dfsc {
		0b000000 => "Address size fault, level 0 of translation or translation table base register.",
		0b000001 => "Address size fault, level 1.",
		0b000010 => "Address size fault, level 2.",
		0b000011 => "Address size fault, level 3.",
		0b000100 => "Translation fault, level 0.",
		0b000101 => "Translation fault, level 1.",
		0b000110 => "Translation fault, level 2.",
		0b000111 => "Translation fault, level 3.",
		0b001001 => "Access flag fault, level 1.",
		0b001010 => "Access flag fault, level 2.",
		0b001011 => "Access flag fault, level 3.",
		0b001000 => "When FEAT_LPA2 is implemented Access flag fault, level 0.",
		0b001100 => "When FEAT_LPA2 is implemented Permission fault, level 0.",
		0b001101 => "Permission fault, level 1.",
		0b001110 => "Permission fault, level 2.",
		0b001111 => "Permission fault, level 3.",
		0b010000 => "Synchronous External abort, not on translation table walk or hardware update of translation table.",
		0b010001 => "When FEAT_MTE2 is implemented Synchronous Tag Check Fault.",
		0b010011 => "When FEAT_LPA2 is implemented Synchronous External abort on translation table walk or hardware update of translation table, level -1.",
		0b010100 => "Synchronous External abort on translation table walk or hardware update of translation table, level 0.",
		0b010101 => "Synchronous External abort on translation table walk or hardware update of translation table, level 1.",
		0b010110 => "Synchronous External abort on translation table walk or hardware update of translation table, level 2.",
		0b010111 => "Synchronous External abort on translation table walk or hardware update of translation table, level 3.",
		0b011000 => "When FEAT_RAS is not implemented Synchronous parity or ECC error on memory access, not on translation table walk.",
		0b011011 => "When FEAT_LPA2 is implemented and FEAT_RAS is not implemented Synchronous parity or ECC error on memory access on translation table walk or hardware update of translation table, level -1.",
		0b011100 => "When FEAT_RAS is not implemented Synchronous parity or ECC error on memory access on translation table walk or hardware update of translation table, level 0.",
		0b011101 => "When FEAT_RAS is not implemented Synchronous parity or ECC error on memory access on translation table walk or hardware update of translation table, level 1.",
		0b011110 => "When FEAT_RAS is not implemented Synchronous parity or ECC error on memory access on translation table walk or hardware update of translation table, level 2.",
		0b011111 => "When FEAT_RAS is not implemented Synchronous parity or ECC error on memory access on translation table walk or hardware update of translation table, level 3.",
		0b100001 => "Alignment fault.",
		0b101001 => "When FEAT_LPA2 is implemented Address size fault, level -1.",
		0b101011 => "When FEAT_LPA2 is implemented Translation fault, level -1.",
		0b110000 => "TLB conflict abort.",
		0b110001 => "When FEAT_HAFDBS is implemented Unsupported atomic hardware update fault.",
		0b110100 => "IMPLEMENTATION DEFINED fault (Lockdown).",
		0b110101 => "IMPLEMENTATION DEFINED fault (Unsupported Exclusive or Atomic access).",
		_ => "unknown ?"
	}
}

fn stringify_ifsc(ifsc: usize) -> &'static str {
	match ifsc {
		0b000000 => "Address size fault, level 0 of translation or translation table base register.",
		0b000001 => "Address size fault, level 1.",
		0b000010 => "Address size fault, level 2.",
		0b000011 => "Address size fault, level 3.",
		0b000100 => "Translation fault, level 0.",
		0b000101 => "Translation fault, level 1.",
		0b000110 => "Translation fault, level 2.",
		0b000111 => "Translation fault, level 3.",
		0b001001 => "Access flag fault, level 1.",
		0b001010 => "Access flag fault, level 2.",
		0b001011 => "Access flag fault, level 3.",
		0b001000 => "When FEAT_LPA2 is implemented Access flag fault, level 0.",
		0b001100 => "When FEAT_LPA2 is implemented Permission fault, level 0.",
		0b001101 => "Permission fault, level 1.",
		0b001110 => "Permission fault, level 2.",
		0b001111 => "Permission fault, level 3.",
		0b010000 => "Synchronous External abort, not on translation table walk or hardware update of translation table.",
		0b010011 => "When FEAT_LPA2 is implemented Synchronous External abort on translation table walk or hardware update of translation table, level -1.",
		0b010100 => "Synchronous External abort on translation table walk or hardware update of translation table, level 0.",
		0b010101 => "Synchronous External abort on translation table walk or hardware update of translation table, level 1.",
		0b010110 => "Synchronous External abort on translation table walk or hardware update of translation table, level 2.",
		0b010111 => "Synchronous External abort on translation table walk or hardware update of translation table, level 3.",
		0b011000 => "When FEAT_RAS is not implemented Synchronous parity or ECC error on memory access, not on translation table walk.",
		0b011011 => "When FEAT_LPA2 is implemented and FEAT_RAS is not implemented Synchronous parity or ECC error on memory access on translation table walk or hardware update of translation table, level -1.",
		0b011100 => "When FEAT_RAS is not implemented Synchronous parity or ECC error on memory access on translation table walk or hardware update of translation table, level 0.",
		0b011101 => "When FEAT_RAS is not implemented Synchronous parity or ECC error on memory access on translation table walk or hardware update of translation table, level 1.",
		0b011110 => "When FEAT_RAS is not implemented Synchronous parity or ECC error on memory access on translation table walk or hardware update of translation table, level 2.",
		0b011111 => "When FEAT_RAS is not implemented Synchronous parity or ECC error on memory access on translation table walk or hardware update of translation table, level 3.",
		0b101001 => "When FEAT_LPA2 is implemented Address size fault, level -1.",
		0b101011 => "When FEAT_LPA2 is implemented Translation fault, level -1.",
		0b110000 => "TLB conflict abort.",
		0b110001 => "When FEAT_HAFDBS is implemented Unsupported atomic hardware update fault",
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
		println!("lr: {:x}, ec: {:} ({}), iss: {:x}", ctx.saved_pc, stringify_ec(ec),ec, iss);
		println!("FAR: {:x}", get_far_el1());

		// better handling of data abort
		if ec == 0b100101 {
			let dfsc = iss & 0x3f;
			println!("data fault status: {}", stringify_dfsc(dfsc));
		}

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
			if iss < svc_wrappers::SVC_HANDLERS.len() {
				svc_wrappers::SVC_HANDLERS[iss](ctx);
			} else {
				panic!("Invalid SVC!");
			}
		} else {
			println!("Exception!!! rust_lower_el_spx_sync!\n");
			println!("pc: {:x}, ec: {:} ({}), iss: {:x}", ctx.saved_pc, stringify_ec(ec), ec, iss);
			println!("FAR: {:x}", get_far_el1());
			
			println!("LR: {:x}", ctx.regs[30]);

			// better handling of data abort
			if ec == 0b100100 {
				let dfsc = iss & 0x3f;
				println!("data fault status: {}", stringify_dfsc(dfsc));
			} else if ec == 0b100000 { // instruction abort
				let ifsc = iss & 0x3f;
				println!("instruction fault status: {}", stringify_ifsc(ifsc));
			}

			loop {}
		}
	}
}

#[no_mangle]
pub extern "C" fn rust_lower_el_aarch64_irq(_ctx: &mut ExceptionContext) {
	// we know it's an interrupt
	// which one?
	// for now, just ack timer

	{
		let timer_lock = DEFAULT_TIMER.lock();
		timer_lock.reset_timer();
	}

	let timer_irq = 16 + 14;
	{ GIC.lock().ack_interrupt(timer_irq); }

	timer::tick();
}


unsafe fn get_daif() -> usize {
	let mut value: usize;
	asm!("mrs {daif}, daif", daif = out(reg) value);
	value
}

unsafe fn set_daif(value: usize) {
	asm!("msr daif, {daif}", daif = in(reg) value);
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

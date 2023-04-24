use super::context::ExceptionContext;
use crate::arch::aarch64::svc_wrappers;
use crate::drivers::Timer;
use crate::drivers::{InterruptController, InterruptDistributor};
use crate::platform::{DEFAULT_TIMER, INTERRUPT_CONTROLLER, INTERRUPT_DISTRIBUTOR};
use crate::timer;

use aarch64_cpu::registers::*;
use tock_registers::interfaces::Readable;

fn stringify_ec(ec: u64) -> &'static str {
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
        _ => "unknown ?",
    }
}

fn stringify_dfsc(dfsc: u64) -> &'static str {
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

fn stringify_ifsc(ifsc: u64) -> &'static str {
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
    let esr = ESR_EL1.get();
    let ec = (esr & (0x3f << 26)) >> 26;
    let iss = esr & 0xffffff;

    if ec == 0b100101 {
        println!("Data abort!");
    }

    println!("Exception!!! rust_curr_el_spx_sync!\n");
    println!(
        "lr: {:x}, ec: {:} ({}), iss: {:x}",
        ctx.saved_pc,
        stringify_ec(ec),
        ec,
        iss
    );
    println!("FAR: {:x}", FAR_EL1.get());

    // better handling of data abort
    if ec == 0b100101 {
        let dfsc = iss & 0x3f;
        println!("data fault status: {}", stringify_dfsc(dfsc));
    }

    loop {}
}

#[no_mangle]
pub extern "C" fn rust_lower_el_spx_sync(ctx: &mut ExceptionContext) {
    let esr = ESR_EL1.get();
    let ec = (esr & (0x3f << 26)) >> 26;
    let iss = esr & 0xffffff;

    // 0b010101 SVC instruction execution in AArch64 state.
    if ec == 0b010101 {
        if (iss as usize) < svc_wrappers::SVC_HANDLERS.len() {
            svc_wrappers::SVC_HANDLERS[iss as usize](ctx);
        } else {
            panic!("Invalid SVC!");
        }
    } else {
        println!("Exception!!! rust_lower_el_spx_sync!\n");
        println!(
            "pc: {:x}, ec: {:} ({}), iss: {:x}",
            ctx.saved_pc,
            stringify_ec(ec),
            ec,
            iss
        );
        println!("FAR: {:x}", FAR_EL1.get());

        println!("LR: {:x}", ctx.regs[30]);

        // better handling of data abort
        if ec == 0b100100 {
            let dfsc = iss & 0x3f;
            println!("data fault status: {}", stringify_dfsc(dfsc));

            let current_process = crate::scheduler::get_current_process();
            let proc_locked = current_process.lock();
            println!(
                "?? {:?}",
                proc_locked
                    .address_space
                    .page_table
                    .virt_to_phys(FAR_EL1.get() as usize)
            );
        } else if ec == 0b100000 {
            // instruction abort
            let ifsc = iss & 0x3f;
            println!("instruction fault status: {}", stringify_ifsc(ifsc));
        }

        loop {}
    }
}

#[no_mangle]
pub extern "C" fn rust_lower_el_aarch64_irq(_ctx: &mut ExceptionContext) {
    let next = INTERRUPT_CONTROLLER.lock().next_pending();
    if let Some(interrupt) = next {
        // handle!
        match interrupt {
            // TODO: Arch specific might have different way of identfying interrupts.
            1 => {
                // Pi3 timer
                let mut timer_lock = DEFAULT_TIMER.lock();
                timer_lock.tick();
                timer_lock.reset_timer();
                INTERRUPT_CONTROLLER.lock().ack_interrupt(interrupt);
            }
            30 => {
                let mut timer_lock = DEFAULT_TIMER.lock();
                timer_lock.tick();
                timer_lock.reset_timer();
                INTERRUPT_CONTROLLER.lock().ack_interrupt(interrupt);
            }
            _ => {
                INTERRUPT_DISTRIBUTOR.lock().disable_interrupt(interrupt);
                if !crate::svc::event::dispatch_interrupt_event(interrupt as usize) {
                    // An interrupt event will ack the interrupt when it's done.
                    INTERRUPT_CONTROLLER.lock().ack_interrupt(interrupt);
                }
            }
        }
    }

    timer::tick();
}

pub fn enable_interrupts() {
    unimplemented!();
}

pub fn disable_interrupts() {
    unimplemented!();
}

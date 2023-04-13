use crate::arch::context::ExceptionContext;
use crate::drivers::InterruptController;
use crate::drivers::Timer;
use crate::platform::INTERRUPT_CONTROLLER;
use crate::platform::DEFAULT_TIMER;
use core::arch::{asm, global_asm};

macro_rules! interrupt_noerror {
    ($interrupt_name:ident, $interrupt_number:expr) => {
        #[naked]
        #[no_mangle]
        unsafe extern "C" fn $interrupt_name() {
            asm!(
                concat!(
                    "push 0\n",
                    "push ",
                    stringify!($interrupt_number),
                    "\n",
                    "jmp exception_error\n"
                ),
                options(noreturn)
            );
        }
    };
}

macro_rules! interrupt_error {
    ($interrupt_name:ident, $interrupt_number:expr) => {
        #[naked]
        #[no_mangle]
        unsafe extern "C" fn $interrupt_name() {
            asm!(
                concat!(
                    "push ",
                    stringify!($interrupt_number),
                    "\n",
                    "jmp exception_error"
                ),
                options(noreturn)
            );
        }
    };
}

macro_rules! irq_handler {
    ($interrupt_name:ident, $interrupt_number:expr) => {
        #[naked]
        #[no_mangle]
        unsafe extern "C" fn $interrupt_name() {
            asm!(
                concat!(
                    "push 0\n",
                    "push ",
                    stringify!($interrupt_number),
                    "\n",
                    "jmp exception_error"
                ),
                options(noreturn)
            );
        }
    };
}

global_asm!(
    "
.global exception_error
.global exception_no_error
.global restore_exception_context
.extern handle_exception

// stack layout:
// ss (+48)
// rsp (+40)
// rflags (+32)
// cs  (+24)
// rip (+16)
// error code (+8)
// interrupt number (+0)

exception_error:
// error code
// interrupt number
push r15
push r14
push r13
push r12
push r11
push r10
push r9
push r8
push rdi
push rsi
push rbp
push rdx
push rcx
push rbx
push rax

// Reach back into the stack to grab the error code...
mov rdi, rsp
mov rsi, [rsp + 16*8]
mov rdx, [rsp + 15*8]
call handle_exception

// falls through
restore_exception_context:
pop rax
pop rbx
pop rcx
pop rdx
pop rbp
pop rsi
pop rdi
pop r8
pop r9
pop r10
pop r11
pop r12
pop r13
pop r14
pop r15

// Drop error code + interrupt number off stack
add rsp, 16
iretq"
);

interrupt_noerror!(interrupt_0, 0);
interrupt_noerror!(interrupt_1, 1);
interrupt_noerror!(interrupt_2, 2);
interrupt_noerror!(interrupt_3, 3);
interrupt_noerror!(interrupt_4, 4);
interrupt_noerror!(interrupt_5, 5);
interrupt_noerror!(interrupt_6, 6);
interrupt_noerror!(interrupt_7, 7);
interrupt_error!(interrupt_8, 8);
interrupt_noerror!(interrupt_9, 9);
interrupt_error!(interrupt_10, 10);
interrupt_error!(interrupt_11, 11);
interrupt_error!(interrupt_12, 12);
interrupt_error!(interrupt_13, 13);
interrupt_error!(interrupt_14, 14);
interrupt_noerror!(interrupt_15, 15);
interrupt_noerror!(interrupt_16, 16);
interrupt_error!(interrupt_17, 17);
interrupt_noerror!(interrupt_18, 18);
interrupt_noerror!(interrupt_19, 19);
interrupt_noerror!(interrupt_20, 20);
interrupt_noerror!(interrupt_21, 21);
interrupt_noerror!(interrupt_22, 22);
interrupt_noerror!(interrupt_23, 23);
interrupt_noerror!(interrupt_24, 24);
interrupt_noerror!(interrupt_25, 25);
interrupt_noerror!(interrupt_26, 26);
interrupt_noerror!(interrupt_27, 27);
interrupt_noerror!(interrupt_28, 28);
interrupt_noerror!(interrupt_29, 29);
interrupt_error!(interrupt_30, 30);
interrupt_noerror!(interrupt_31, 31);

interrupt_noerror!(interrupt_128, 128);

irq_handler!(irq_0, 32);
irq_handler!(irq_1, 33);
irq_handler!(irq_2, 34);
irq_handler!(irq_3, 35);
irq_handler!(irq_4, 36);
irq_handler!(irq_5, 37);
irq_handler!(irq_6, 38);
irq_handler!(irq_7, 39);
irq_handler!(irq_8, 40);
irq_handler!(irq_9, 41);
irq_handler!(irq_10, 42);
irq_handler!(irq_11, 43);
irq_handler!(irq_12, 44);
irq_handler!(irq_13, 45);
irq_handler!(irq_14, 46);
irq_handler!(irq_15, 47);

interrupt_noerror!(unknown_interrupt, 255);

pub const INTERRUPT_HANDLERS: [unsafe extern "C" fn(); 48] = [
    interrupt_0,
    interrupt_1,
    interrupt_2,
    interrupt_3,
    interrupt_4,
    interrupt_5,
    interrupt_6,
    interrupt_7,
    interrupt_8,
    interrupt_9,
    interrupt_10,
    interrupt_11,
    interrupt_12,
    interrupt_13,
    interrupt_14,
    interrupt_15,
    interrupt_16,
    interrupt_17,
    interrupt_18,
    interrupt_19,
    interrupt_20,
    interrupt_21,
    interrupt_22,
    interrupt_23,
    interrupt_24,
    interrupt_25,
    interrupt_26,
    interrupt_27,
    interrupt_28,
    interrupt_29,
    interrupt_30,
    interrupt_31,
    irq_0,
    irq_1,
    irq_2,
    irq_3,
    irq_4,
    irq_5,
    irq_6,
    irq_7,
    irq_8,
    irq_9,
    irq_10,
    irq_11,
    irq_12,
    irq_13,
    irq_14,
    irq_15,
];

pub fn read_cr2() -> usize {
    unsafe {
        let cr2: usize;
        asm!("mov {cr2}, cr2", cr2 = out(reg)(cr2));
        cr2
    }
}

#[no_mangle]
unsafe extern "C" fn handle_exception(
    ctx: &ExceptionContext,
    error_code: u64,
    interrupt_number: u64,
) {
    //println!("Current process: {}", crate::scheduler::get_current_process().lock().name);
    match interrupt_number {
        0x6 => {
            println!("Invalid instruction!");
            panic!("No");
        }

        0xe => {
            let cr2 = read_cr2();
            println!("Page fault at {:x}!", cr2);
            if (error_code & (1 << 0)) == (1 << 0) {
                print!("protection violation");
            } else {
                print!("not present")
            }

            if (error_code & (1 << 1)) == (1 << 1) {
                print!(", write");
            } else {
                print!(", read");
            }

            if (error_code & (1 << 2)) == (1 << 2) {
                print!(", user");
            } else {
                print!(", supervisor");
            }

            if (error_code & (1 << 4)) == (1 << 4) {
                print!(" instruction fetch");
            } else {
                print!(" data fetch");
            }

            if (error_code & (1 << 3)) == (1 << 3) {
                print!(" (reserved bit violation)");
            }

            println!("");

            let process = &crate::scheduler::get_current_process();
            let process_locked = process.lock();
            let _pg = &process_locked.address_space.page_table;
            //println!("Walk: {:x}", pg.virt_to_phys(cr2).unwrap().0);

            println!("stack dump");
            for i in -32..32 {
                println!("{:?}: {:x}", i, *(ctx.regs.rsp as *const usize).offset(i));
            }

            /*if error_code & (1<<5) {
                // protection key
            }
            if error_code & (1<<6) {
                // shadow stack
            }
            if error_code & (1<<7) {
                // hlat
            }
            if error_code & (1<<15) {
                // sgx
            }*/

            panic!("Can't handle page fault!");
        }
        32..=39 => {
            // IRQ0-7
            let irq_number = interrupt_number - 32;

            if irq_number == 7 {
                println!("Spurious IRQ?");
                // todo spurious irq handling
            } else if irq_number == 0 {
                // handle Timer specially
                {
                    INTERRUPT_CONTROLLER.lock().ack_interrupt(0);
                }

                {
                    let mut timer_lock = DEFAULT_TIMER.lock();
                    timer_lock.tick();
                }

                crate::timer::tick();
            } else {
                // pog
                {
                    crate::svc::event::dispatch_interrupt_event(irq_number as usize);
                    INTERRUPT_CONTROLLER.lock().ack_interrupt(irq_number as u32);
                }
            }
        }
        40..=47 => {
            // IRQ8-15
            let irq_number = interrupt_number - 32;

            if irq_number == 15 {
                // todo spurious irq handling
            } else {
                // pog
                {
                    crate::svc::event::dispatch_interrupt_event(irq_number as usize);
                    INTERRUPT_CONTROLLER.lock().ack_interrupt(irq_number as u32);
                }
            }
        }
        _ => {
            println!(
                "Current process: {}",
                crate::scheduler::get_current_process().lock().name
            );
            println!("error_code: {}", error_code);
            println!("register dump:\n{:?}", ctx.regs);
            panic!("Unhandled interrupt {:?}", interrupt_number);
        }
    }
}

use core::arch::{asm, global_asm};

macro_rules! interrupt_noerror {
	($interrupt_name:ident, $interrupt_number:expr) => {
		#[naked]
		#[no_mangle]
		unsafe extern "C" fn $interrupt_name() {
			asm!(concat!("push 0\n",
				  "push ", stringify!($interrupt_number),"\n",
				  "jmp exception_error\n"), options(noreturn));
		}
	}
}

macro_rules! interrupt_error {
	($interrupt_name:ident, $interrupt_number:expr) => {
		#[naked]
		#[no_mangle]
		unsafe extern "C" fn $interrupt_name() {
			asm!(concat!("push ", stringify!($interrupt_number),"\n",
				  "jmp exception_error"), options(noreturn));
		}
	}
}

macro_rules! irq_handler {
	($interrupt_name:ident, $interrupt_number:expr) => {
		#[naked]
		#[no_mangle]
		unsafe extern "C" fn $interrupt_name() {
			asm!(concat!("push 0\n", "push ", stringify!($interrupt_number),"\n",
				  "jmp exception_error"), options(noreturn));
		}
	}
}

global_asm!("
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
// interrupt number (+8)
// error code (+0)

exception_error:
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
mov rsi, rsp
mov rdi, [rsp - 16*8]
mov rdx, [rsp - 17*8]
call handle_exception
xchg bx, bx

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
xchg bx, bx
iretq");

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
	irq_15
];

#[no_mangle]
unsafe extern "C" fn handle_exception() {
	println!("exception");
	unimplemented!();
}
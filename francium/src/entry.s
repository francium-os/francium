.global _start
.global exception_handler
.extern rust_main

.section .text.entry
_start:
	ldr x0, =_bootstrap_stack_top
	mov sp, x0
	b rust_main

.section .text.exceptions
exception_handler:
	ldr x0, =0xcafebabe
	b .

.section .data.bootstrap_stack
_bootstrap_stack_bottom:
.space 0x8000
_bootstrap_stack_top:

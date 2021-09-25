.global kernel_start
.global set_ttbr0_el1

.extern rust_main
.extern __bootstrap_stack_bottom
.extern __bootstrap_stack_top

.section .text
kernel_start:
     // Setup stack
	ldr x0, =__bootstrap_stack_top
	mov sp, x0

     // Setup vbar
	ldr x0, =__vbar
	msr vbar_el1, x0

     // Zero out bss!
     mov x2, 0
     ldr x1, =__bss_end
     ldr x0, =__bss_start
     .bss_clear:
     str x2, [x0]
     add x0, x0, #8
     cmp x0, x1
     bne .bss_clear

	b rust_main

.section .bss.bootstrap_stack
__bootstrap_stack_guard:
.space 0x10
__bootstrap_stack_bottom:
.space 0x80000
__bootstrap_stack_top:

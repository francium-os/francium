.global switch_stacks
.global __bootstrap_stack_bottom
.global __bootstrap_stack_top
.global interrupt_stack_top

.section .text
switch_stacks:
    // Setup stack
    pop rax
	movabs rsp, offset __bootstrap_stack_top
	push rax
	ret

.section .bss.bootstrap_stack
// Stack must be 0x10 aligned!
.align 4
__bootstrap_stack_guard:
.space 0x10
__bootstrap_stack_bottom:
.space 0x40000
__bootstrap_stack_top:

interrupt_stack_guard:
.space 0x10
interrupt_stack_bottom:
.space 0x1000
interrupt_stack_top:
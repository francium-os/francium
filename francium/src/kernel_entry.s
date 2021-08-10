.global kernel_start
.global set_ttbr0_el1

.extern rust_main
.extern rust_curr_el_spx_sync
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

.section .text.exceptions

// https://developer.arm.com/documentation/100933/0100/AArch64-exception-vector-table

// Typical exception vector table code.
.balign 0x800
vector_table_el1:
curr_el_sp0_sync:        // The exception handler for a synchronous 
                         // exception from the current EL using SP0.
ldr x0, =0xcafebabe
b .

.balign 0x80
curr_el_sp0_irq:         // The exception handler for an IRQ exception
                         // from the current EL using SP0.
b .
.balign 0x80
curr_el_sp0_fiq:         // The exception handler for an FIQ exception
                         // from the current EL using SP0.
b .

.balign 0x80
curr_el_sp0_serror:      // The exception handler for a System Error 
                         // exception from the current EL using SP0.
b .

.balign 0x80
curr_el_spx_sync:        // The exception handler for a synchrous 
                         // exception from the current EL using the
                         // current SP.
sub sp, sp,    #0x110
stp x0, x1,    [sp, #0x00]
stp x2, x3,    [sp, #0x10]
stp x4, x3,    [sp, #0x20]
stp x6, x3,    [sp, #0x30]
stp x8, x3,    [sp, #0x40]
stp x10, x3,   [sp, #0x50]
stp x12, x3,   [sp, #0x60]
stp x14, x3,   [sp, #0x70]
stp x16, x3,   [sp, #0x80]
stp x18, x3,   [sp, #0x90]
stp x20, x3,   [sp, #0xa0]
stp x22, x3,   [sp, #0xb0]
stp x24, x3,   [sp, #0xc0]
stp x26, x3,   [sp, #0xd0]
stp x28, x29,  [sp, #0xe0]
str x30,       [sp, #0xf0]
mov x0, sp
bl rust_curr_el_spx_sync
b restore_exception_context

.balign 0x80
curr_el_spx_irq:         // The exception handler for an IRQ exception from 
                         // the current EL using the current SP.
b .

.balign 0x80
curr_el_spx_fiq:         // The exception handler for an FIQ from 
                         // the current EL using the current SP.
b .

.balign 0x80
curr_el_spx_serror:      // The exception handler for a System Error 
                         // exception from the current EL using the
                         // current SP.
b .

 .balign 0x80
lower_el_aarch64_sync:   // The exception handler for a synchronous 
                         // exception from a lower EL (AArch64).
sub sp, sp,    #0x110
stp x0, x1,    [sp, #0x00]
stp x2, x3,    [sp, #0x10]
stp x4, x3,    [sp, #0x20]
stp x6, x3,    [sp, #0x30]
stp x8, x3,    [sp, #0x40]
stp x10, x3,   [sp, #0x50]
stp x12, x3,   [sp, #0x60]
stp x14, x3,   [sp, #0x70]
stp x16, x3,   [sp, #0x80]
stp x18, x3,   [sp, #0x90]
stp x20, x3,   [sp, #0xa0]
stp x22, x3,   [sp, #0xb0]
stp x24, x3,   [sp, #0xc0]
stp x26, x3,   [sp, #0xd0]
stp x28, x29,  [sp, #0xe0]
str x30,       [sp, #0xf0]
mrs x0, elr_el1
str x0,        [sp, #0x100]
mrs x0, esr_el1
str x0,        [sp, #0x108]

mov x0, sp
bl rust_lower_el_spx_sync
b restore_exception_context
b .

.balign 0x80
lower_el_aarch64_irq:    // The exception handler for an IRQ from a lower EL
                         // (AArch64).
b .

.balign 0x80
lower_el_aarch64_fiq:    // The exception handler for an FIQ from a lower EL
                         // (AArch64).
b .

.balign 0x80
lower_el_aarch64_serror: // The exception handler for a System Error 
                         // exception from a lower EL(AArch64).
b .

.balign 0x80
lower_el_aarch32_sync:   // The exception handler for a synchronous 
                         // exception from a lower EL(AArch32).
b .

.balign 0x80
lower_el_aarch32_irq:    // The exception handler for an IRQ exception 
                         // from a lower EL (AArch32).
b .

.balign 0x80
lower_el_aarch32_fiq:    // The exception handler for an FIQ exception from 
                         // a lower EL (AArch32).
b .

.balign 0x80
lower_el_aarch32_serror: // The exception handler for a System Error
                         // exception from a lower EL(AArch32).
b .


restore_exception_context:
ldp x0, x1,    [sp, #0x00]
ldp x2, x3,    [sp, #0x10]
ldp x4, x3,    [sp, #0x20]
ldp x6, x3,    [sp, #0x30]
ldp x8, x3,    [sp, #0x40]
ldp x10, x3,   [sp, #0x50]
ldp x12, x3,   [sp, #0x60]
ldp x14, x3,   [sp, #0x70]
ldp x16, x3,   [sp, #0x80]
ldp x18, x3,   [sp, #0x90]
ldp x20, x3,   [sp, #0xa0]
stp x22, x3,   [sp, #0xb0]
ldp x24, x3,   [sp, #0xc0]
ldp x26, x3,   [sp, #0xd0]
ldp x28, x29,  [sp, #0xe0]
ldr x30,       [sp, #0xf0]
add sp, sp, #0x100
eret

.section .bss.bootstrap_stack
__bootstrap_stack_guard:
.space 0x10
__bootstrap_stack_bottom:
.space 0x40000
__bootstrap_stack_top:

.global get_daif
.global set_daif
.global get_far_el1
.global get_esr_el1

.global restore_exception_context

.extern rust_curr_el_spx_sync
.extern rust_lower_el_spx_sync
.extern rust_lower_el_aarch64_irq

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
sub sp, sp,    #0x120
stp x0, x1,    [sp, #0x00]
stp x2, x3,    [sp, #0x10]
stp x4, x5,    [sp, #0x20]
stp x6, x7,    [sp, #0x30]
stp x8, x9,    [sp, #0x40]
stp x10, x11,  [sp, #0x50]
stp x12, x13,  [sp, #0x60]
stp x14, x15,  [sp, #0x70]
stp x16, x17,  [sp, #0x80]
stp x18, x19,  [sp, #0x90]
stp x20, x21,  [sp, #0xa0]
stp x22, x23,  [sp, #0xb0]
stp x24, x25,  [sp, #0xc0]
stp x26, x27,  [sp, #0xd0]
stp x28, x29,  [sp, #0xe0]

mrs x0, sp_el0
stp x30, x0,   [sp, #0xf0]

mrs x0, elr_el1
mrs x1, spsr_el1
stp x0, x1,    [sp, #0x100]

mrs x0, tpidr_el0
str x0,        [sp, #0x110]

mov x0, sp
bl rust_curr_el_spx_sync
b restore_exception_context

.balign 0x80
curr_el_spx_irq:         // The exception handler for an IRQ exception from 
                         // the current EL using the current SP.
sub sp, sp,    #0x120
stp x0, x1,    [sp, #0x00]
stp x2, x3,    [sp, #0x10]
stp x4, x5,    [sp, #0x20]
stp x6, x7,    [sp, #0x30]
stp x8, x9,    [sp, #0x40]
stp x10, x11,  [sp, #0x50]
stp x12, x13,  [sp, #0x60]
stp x14, x15,  [sp, #0x70]
stp x16, x17,  [sp, #0x80]
stp x18, x19,  [sp, #0x90]
stp x20, x21,  [sp, #0xa0]
stp x22, x23,  [sp, #0xb0]
stp x24, x25,  [sp, #0xc0]
stp x26, x27,  [sp, #0xd0]
stp x28, x29,  [sp, #0xe0]

mrs x0, sp_el0
stp x30, x0,   [sp, #0xf0]

mrs x0, elr_el1
mrs x1, spsr_el1
stp x0, x1,    [sp, #0x100]

mrs x0, tpidr_el0
str x0,        [sp, #0x110]

mov x0, sp
bl rust_lower_el_aarch64_irq
b restore_exception_context
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
sub sp, sp,    #0x120
stp x0, x1,    [sp, #0x00]
stp x2, x3,    [sp, #0x10]
stp x4, x5,    [sp, #0x20]
stp x6, x7,    [sp, #0x30]
stp x8, x9,    [sp, #0x40]
stp x10, x11,  [sp, #0x50]
stp x12, x13,  [sp, #0x60]
stp x14, x15,  [sp, #0x70]
stp x16, x17,  [sp, #0x80]
stp x18, x19,  [sp, #0x90]
stp x20, x21,  [sp, #0xa0]
stp x22, x23,  [sp, #0xb0]
stp x24, x25,  [sp, #0xc0]
stp x26, x27,  [sp, #0xd0]
stp x28, x29,  [sp, #0xe0]

mrs x0, sp_el0
stp x30, x0,   [sp, #0xf0]

mrs x0, elr_el1
mrs x1, spsr_el1
stp x0, x1,    [sp, #0x100]

mrs x0, tpidr_el0
str x0,        [sp, #0x110]

mov x0, sp
bl rust_lower_el_spx_sync
b restore_exception_context
b .

.balign 0x80
lower_el_aarch64_irq:    // The exception handler for an IRQ from a lower EL
                         // (AArch64).
sub sp, sp,    #0x120
stp x0, x1,    [sp, #0x00]
stp x2, x3,    [sp, #0x10]
stp x4, x5,    [sp, #0x20]
stp x6, x7,    [sp, #0x30]
stp x8, x9,    [sp, #0x40]
stp x10, x11,  [sp, #0x50]
stp x12, x13,  [sp, #0x60]
stp x14, x15,  [sp, #0x70]
stp x16, x17,  [sp, #0x80]
stp x18, x19,  [sp, #0x90]
stp x20, x21,  [sp, #0xa0]
stp x22, x23,  [sp, #0xb0]
stp x24, x25,  [sp, #0xc0]
stp x26, x27,  [sp, #0xd0]
stp x28, x29,  [sp, #0xe0]

mrs x0, sp_el0
stp x30, x0,   [sp, #0xf0]

mrs x0, elr_el1
mrs x1, spsr_el1
// todo: tpidr?
stp x0, x1,    [sp, #0x100]

mrs x0, tpidr_el0
str x0,        [sp, #0x110]

mov x0, sp
bl rust_lower_el_aarch64_irq
b restore_exception_context
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
ldp x30, x0,   [sp, #0xf0]
msr sp_el0, x0

ldp x0, x1,    [sp, #0x100]
msr elr_el1, x0
msr spsr_el1, x1

ldr x0,        [sp, #0x110]
msr tpidr_el0, x0

ldp x0, x1,    [sp, #0x00]
ldp x2, x3,    [sp, #0x10]
ldp x4, x5,    [sp, #0x20]
ldp x6, x7,    [sp, #0x30]
ldp x8, x9,    [sp, #0x40]
ldp x10, x11,  [sp, #0x50]
ldp x12, x13,  [sp, #0x60]
ldp x14, x15,  [sp, #0x70]
ldp x16, x17,  [sp, #0x80]
ldp x18, x19,  [sp, #0x90]
ldp x20, x21,  [sp, #0xa0]
ldp x22, x23,  [sp, #0xb0]
ldp x24, x25,  [sp, #0xc0]
ldp x26, x27,  [sp, #0xd0]
ldp x28, x29,  [sp, #0xe0]
add sp, sp, #0x120
eret

.section .text
get_daif:
mrs x0, daif
ret

set_daif:
msr daif, x0
ret

get_far_el1:
mrs x0, far_el1
ret

get_esr_el1:
mrs x0, esr_el1
ret

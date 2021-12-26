.global restore_thread_context

.global user_thread_starter

.section .text
get_esr_el1:
mrs x0, esr_el1
ret

restore_thread_context:
// context in x0, x1 is the mutex

// load lr, sp
ldp x30, x2,   [x0, #0xf0]
mov sp, x2

// We need to save LR around this call...
str   lr, [sp, #-16]!

// Unlock the mutex for the thread context.
mov x0, x1
bl force_unlock_mutex

// Restore LR.
ldr   lr, [sp], #16

// ok, now zero everything
mov x0, xzr
mov x1, xzr
mov x2, xzr
mov x3, xzr
mov x4, xzr
mov x5, xzr
mov x6, xzr
mov x7, xzr
mov x8, xzr
mov x9, xzr
mov x10, xzr
mov x11, xzr
mov x12, xzr
mov x13, xzr
mov x14, xzr
mov x15, xzr
mov x16, xzr
mov x17, xzr
mov x18, xzr
mov x19, xzr
mov x20, xzr
mov x21, xzr
mov x22, xzr
mov x23, xzr
mov x24, xzr
mov x25, xzr
mov x26, xzr
mov x27, xzr
mov x28, xzr
mov x29, xzr

// we loaded LR, now ret!
ret

// As defined in interrupt.s
user_thread_starter:
b restore_exception_context
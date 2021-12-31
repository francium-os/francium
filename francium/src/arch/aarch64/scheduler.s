.global switch_thread_asm
.extern force_unlock_mutex

// x0: from context
// x1: to context
// x2: from mutex
// x3: to mutex

switch_thread_asm:

// Save callee save registers.
stp x19, x20,  [x0, #0x98]
stp x21, x22,   [x0, #0xa8]
stp x23, x24,  [x0, #0xb8]
stp x25, x26,  [x0, #0xc8]
stp x27, x28,  [x0, #0xd8]
stp x29, x30,  [x0, #0xe8]

mov x4, sp
// Save SP.
str x4, [x0, #0xf8]

ldp x19, x20,  [x1, #0x98]
ldp x21, x22,  [x1, #0xa8]
ldp x23, x24,  [x1, #0xb8]
ldp x25, x26,  [x1, #0xc8]
ldp x27, x28,  [x1, #0xd8]
ldp x29, x30,  [x1, #0xe8]
// Restore SP.
ldr x0, [x1, #0xf8]
mov sp, x0

// We need to save LR/x1 around these calls...
stp lr, x0, [sp, #-16]!

// Unlock the mutex for the from context.
mov x0, x2
bl force_unlock_mutex

// Unlock the mutex for the to context.
mov x0, x3
bl force_unlock_mutex

// Restore LR.
ldp  lr, x0, [sp], #16

// load tag
ldr x0, [x1, #0x00]

ret
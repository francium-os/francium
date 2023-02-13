.section .text
.global switch_thread_asm
.extern force_unlock_mutex

// x0: from context
// x1: to context
// x2: from mutex
// x3: to mutex

switch_thread_asm:

// Save callee save registers.
stp x19, x20,  [x0, #0x98]
stp x21, x22,  [x0, #0xa8]
stp x23, x24,  [x0, #0xb8]
stp x25, x26,  [x0, #0xc8]
stp x27, x28,  [x0, #0xd8]
stp x29, x30,  [x0, #0xe8]

/* Wow, SIMD state is expensive. */
str q0, [x0, #0x100]
str q1, [x0, #0x110]
str q2, [x0, #0x120]
str q3, [x0, #0x130]
str q4, [x0, #0x140]
str q5, [x0, #0x150]
str q6, [x0, #0x160]
str q7, [x0, #0x170]
str q8, [x0, #0x180]
str q9, [x0, #0x190]
str q10, [x0, #0x1a0]
str q11, [x0, #0x1b0]
str q12, [x0, #0x1c0]
str q13, [x0, #0x1d0]
str q14, [x0, #0x1e0]
str q15, [x0, #0x1f0]
str q16, [x0, #0x200]
str q17, [x0, #0x210]
str q18, [x0, #0x220]
str q19, [x0, #0x230]
str q20, [x0, #0x240]
str q21, [x0, #0x250]
str q22, [x0, #0x260]
str q23, [x0, #0x270]
str q24, [x0, #0x280]
str q25, [x0, #0x290]
str q26, [x0, #0x2a0]
str q27, [x0, #0x2b0]
str q28, [x0, #0x2c0]
str q29, [x0, #0x2d0]
str q30, [x0, #0x2e0]
str q31, [x0, #0x2f0]

mov x4, sp
// Save SP.
str x4, [x0, #0xf8]

ldp x19, x20,  [x1, #0x98]
ldp x21, x22,  [x1, #0xa8]
ldp x23, x24,  [x1, #0xb8]
ldp x25, x26,  [x1, #0xc8]
ldp x27, x28,  [x1, #0xd8]
ldp x29, x30,  [x1, #0xe8]

/* SIMD state is still expensive. */
ldr q0, [x1, #0x100]
ldr q1, [x1, #0x110]
ldr q2, [x1, #0x120]
ldr q3, [x1, #0x130]
ldr q4, [x1, #0x140]
ldr q5, [x1, #0x150]
ldr q6, [x1, #0x160]
ldr q7, [x1, #0x170]
ldr q8, [x1, #0x180]
ldr q9, [x1, #0x190]
ldr q10, [x1, #0x1a0]
ldr q11, [x1, #0x1b0]
ldr q12, [x1, #0x1c0]
ldr q13, [x1, #0x1d0]
ldr q14, [x1, #0x1e0]
ldr q15, [x1, #0x1f0]
ldr q16, [x1, #0x200]
ldr q17, [x1, #0x210]
ldr q18, [x1, #0x220]
ldr q19, [x1, #0x230]
ldr q20, [x1, #0x240]
ldr q21, [x1, #0x250]
ldr q22, [x1, #0x260]
ldr q23, [x1, #0x270]
ldr q24, [x1, #0x280]
ldr q25, [x1, #0x290]
ldr q26, [x1, #0x2a0]
ldr q27, [x1, #0x2b0]
ldr q28, [x1, #0x2c0]
ldr q29, [x1, #0x2d0]
ldr q30, [x1, #0x2e0]
ldr q31, [x1, #0x2f0]

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


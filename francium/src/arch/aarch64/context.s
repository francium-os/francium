.global restore_process_context
.global get_spsr_el1
.global get_sp_el0

.section .text
get_spsr_el1:
mrs x0, spsr_el1
ret

get_sp_el0:
mrs x0, sp_el0
ret

restore_process_context:
// context in x0, x1 is scratch
ldp x2, x3,    [x0, #0x10]
ldp x4, x5,    [x0, #0x20]
ldp x6, x7,    [x0, #0x30]
ldp x8, x9,    [x0, #0x40]
ldp x10, x11,  [x0, #0x50]
ldp x12, x13,  [x0, #0x60]
ldp x14, x15,  [x0, #0x70]
ldp x16, x17,  [x0, #0x80]
ldp x18, x19,  [x0, #0x90]
ldp x20, x21,  [x0, #0xa0]
ldp x22, x23,  [x0, #0xb0]
ldp x24, x25,  [x0, #0xc0]
ldp x26, x27,  [x0, #0xd0]
ldp x28, x29,  [x0, #0xe0]
ldr x30,       [x0, #0xf0]

// "x31" is SP - set it.
ldr x1,		   [x0, #0xf8]
msr sp_el0, x1

// load PC, set it
ldr x1,		   [x0, #0x100]
msr elr_el1, x1

// load SPSR, set it
ldr x1,		   [x0, #0x108]
msr spsr_el1, x1

ldp x0, x1,    [x0]
eret
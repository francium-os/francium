.global _start
.extern kernel_start
.extern initial_level_0_table
.extern initial_level_1_table
# this file expects to be loaded at whatever address 
# needs to be ~ position independent

# regions of interest:
# ram starts at 0GB
# arm says No, you can't have a block mapping at level 0
# so we have to make a level 1 table with the blocks in it...

.equ KERNEL_BASE, 0xfffffff800000000

.equ SCTLR_LSMAOE, (1<<29)
.equ SCTLR_NTLSMD, 1<<28
.equ SCTLR_TSCXT,  1<<20
.equ SCTLR_ITD, 1<<7

.equ SCTLR_I, 1 << 12
.equ SCTLR_SPAN, 1 << 3
.equ SCTLR_C, 1 << 2
.equ SCTLR_M, 1 << 0

.equ TG1_4KB, (0b10<<30)
.equ SH1_NS, (0b00<<28)
.equ ORGN1_NC, (0b00<<26)
.equ IRGN1_NC, (0b00<<24)

.section .text.entry
_start:

mrs x2, currentel
cmp x2, 0x8
bne .already_el1
# we are in el2, switch down to el1

ldr x2, =(.already_el1 - KERNEL_BASE + PHYS_BASE)
msr elr_el2, x2
# EL1h (SPSel = 1) with interrupt disabled
ldr x2, =0x3c5 
msr spsr_el2, x2

# el1 uses aarch64
ldr x2, =(1<<31) 
msr hcr_el2, x2
eret

.already_el1:
# setup mair_el1
# attr 0 = 0xff (normal memory, write through, non transient), attr 1 = normal noncacheable, attr 2 = 0x00 (device-ngnrne)
ldr x2, =0x0044ff
msr mair_el1, x2

# set page tables
ldr x0, =(initial_level_0_table - KERNEL_BASE + PHYS_BASE)
msr ttbr0_el1, x0
msr ttbr1_el1, x0

# Zero works well enough. Ish.
# Set T0SZ / T1SZ to 16.
ldr x0, = TG1_4KB | (16 << 16) | (16 << 0)
msr tcr_el1, x0

ldr x0, = SCTLR_LSMAOE | SCTLR_NTLSMD | SCTLR_TSCXT | SCTLR_SPAN | SCTLR_I | SCTLR_C | SCTLR_M
msr sctlr_el1, x0

dsb sy
isb sy

# Disable trapping of SIMD/FP instructions.
mrs    x1, cpacr_el1
mov    x0, #(3 << 20)
orr    x0, x1, x0
msr    cpacr_el1, x0

# This is important - if we do a `b kernel_start` it will be relative.
ldr x0, =kernel_start
br x0

.section .rodata.pagetables
.balign 4096
initial_level_0_table:
# lol thanks clang
# +3 is relocatable, |3 isn't

# table, level 1.
.quad (initial_linear_memory_table - KERNEL_BASE + PHYS_BASE) + (1<<10) + 3
.rept 447
.quad 0
.endr
.quad (initial_device_memory_table - KERNEL_BASE + PHYS_BASE) + (1<<10) + 3
.rept 31
.quad 0
.endr
.quad (initial_linear_memory_table - KERNEL_BASE + PHYS_BASE) + (1<<10) + 3
.rept 30
.quad 0
.endr

# map kernel
.quad (initial_kernel_map - KERNEL_BASE + PHYS_BASE) + (1<<10) + 3

# Map a 512GB block as identity using attr 2 (device memory)
.balign 4096
initial_device_memory_table:
.set i,0
.rept 512
.quad i * 0x40000000 + (1<<10) + (2 << 2) + 1
.set i, i+1
.endr

# Map a 512GB block as identity using attr 0 (normal, cacheable memory)
.balign 4096
initial_linear_memory_table:
.set i,0
.rept 512
.quad i * 0x40000000 + (1<<10) + (0 << 2) + 1
.set i, i+1
.endr

.balign 4096
initial_kernel_map:
.rept 512-32
.quad 0
.endr
.quad PHYS_BASE + (1<<10) + 1
.rept 31
.quad 0
.endr

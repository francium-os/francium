.global _start
.extern kernel_start
.extern initial_level_0_table
.extern initial_level_1_table
# this file expects to be loaded at whatever fuckin address 
# needs to be ~ position independent

# regions of interest:
# ram starts at 1GB or 0x40000000
# arm says No, you can't have a block mapping at level 0
# so we have to make a level 1 table with the blocks in it...

.equ KERNEL_BASE, 0xfffffff800000000
.equ PHYS_BASE, 0x40000000

.equ SCTLR_LSMAOE, (1<<29)
.equ SCTLR_NTLSMD, 1<<28
.equ SCTLR_TSCXT,  1<<20
.equ SCTLR_ITD, 1<<7

.equ SCTLR_I, 1 << 12
.equ SCTLR_SPAN, 1 << 3
.equ SCTLR_C, 1 << 2
.equ SCTLR_M, 1 << 0

.section .text.entry
_start:
# set page tables
ldr x0, =(initial_level_0_table - KERNEL_BASE + PHYS_BASE)
msr ttbr0_el1, x0
msr ttbr1_el1, x0
ldr x0, = SCTLR_LSMAOE | SCTLR_NTLSMD | SCTLR_TSCXT | SCTLR_I | SCTLR_SPAN | SCTLR_C | SCTLR_M;
msr sctlr_el1, x0

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
.quad (initial_level_1_table - KERNEL_BASE + PHYS_BASE) + (1<<10) + 3
.rept 510
.quad 0
.endr
.quad (initial_kernel_map - KERNEL_BASE + PHYS_BASE) + (1<<10) + 3

# map a 512GB block as identity
.balign 4096
initial_level_1_table:
.set i,0
.rept 512
.quad i * 0x40000000 + (1<<10) + 1
.set i, i+1
.endr

.balign 4096
initial_kernel_map:
.rept 512-32
.quad 0
.endr
.quad 0x40000000 + (1<<10) + 1
.rept 31
.quad 0
.endr
.global set_ttbr0_el1
.global set_ttbr1_el1
.global get_sctlr_el1
.global set_sctlr_el1
.global get_tcr_el1
.global set_tcr_el1
.global wfe

.section .text
set_ttbr0_el1:
msr ttbr0_el1, x0

tlbi vmalle1
dsb ish
isb
ret

set_ttbr1_el1:
msr ttbr1_el1, x0

tlbi vmalle1
dsb ish
isb
ret

get_sctlr_el1:
mrs x0, sctlr_el1
ret

set_sctlr_el1:
msr sctlr_el1, x0
ret

get_tcr_el1:
mrs x0, tcr_el1
ret

set_tcr_el1:
msr tcr_el1, x0
ret

wfe:
wfe
ret
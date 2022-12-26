.section .text
.global spin_for_cycles

spin_for_cycles:
.loop: nop
sub x0, x0, 1
cbnz x0, .loop
ret
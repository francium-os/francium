.global spin_for_cycles
.global clear_cache_for_address

spin_for_cycles:
.loop: nop
sub x0, x0, 1
cbnz x0, .loop
ret

clear_cache_for_address:
dc cvau, x0
ic ivau, x0
ret

.global switch_thread_asm
.extern force_unlock_mutex

switch_thread_asm:
xchg bx, bx
.a: jmp .a
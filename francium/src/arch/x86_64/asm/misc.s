.global invalidate_tlb
.global clear_cache_for_address
.global restore_thread_context
.global read_cr3

invalidate_tlb:
xchg bx, bx
ret

clear_cache_for_address:
xchg bx, bx
ret

restore_thread_context:
xchg bx, bx
a: jmp a
ret

read_cr3:
mov rax, cr3
ret
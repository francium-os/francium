.global invalidate_tlb
.global clear_cache_for_address
.global restore_thread_context

invalidate_tlb:
ret

clear_cache_for_address:
ret

restore_thread_context:
int 3
a: jmp a
ret
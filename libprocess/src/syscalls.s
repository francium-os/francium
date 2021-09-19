.global syscall_debug_output

.section .text
syscall_debug_output:
svc #1
ret
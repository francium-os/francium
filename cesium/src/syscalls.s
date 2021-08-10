.global syscall_print

.section .text
syscall_print:
svc #0
ret
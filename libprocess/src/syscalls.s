.global syscall_debug_output
.global syscall_create_port
.global syscall_connect_to_port
.global syscall_exit_process

.section .text
syscall_debug_output:
svc #1
ret

syscall_create_port:
svc #2
ret

syscall_connect_to_port:
svc #3
ret

syscall_exit_process:
svc #4
ret
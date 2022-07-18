.global syscall_debug_output
.global syscall_create_port
.global syscall_connect_to_port
.global syscall_exit_process
.global syscall_close_handle
.global syscall_ipc_request
.global syscall_ipc_reply
.global syscall_ipc_receive
.global syscall_ipc_accept
.global syscall_get_process_id
.global get_tpidr_el0_asm

.section .text
syscall_debug_output:
svc #1
ret

syscall_create_port:
mov x9, x1
svc #2
str w1, [x9]
ret

syscall_connect_to_port:
mov x9, x1
svc #3
str w1, [x9]
ret

syscall_exit_process:
svc #4
ret

syscall_close_handle:
svc #5
ret

syscall_ipc_request:
svc #6
ret

syscall_ipc_reply:
svc #7
ret

syscall_ipc_receive:
mov x9, x2
svc #8
str x1, [x9]
ret

syscall_ipc_accept:
mov x9, x1
svc #9
str x1, [x9]
ret

syscall_get_process_id:
svc #0x0a
ret

get_tpidr_el0_asm:
mrs x0, tpidr_el0
ret
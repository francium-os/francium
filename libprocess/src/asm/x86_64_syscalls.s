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

.global syscall_break

.section .text
syscall_break:
syscall #0
ret

syscall_debug_output:
syscall #0
ret

syscall_create_port:
syscall #0
ret

syscall_connect_to_port:
syscall #0
ret

syscall_exit_process:
syscall #0
ret

syscall_close_handle:
syscall #0
ret
syscall_ipc_request:
syscall #0
ret
syscall_ipc_reply:
syscall #0
ret
syscall_ipc_receive:
syscall #0
ret
syscall_ipc_accept:
syscall #0
ret
syscall_get_process_id:
syscall #0
ret

.global syscall_break
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
.global syscall_map_memory
.global syscall_sleep_ns
.global syscall_get_thread_id
.global syscall_create_thread
.global syscall_futex_wait
.global syscall_futex_wake
.global syscall_map_device_memory
.global syscall_get_system_tick
.global syscall_query_physical_address
.global syscall_create_event
.global syscall_bind_interrupt
.global syscall_unbind_interrupt

.global get_tpidr_el0_asm

.section .text
syscall_break:
svc #0
ret

syscall_debug_output:
svc #1
ret

syscall_create_port:
mov x9, x1
svc #2
str w1, [x9]
ret

syscall_connect_to_named_port:
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
mov x9, x3
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

syscall_connect_to_port_handle:
mov x9, x1
svc #0x0b
str w1, [x9]
ret

syscall_map_memory:
mov x9, x3
svc #0x0c
str x1, [x9]
ret

syscall_sleep_ns:
svc #0x0d
ret

// Currently should not be used on ARM!
syscall_bodge:
brk #0
ret

syscall_get_thread_id:
svc #0x0f
ret

syscall_create_thread:
mov x9, x2
svc #0x10
str w1, [x9]
ret

syscall_futex_wait:
svc #0x11
ret

syscall_futex_wake:
svc #0x12
ret

syscall_map_device_memory:
mov x9, x5
svc #0x13
str x1, [x9]
ret

// syscall_get_system_info

syscall_get_system_tick:
svc #0x15
ret

syscall_query_physical_address:
mov x9, x1
svc #0x16
str x1, [x9]
ret

syscall_create_event:
mov x9, x0
svc #0x17
str w1, [x9]
ret

syscall_bind_interrupt:
svc #0x18
ret

syscall_unbind_interrupt:
svc #0x19
ret

syscall_wait_one:
svc #0x1a
ret

get_tpidr_el0_asm:
mrs x0, tpidr_el0
ret


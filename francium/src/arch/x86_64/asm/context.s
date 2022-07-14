.global setup_initial_thread_context
.global user_thread_starter


.section .text
setup_initial_thread_context:
// args: ctx: rdi, mutex: rsi
xchg bx, bx

// store ctx in rbx which is saved
mov rbx, rdi
mov rsp, [rbx + 152]

mov rdi, rsi
call force_unlock_mutex

// push return addr
mov rax, [rbx + 128]
push rax

// now zero everything
mov rax, 0
mov rbx, 0
mov rcx, 0
mov rdx, 0
mov rbp, 0
mov rsi, 0
mov rdi, 0
mov r8, 0
mov r9, 0
mov r10, 0
mov r11, 0
mov r12, 0
mov r13, 0
mov r14, 0
mov r15, 0

xchg bx, bx
ret

// As defined in interrupt.s
user_thread_starter:
jmp restore_exception_context
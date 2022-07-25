.global switch_thread_asm
.extern force_unlock_mutex

// sysv abi register order: rdi, rsi, rdx, rcx, r8, r9
// sysv abi callee save: rbx, rsp, rbp, r12, r13, r14, and r15;

// rdi: from context
// rsi: to context
// rdx: from mutex
// rcx: to mutex

switch_thread_asm:
// Save callee save registers.
mov [rdi + 8], rbx
mov [rdi + 32], rbp
mov [rdi + 88], r12
mov [rdi + 96], r13
mov [rdi + 104], r14
mov [rdi + 112], r15

// Pop return address off the stack, and save it in the context.
pop rax
mov [rdi + 128 + 8], rax

// Save SP.
mov [rdi + 152 + 8], rsp

// Load our new registers.
mov rbx, [rsi + 8]
mov rbp, [rsi + 32]
mov r12, [rsi + 88]
mov r13, [rsi + 96]
mov r14, [rsi + 104]
mov r15, [rsi + 112]

// Restore SP.
mov rsp, [rsi + 152 + 8]

// Unlock the mutex for the from context.
mov rax, rdx
call force_unlock_mutex

// Unlock the mutex for the to context.
mov rax, rcx
call force_unlock_mutex

// load tag
mov rax, [rsi]

xchg bx, bx
mov r10, [rsi + 128 + 8]
push r10
ret

xchg bx, bx
.a: jmp .a
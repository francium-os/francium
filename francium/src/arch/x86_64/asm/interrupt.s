.global exception_error
.global exception_no_error
.global restore_exception_context

// stack layout:
// ss (+40)
// rsp (+32)
// rflags (+24)
// cs  (+16)
// rip (+8)
// error code (+0)

exception_no_error:
push qword ptr 0
jmp exception_error

exception_error:
push r15
push r14
push r13
push r12
push r11
push r10
push r9
push r8
push rdi
push rsi
push rbp
push rdx
push rcx
push rbx
push rax

// Reach back into the stack to grab the error code...
lea rsi, [rsp - 16*8]
mov rdi, [rsp - 16*8]
xchg bx, bx

// falls through
restore_exception_context:
pop rax
pop rbx
pop rcx
pop rdx
pop rbp
pop rsi
pop rdi
pop r8
pop r9
pop r10
pop r11
pop r12
pop r13
pop r14
pop r15

// Drop error code off stack
add rsp, 8

iret
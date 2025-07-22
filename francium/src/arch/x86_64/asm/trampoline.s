.global ap_trampoline
.global ap_trampoline_end
.global ap_entry

// this code will be relocated to 0x8000, sets up environment for calling a C function

.align 64

.code16
ap_trampoline:
    cli
    cld
    ljmp 0, 0x8040
.align 16
per_cpu_table_location:
.quad 0
initial_cr3:
.quad 0
trampoline_location:
.quad 0
.align 32
_L8020_GDT_table:
    .quad 0x0000000000000000
    .quad 0x00209A0000000000
    .quad 0x0000920000000000
    .align 16

_L8040_GDT_value:
    .word _L8040_GDT_value - _L8020_GDT_table - 1
    .long 0x8020
    .long 0, 0

.align 16
_L8060:
    xor ax, ax
    mov    ds, ax

    // Set PAE, PGE bits
    mov eax, 0b10100000
    mov cr4, eax
    mov eax, [0x8018]
    mov cr3, eax

    mov ecx, 0xC0000080 // EFER
    rdmsr
    or eax, 0x00000100 // set LME
    wrmsr

    mov ebx, cr0
    or ebx,0x80000001
    mov cr0, ebx

    lgdt   [0x8040]

    ljmp 8, 0x8090

    .align 16
    .code64
_L8090:
    
    // Get the CPU number (well, apic ID)
    mov rax, 1
    cpuid
    shr ebx, 24
    mov edi, ebx

    mov ecx, 0xC0000080 // EFER
    rdmsr
    or eax, 1<<11 // set NX enable
    wrmsr

    // Now go!
    mov rax, qword ptr [0x8020]
    // - 0xfffffff800000000)
    //mov rbx, 0xfffffff800000000
    //add rax, rbx
    jmp rax

ap_trampoline_end:

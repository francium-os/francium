.global ap_trampoline
.global ap_trampoline_end

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
    jmp _L8090

ap_trampoline_end:

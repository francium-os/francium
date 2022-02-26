.global _start
.global multiboot_header

.extern kernel_start

.section .text.entry
_start:
a:
jmp a

.section .text.multiboot
.balign 8
multiboot_header:
.long 0xE85250D6
.long 0
.long multiboot_header_end - multiboot_header
.long -(0xE85250D6 + 0 + (multiboot_header_end - multiboot_header))

multiboot_end_tag:
.short 0
.short 0
.long 8
multiboot_header_end:
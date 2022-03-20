ENTRY(_start)

SECTIONS
{
  KERNEL_BASE = 0xfffffff800000000;

  . = KERNEL_BASE;
  .text : AT(0x40000000)
  {
    __text_start = .;
    *(.text.entry)
    . = KERNEL_BASE + 0x800;
    __vbar = .;

    KEEP(*(.text.exceptions))

    . = ALIGN(0x1000);
    *(.text .text.*);
    __text_end = .;
  }
  .rodata : ALIGN(0x1000)
  {
    __rodata_start = .;
    *(.rodata .rodata.*)
    __rodata_end = .;
  }
  .data : ALIGN(0x1000)
  {
    __data_start = .;
    *(.data .data.*)
    __data_end = .;
  }
  .bss : ALIGN(0x1000)
  {
    __bss_start = .;
    *(.bss .bss.*)
    __bss_end = .;
  }
  
  /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}

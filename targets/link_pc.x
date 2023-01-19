ENTRY(_start)

SECTIONS
{
  KERNEL_BASE = 0xfffffff800000000;

  . = KERNEL_BASE;
  .text : AT(0x40000000)
  {
    __text_start = .;
    *(.text.entry)
    KEEP(*(.text.multiboot))

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

  .bootloader-config :
  {
    *(.bootloader-config)
  }

  .hash : { *(.hash) }
  .dynsym : { *(.dynsym) }
  .dynstr : { *(.dynstr) }

  .rela.dyn : { *(.rela.dyn) }
  .rela.plt : {  *(.rela.plt) }

  . = ALIGN(0x1000);

  .data.rel.ro : { *(.data.rel.ro.local* .gnu.linkonce.d.rel.ro.local.*) *(.data.rel.ro .data.rel.ro.* .gnu.linkonce.d.rel.ro.*) }
  .dynamic        : { *(.dynamic) }
  .got            : { *(.got) *(.igot) }
  . = DATA_SEGMENT_RELRO_END (SIZEOF (.got.plt) >= 24 ? 24 : 0, .);
  .got.plt        : { *(.got.plt) *(.igot.plt) }

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

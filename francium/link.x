ENTRY(_start)

EXTERN(exception_handler)

SECTIONS
{
  # memory starts at 1gb, lmao

  . = 0x40000000;

  .text : ALIGN(0x1000)
  {
    __text_start = .;

    *(.text.entry)
    . = 0x40000200;
    *(.text.exceptions)

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
    *(.bss)
    __bss_end = .;
  }
}

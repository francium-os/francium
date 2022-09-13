ENTRY(_start);

SECTIONS
{
  . = 0x100000;
  .text ALIGN(0x1000) :
  {
    . = .;

    __text_start = .;
    *(.text .text.*);
    __text_end = .;
  }

  .rodata ALIGN(0x1000) :
  {
    __rodata_start = .;
    *(.rodata .rodata.*)
    __rodata_end = .;
  }

  .data ALIGN(0x1000) :
  {
    __data_start = .;
    *(.data .data.*)
    __data_end = .;
  }

  .bss ALIGN(0x1000) :
  {
    __bss_start = .;
    *(.bss .bss.*)
    __bss_end = .;
  }

  .tdata : {
    *(.tdata.IPC_BUFFER)
    *(.tbss.IPC_BUFFER)

    *(.tdata.*)
    *(.tbss.*)
  } :tls

  . = ALIGN(0x1000);
}

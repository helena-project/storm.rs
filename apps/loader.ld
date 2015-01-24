OUTPUT_FORMAT("elf32-littlearm", "elf32-littlearm", "elf32-littlearm")
OUTPUT_ARCH(arm)

SECTIONS
{
  .text ALIGN(0x4) :
  {
    *(.text*)
  }

  .rodata ALIGN(0x4) :
  {
    *(.rodata*)
  }

  .data ALIGN(0x4) :
  {
    *(.data)
  }

  .bss ALIGN(0x4) :
  {
    sbss = .;
    *(COMMON)
    *(.bss)
    ebss = .;
  }

  .apps ALIGN(0x4) :
  {
    apps_start = .;
    *(.app.*)
    apps_end = .;
  }
}

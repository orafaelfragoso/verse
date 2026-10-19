MEMORY { FLASH : ORIGIN = 0x08000000, LENGTH = 512K  RAM : ORIGIN = 0x20000000, LENGTH = 128K }
ENTRY(rt_panic)
SECTIONS {
  .text : { KEEP(*(.text.rt_panic)) KEEP(*(.text.drop_drain)) *(.text .text.*) } > FLASH
  .rodata : { *(.rodata .rodata.*) } > FLASH
  .data : { *(.data .data.*) } > RAM AT > FLASH
  .bss (NOLOAD) : { *(.bss .bss.*) } > RAM
  /DISCARD/ : { *(.ARM.exidx .ARM.exidx.*) }
}

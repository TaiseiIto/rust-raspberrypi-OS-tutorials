/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>
 */

/* The address at which the the kernel binary will be loaded by the Raspberry's firmware */
/* Raspberryのfirmwareによって読み込まれるkernel bynaryのaddress */
/* この番地がentry pointだよ */
/* segmentの権限関係? */
__rpi_phys_dram_start_addr = 0;

/* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
__rpi_phys_binary_load_addr = 0x80000;


ENTRY(__rpi_phys_binary_load_addr)

/* Flags:
 *     4 == R
 *     5 == RX
 *     6 == RW
 *
 * Segments are marked PT_LOAD below so that the ELF file provides virtual and physical addresses.
 * It doesn't mean all of them need actually be loaded.
 */
PHDRS
{
    segment_boot_core_stack PT_LOAD FLAGS(6);
    segment_code            PT_LOAD FLAGS(5);
    segment_data            PT_LOAD FLAGS(6);
}

SECTIONS
{
    /* 現在地はentry pointだよ */
    /* stackはextry pointから上(番地の若い)方向に伸びていくらしい． */
    . =  __rpi_phys_dram_start_addr;

    /***********************************************************************************************
    * Boot Core Stack
    ***********************************************************************************************/
    .boot_core_stack (NOLOAD) :
    {
                                             /*   ^             */
                                             /*   | stack       */
        . += __rpi_phys_binary_load_addr;    /*   | growth      */
                                             /*   | direction   */
        __boot_core_stack_end_exclusive = .; /*   |             */
    } :segment_boot_core_stack

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    .text :
    {
        KEEP(*(.text._start))     /* ここに../../_arch/aarch64/cpu/boot.sの.text._startが配置されるっぽい */
        *(.text._start_arguments) /* Constants (or statics in Rust speak) read by _start(). */
        *(.text._start_rust)      /* The Rust entry point */
        *(.text*)                 /* Everything else */
    } :segment_code

    .rodata : ALIGN(8) { *(.rodata*) } :segment_code
    .got    : ALIGN(8) { *(.got)     } :segment_code

    /***********************************************************************************************
    * Data + BSS ../../../README.mdの最初に書いてあるBSSがここで定義されてるっぽい
    ***********************************************************************************************/
    .data : { *(.data*) } :segment_data

    /* Section is zeroed in pairs of u64. Align start and end to 16 bytes */
    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .; /* このsymbolは，./memory.rsでRustで.bss領域の範囲を定義するために使われている */
        *(.bss*);
                /* なんで8足してもう一度引いてるんだ? */
                /* 多分inclusiveだから、endの後8バイトもbss領域に含めたいのか */
        . = ALIGN(16);
        __bss_end_exclusive = .;
    } :segment_data
}

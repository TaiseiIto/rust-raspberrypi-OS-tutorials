/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
 */

/* The address at which the the kernel binary will be loaded by the Raspberry's firmware */
/* Raspberryのfirmwareによって読み込まれるkernel bynaryのaddress */
__rpi_load_addr = 0x80000;

/* この番地がentry pointだよ */
ENTRY(__rpi_load_addr)

/* segmentの権限関係? */
PHDRS
{
    segment_rx PT_LOAD FLAGS(5); /* 5 == RX 読み実行可能 */
    segment_rw PT_LOAD FLAGS(6); /* 6 == RW 読み書き可能 */
}

SECTIONS
{
    /* 現在地はentry pointだよ */
    . =  __rpi_load_addr;
                                        /*   ^             */
                                        /*   | stack       */
                                        /*   | growth      */
                                        /*   | direction   */
   __boot_core_stack_end_exclusive = .; /*   |             */
   /* stackはextry pointから上(番地の若い)方向に伸びていくらしい． */

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    .text :
    {
        KEEP(*(.text._start))     /* ここに../../_arch/aarch64/cpu/boot.sの.text._startが配置されるっぽい */
        *(.text._start_arguments) /* Constants (or statics in Rust speak) read by _start(). */
        *(.text._start_rust)      /* The Rust entry point */
        *(.text*)                 /* Everything else */
    } :segment_rx                 /* 読み実行可能 */

    .rodata : ALIGN(8) { *(.rodata*) } :segment_rx
    .got    : ALIGN(8) { *(.got)     } :segment_rx

    /***********************************************************************************************
    * Data + BSS ../../../README.mdの最初に書いてあるBSSがここで定義されてるっぽい
    ***********************************************************************************************/
    .data : { *(.data*) } :segment_rw   /* 読み書き可能 */

    /* Section is zeroed in u64 chunks, align start and end to 8 bytes */
    .bss : ALIGN(8)
    {
        __bss_start = .; /* このsymbolは，./memory.rsでRustで.bss領域の範囲を定義するために使われている */
        *(.bss*);
        . = ALIGN(8);
                /* なんで8足してもう一度引いてるんだ? */
                /* 多分inclusiveだから、endの後8バイトもbss領域に含めたいのか */
        . += 8; /* Fill for the bss == 0 case, so that __bss_start <= __bss_end_inclusive holds */
        __bss_end_inclusive = . - 8; /* このsymbolは，./memory.rsでRustで.bss領域の範囲を定義するために使われている */
    } :NONE
}

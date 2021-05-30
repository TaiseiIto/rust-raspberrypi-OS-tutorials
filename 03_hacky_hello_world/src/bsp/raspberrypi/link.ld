/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
 */

/* The address at which the the kernel binary will be loaded by the Raspberry's firmware */
/* この番地がentry pointになる */
__rpi_load_addr = 0x80000;

/* ここがentry pointだよという宣言っぽい */
ENTRY(__rpi_load_addr)

PHDRS /* ここでmemory上の各領域に設定される権限を定義してるっぽい */
{
    segment_rx PT_LOAD FLAGS(5); /* 5 == RX 読み実行可能 */
    segment_rw PT_LOAD FLAGS(6); /* 6 == RW 読み書き可能*/
}

SECTIONS
{
    . =  __rpi_load_addr; /* entry point以降のメモリ領域を定義していくよ． */
                                        /*   ^             */
                                        /*   | stack       */
                                        /*   | growth      */
                                        /*   | direction   */
   __boot_core_stack_end_exclusive = .; /*   | stackはentry pointから番地の若い方向に伸びてくよ． */

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    .text : /* ここは実行codeだよ */
    {
        KEEP(*(.text._start))     /* ../../_arch/<arch_name>/cpu/boot.sで定義されてるassemblyのentry pointである_start()がここに対応してるよ． */
        *(.text._start_arguments) /* Constants (or statics in Rust speak) read by _start(). */
        *(.text._start_rust)      /* The Rust entry point */
        *(.text*)                 /* Everything else */
    } :segment_rx /* ここは読み実行可能だよ． */

    .rodata : ALIGN(8) { *(.rodata*) } :segment_rx /* 読み実行可能だよ */
    .got    : ALIGN(8) { *(.got)     } :segment_rx /* 読み実行可能だよ */

    /***********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/
    .data : { *(.data*) } :segment_rw /* ここから読み書き可能なdata領域だよ */

    /* Section is zeroed in u64 chunks, align start and end to 8 bytes */
    .bss : ALIGN(8)
    {
        __bss_start = .; /* ここから明示的に初期化されないdataが置かれる.bss領域だよ */
        *(.bss*);
        . = ALIGN(8);

        . += 8; /* Fill for the bss == 0 case, so that __bss_start <= __bss_end_inclusive holds */
        __bss_end_inclusive = . - 8; /* ここが.bss領域の終わりだけど，この8bytesも含めて.bss領域だよ． */
    } :NONE
}

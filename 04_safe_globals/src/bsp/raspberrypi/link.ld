/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
 */

/* The address at which the the kernel binary will be loaded by the Raspberry's firmware */
/* ここがentry pointだよ */
__rpi_load_addr = 0x80000;

/* entry pointの指定 */
ENTRY(__rpi_load_addr)

/* memoryの各領域に設定される権限をここで定義するよ */
PHDRS
{
    segment_rx PT_LOAD FLAGS(5); /* 5 == RX 読み実行可能 */
    segment_rw PT_LOAD FLAGS(6); /* 6 == RW 読み書き可能 */
}

SECTIONS
{
    . =  __rpi_load_addr; /* entry point以降の領域を定義するよ */
                                        /*   ^             */
                                        /*   | stack       */
                                        /*   | growth      */
                                        /*   | direction   */
   __boot_core_stack_end_exclusive = .; /*   | stackはentry pointから番地の若い方向に伸びてくよ */

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    .text : /* entry pointに実行codeを配置するよ */
    {
        KEEP(*(.text._start))     /* ここに../../_arch/__arch_name__/cpu/boot.sで定義される起動直後に実行されるassembly関数_start()を配置するよ */
        *(.text._start_arguments) /* Constants (or statics in Rust speak) read by _start(). */
        *(.text._start_rust)      /* The Rust entry point */
        *(.text*)                 /* Everything else */
    } :segment_rx /* 読み実行可能だよ */

    /* これは何なのかよくわからん */
    .rodata : ALIGN(8) { *(.rodata*) } :segment_rx /* 読み実行可能だよ */
    .got    : ALIGN(8) { *(.got)     } :segment_rx /* 読み実行可能だよ */

    /***********************************************************************************************
    * Data + BSS 実行中に一貫して同一の実体として存在する変数の場所だよ
    ***********************************************************************************************/
    .data : { *(.data*) } :segment_rw /* 読み書き可能だよ */

    /* Section is zeroed in u64 chunks, align start and end to 8 bytes */
    .bss : ALIGN(8) /* 明示的に初期化されないdataが配置される.bss領域だよ */
    {
        __bss_start = .; /* ここからbss領域が始まるよ */
        *(.bss*);
        . = ALIGN(8);

        . += 8; /* Fill for the bss == 0 case, so that __bss_start <= __bss_end_inclusive holds */
        __bss_end_inclusive = . - 8; /* __bss_end_inclusive以降8bytesまでが.bss領域だよ */
    } :NONE
}

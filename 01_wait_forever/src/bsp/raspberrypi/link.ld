/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
 */

/* The address at which the the kernel binary will be loaded by the Raspberry's firmware */
/* Raspberryのfirmwareによって読み込まれるkernel bynaryのaddress */
/* この番地がentry pointっぽい */
__rpi_load_addr = 0x80000;

ENTRY(__rpi_load_addr)

PHDRS
{
    segment_rx PT_LOAD FLAGS(5); /* 5 == RX */
}

SECTIONS
{
    /* ここがentry point(0x80000) */
    . =  __rpi_load_addr;

    /***********************************************************************************************
    * Code
    ***********************************************************************************************/
    .text :
    {
        /* ここが../../_arch/aarch64/cpi/boot.sの.section.text._startに対応してるっぽい */
        KEEP(*(.text._start))
    } :segment_rx
}

/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>
 */

/* Raspberryのfirmwareによって読み込まれるkernel bynaryのaddress */
/* この番地がentry pointっぽい */
/* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
__rpi_phys_binary_load_addr = 0x80000;


ENTRY(__rpi_phys_binary_load_addr)

/* Flags:
 *     4 == R
 *     5 == RX
 *     6 == RW
 */
PHDRS
{
    segment_code PT_LOAD FLAGS(5);
}

SECTIONS
{
    /* ここがentry point(0x80000) */
    . =  __rpi_phys_binary_load_addr;

    /***********************************************************************************************
    * Code
    ***********************************************************************************************/
    .text :
    {
        /* ここが../../_arch/aarch64/cpi/boot.sの.section.text._startに対応してるっぽい */
        KEEP(*(.text._start))
    } :segment_code
}

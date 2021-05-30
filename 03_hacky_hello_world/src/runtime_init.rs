// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Rust runtime initialization code.

use crate::{bsp, memory};

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// Zero out the .bss section.
/// .bss領域のすべてのbitを0で初期化するよ．
/// # Safety
/// - `kernel_init()`に飛ぶ前に一度だけ実行されるよ．
/// - Must only be called pre `kernel_init()`.
#[inline(always)]
unsafe fn zero_bss() {
    memory::zero_volatile(bsp::memory::bss_range_inclusive()/* bps/raspberrypi/memory.rsで定義されている.bss領域 */); // memory.rsのzero_volatileに飛ぶよ．
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to kernel
/// init code.
/// 各_arch/__arch_name__/cpu/boot.rsからここに飛んでくるよ．CやC++の`crt0`や`c0`と同等だよ．`bss`領域を初期化してmain.rsのkernel_initに飛ぶよ．
/// # Safety
///
/// - Only a single core must be active and running this function.
pub unsafe fn runtime_init() -> ! {
    zero_bss(); // bss領域の全てのbitを0で初期化するよ．

    crate::kernel_init() //main.rsのkernel_initに飛ぶよ．
}

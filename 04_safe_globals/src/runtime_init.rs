// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Rust runtime initialization code.

use crate::{bsp, memory};

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// Zero out the .bss section.
/// bss領域を0で初期化するよ．
/// # Safety
///
/// - Must only be called pre `kernel_init()`.
/// - `kernel_init()`の前に呼び出す必要があるよ．
/// - ./bsp/raspberrypi/memory.rsで定義されているbss領域を./memory.rsのzero_volatile関数で初期化するよ．
#[inline(always)]
unsafe fn zero_bss() {
    memory::zero_volatile(bsp::memory::bss_range_inclusive());
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to kernel
/// init code.
/// CやC++の`crt0`や`c0`に相当するやつだよ．mainを実行する前に必要な初期化の処理だよ．
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - 単一のcoreのみがこの関数を実行するよ．
pub unsafe fn runtime_init() -> ! {
    zero_bss();// bss領域を0で初期化するよ．

    crate::kernel_init() // ./main.rsのkernel_initに飛ぶよ．
}

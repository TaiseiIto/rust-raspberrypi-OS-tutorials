// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Rust runtime initialization code.

use crate::{bsp, memory};

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// Zero out the .bss section.
/// .bss領域を0埋め
/// # Safety
///
/// - Must only be called pre `kernel_init()`.
/// - 下のkernel_init()から呼び出される
#[inline(always)]
unsafe fn zero_bss() {
    // .bssの領域をrangeで指定して./memory.rsの0埋め関数を呼ぶ．
    // ./src/raspberrypi/memory.rsのbss_range_inclusive関数が.bss領域を返すようになってる．
    memory::zero_volatile(bsp::memory::bss_range_inclusive());
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// ./_arch/aarch64/cpu/boot.rsの_start_rust()からここに飛ぶ
/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to kernel
/// init code.
/// C/C++の`crt0`や`c0`と同等(なんじゃそりゃ)．bss領域を初期化してkernel init codeに飛ぶ．
/// # Safety
///
/// - Only a single core must be active and running this function.
pub unsafe fn runtime_init() -> ! {
    // bssを0埋め
    zero_bss();
    // ./main.rsのkernel_initへ飛ぶ
    crate::kernel_init()
}

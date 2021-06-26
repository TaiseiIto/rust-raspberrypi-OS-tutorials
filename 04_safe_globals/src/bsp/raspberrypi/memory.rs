// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Management.

use core::{cell::UnsafeCell, ops::RangeInclusive};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Symbols from the linker script.
extern "Rust" {
    static __bss_start: UnsafeCell<u64>;
    static __bss_end_inclusive: UnsafeCell<u64>;
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return the inclusive range spanning the .bss section.
/// bss領域を定義しているよ．
/// 始点__bss_startと終点__bss_end_inclusiveは./link.ldで定義されているよ．
/// 符号なし64bit変数の配列の上下境界含む範囲を返すよ
/// # Safety
///
/// - Values are provided by the linker script and must be trusted as-is.
/// - The linker-provided addresses must be u64 aligned.
pub fn bss_range_inclusive() -> RangeInclusive<*mut u64> {
    let range;
    // 関数内にunsafeがあるけど関数自体もunsafeにならないのか?
    unsafe {
        range = RangeInclusive::new(__bss_start.get(), __bss_end_inclusive.get());
    }

    // 「rangeの範囲が空でない」が満たされなかった場合にpanic!マクロを呼び出すマクロ
    assert!(!range.is_empty());

    // rangeを返すよ
    range
}

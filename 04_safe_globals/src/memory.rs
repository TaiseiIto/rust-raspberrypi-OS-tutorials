// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management.

use core::ops::RangeInclusive;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Zero out an inclusive memory range.
/// メモリ領域[range.start(), range.end()]を0で初期化するよ．
/// # Safety
///
/// - `range.start` and `range.end` must be valid.
/// - `range.start` and `range.end` must be `T` aligned.
pub unsafe fn zero_volatile<T>(range: RangeInclusive<*mut T>)
where
    T: From<u8>, // From trait(u8からキャスト可能)を実装した型T
{
    let mut ptr = *range.start(); // 0を書き込むためのポインタ
    let end_inclusive = *range.end(); // 初期化範囲の終点

    while ptr <= end_inclusive {
        // write_volatileはコンパイラの最適化で消されたり順序が変更されることのないwrite
        core::ptr::write_volatile(ptr, T::from(0)); // u8型の0をT型に変換したやつをptrに書き込む
        ptr = ptr.offset(1); // ポインタを進める
    }
}

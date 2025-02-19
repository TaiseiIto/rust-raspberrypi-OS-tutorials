// 今回追加されたソースファイル

// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! General purpose code.

/// Check if a value is aligned to a given size.
#[inline(always)]
pub const fn is_aligned(value: usize, alignment: usize) -> bool {
    // アライメントは2のn乗
    assert!(alignment.is_power_of_two());
    // アライメントされているか確認
    (value & (alignment - 1)) == 0
}

/// Align down.
#[inline(always)]
pub const fn align_down(value: usize, alignment: usize) -> usize {
    // アライメントは2のn乗
    assert!(alignment.is_power_of_two());
    // アライメントする
    value & !(alignment - 1)
}

/// Align up.
#[inline(always)]
pub const fn align_up(value: usize, alignment: usize) -> usize {
    assert!(alignment.is_power_of_two());

    (value + alignment - 1) & !(alignment - 1)
}

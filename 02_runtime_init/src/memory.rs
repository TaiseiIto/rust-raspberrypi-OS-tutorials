// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management.

use core::ops::RangeInclusive;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Zero out an inclusive memory range.
/// memory領域の0埋め
/// # Safety
///
/// - `range.start` and `range.end` must be valid.
/// - `range.start` and `range.end` must be `T` aligned.
/// ./runtime_init.rsのzero_bssから，memoryの.bss領域を0埋めするために呼び出されてる
pub unsafe fn zero_volatile<T>(range: RangeInclusive<*mut T>)
where
    T: From<u8>,
{
    let mut ptr = *range.start();
    let end_inclusive = *range.end();
    // rangeのstartからendまで0埋め
    while ptr <= end_inclusive {
        core::ptr::write_volatile(ptr, T::from(0));
        ptr = ptr.offset(1);
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management.

pub mod mmu;

use crate::common;
use core::{
    fmt,
    marker::PhantomData,
    ops::{AddAssign, RangeInclusive, SubAssign},
};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Metadata trait for marking the type of an address.
/// アドレスの種類を表すデータ構造が実装すべきtrait
pub trait AddressType: Copy + Clone + PartialOrd + PartialEq {}

/// Zero-sized type to mark a physical address.
/// 物理アドレスであることを示す空の列挙体
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum Physical {}

/// Zero-sized type to mark a virtual address.
/// 仮想アドレスであることを示す空の列挙体
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum Virtual {}

/// Generic address type.
/// アドレスを表す構造体
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct Address<ATYPE: AddressType> {
    // アドレス
    value: usize,
    // アドレスの種類
    _address_type: PhantomData<fn() -> ATYPE>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

// 物理アドレスであることを示す空の列挙体のAddressType traitの実装
impl AddressType for Physical {}
// 仮想アドレスであることを示す空の列挙体のAddressType traitの実装
impl AddressType for Virtual {}

// アドレスを表す構造体の実装
impl<ATYPE: AddressType> Address<ATYPE> {
    /// Create an instance.
    /// 新規作成
    pub const fn new(value: usize) -> Self {
        Self {
            value,
            _address_type: PhantomData,
        }
    }

    /// Align down.
    /// アライメントされたアドレスを返す関数
    pub const fn align_down(self, alignment: usize) -> Self {
        let aligned = common::align_down(self.value, alignment);

        Self {
            value: aligned,
            _address_type: PhantomData,
        }
    }

    /// Converts `Address` into an usize.
    /// Address構造体をusize型に変換
    pub const fn into_usize(self) -> usize {
        self.value
    }
}

// Address構造体とusizeの足し算
impl<ATYPE: AddressType> core::ops::Add<usize> for Address<ATYPE> {
    type Output = Self;

    // usize型のアドレスを足した結果を返す
    fn add(self, other: usize) -> Self {
        Self {
            // アドレスを足し算
            value: self.value + other,
            _address_type: PhantomData,
        }
    }
}

// Address構造体同士の足し算
impl<ATYPE: AddressType> AddAssign for Address<ATYPE> {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            // アドレスを足し算
            value: self.value + other.into_usize(),
            _address_type: PhantomData,
        };
    }
}

// Address構造体からusize型を引く
impl<ATYPE: AddressType> core::ops::Sub<usize> for Address<ATYPE> {
    type Output = Self;

    fn sub(self, other: usize) -> Self {
        Self {
            // アドレスを引き算
            value: self.value - other,
            _address_type: PhantomData,
        }
    }
}

// Address構造体同士の引き算
impl<ATYPE: AddressType> SubAssign for Address<ATYPE> {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            // アドレスを引き算
            value: self.value - other.into_usize(),
            _address_type: PhantomData,
        };
    }
}

// 物理アドレスの書式
impl fmt::Display for Address<Physical> {
    // Don't expect to see physical addresses greater than 40 bit.
    // 物理アドレスの幅は40bit以下であると仮定している
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let q3: u8 = ((self.value >> 32) & 0xff) as u8;
        let q2: u16 = ((self.value >> 16) & 0xffff) as u16;
        let q1: u16 = (self.value & 0xffff) as u16;

        write!(f, "0x")?;
        write!(f, "{:02x}_", q3)?;
        write!(f, "{:04x}_", q2)?;
        write!(f, "{:04x}", q1)
    }
}

// 仮想アドレスの書式
impl fmt::Display for Address<Virtual> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let q4: u16 = ((self.value >> 48) & 0xffff) as u16;
        let q3: u16 = ((self.value >> 32) & 0xffff) as u16;
        let q2: u16 = ((self.value >> 16) & 0xffff) as u16;
        let q1: u16 = (self.value & 0xffff) as u16;

        write!(f, "0x")?;
        write!(f, "{:04x}_", q4)?;
        write!(f, "{:04x}_", q3)?;
        write!(f, "{:04x}_", q2)?;
        write!(f, "{:04x}", q1)
    }
}

/// Zero out an inclusive memory range.
///
/// # Safety
///
/// - `range.start` and `range.end` must be valid.
/// - `range.start` and `range.end` must be `T` aligned.
pub unsafe fn zero_volatile<T>(range: RangeInclusive<*mut T>)
where
    T: From<u8>,
{
    let mut ptr = *range.start();
    let end_inclusive = *range.end();

    while ptr <= end_inclusive {
        core::ptr::write_volatile(ptr, T::from(0));
        ptr = ptr.offset(1);
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Check `zero_volatile()`.
    #[kernel_test]
    fn zero_volatile_works() {
        let mut x: [usize; 3] = [10, 11, 12];
        let x_range = x.as_mut_ptr_range();
        let x_range_inclusive =
            RangeInclusive::new(x_range.start, unsafe { x_range.end.offset(-1) });

        unsafe { zero_volatile(x_range_inclusive) };

        assert_eq!(x, [0, 0, 0]);
    }

    /// Check `bss` section layout.
    #[kernel_test]
    fn bss_section_is_sane() {
        use crate::bsp::memory::bss_range_inclusive;
        use core::mem;

        let start = *bss_range_inclusive().start() as usize;
        let end = *bss_range_inclusive().end() as usize;

        assert_eq!(start % mem::size_of::<usize>(), 0);
        assert_eq!(end % mem::size_of::<usize>(), 0);
        assert!(end >= start);
    }
}

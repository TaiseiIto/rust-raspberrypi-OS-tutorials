// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management Unit types.

// 今回追加されたファイル

use crate::{
    bsp, common,
    memory::{Address, AddressType, Physical, Virtual},
};
use core::{convert::From, marker::PhantomData};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Generic page type.
/// ページを表す構造体
#[repr(C)]
pub struct Page<ATYPE: AddressType> {
    inner: [u8; bsp::memory::mmu::KernelGranule::SIZE],
    _address_type: PhantomData<ATYPE>,
}

/// Type describing a slice of pages.
/// ページの塊を表す構造体
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PageSliceDescriptor<ATYPE: AddressType> {
    start: Address<ATYPE>,
    num_pages: usize,
}

/// Architecture agnostic memory attributes.
/// メモリ属性を表す列挙体(Cacheable領域とDevice領域)
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum MemAttributes {
    CacheableDRAM,
    Device,
}

/// Architecture agnostic access permissions.
/// メモリ属性を表す列挙体(ReadOnlyとReadWrite)
#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub enum AccessPermissions {
    ReadOnly,
    ReadWrite,
}

/// Collection of memory attributes.
/// メモリ属性
#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub struct AttributeFields {
    // Cacheable領域かDevice領域か
    pub mem_attributes: MemAttributes,
    // ReadOnlyかReadWrite
    pub acc_perms: AccessPermissions,
    // 実行不可フラグ
    pub execute_never: bool,
}

/// An MMIO descriptor for use in device drivers.
/// MMIO領域を表す構造体
#[derive(Copy, Clone)]
pub struct MMIODescriptor {
    start_addr: Address<Physical>,
    size: usize,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// Page
//------------------------------------------------------------------------------

// Page構造体の実装
impl<ATYPE: AddressType> Page<ATYPE> {
    /// Get a pointer to the instance.
    pub const fn as_ptr(&self) -> *const Page<ATYPE> {
        // 自身へのポインタ
        self as *const _
    }
}

//------------------------------------------------------------------------------
// PageSliceDescriptor
//------------------------------------------------------------------------------

// Pageの塊を表すPageSliceDescriptor構造体の実装
impl<ATYPE: AddressType> PageSliceDescriptor<ATYPE> {
    /// Create an instance.
    pub const fn from_addr(start: Address<ATYPE>, num_pages: usize) -> Self {
        // 開始アドレスがページの境界になっていることを確認
        assert!(common::is_aligned(
            start.into_usize(),
            bsp::memory::mmu::KernelGranule::SIZE
        ));
        // 無ではないことを確認
        assert!(num_pages > 0);

        Self { start, num_pages }
    }

    /// Return a pointer to the first page of the described slice.
    const fn first_page_ptr(&self) -> *const Page<ATYPE> {
        // 自身の先頭ページへのポインタを返す
        self.start.into_usize() as *const _
    }

    /// Return the number of Pages the slice describes.
    pub const fn num_pages(&self) -> usize {
        // 自身のページ数を返す
        self.num_pages
    }

    /// Return the memory size this descriptor spans.
    pub const fn size(&self) -> usize {
        // 自身の大きさを返す
        self.num_pages * bsp::memory::mmu::KernelGranule::SIZE
    }

    /// Return the start address.
    pub const fn start_addr(&self) -> Address<ATYPE> {
        // 自身の先頭アドレスを返す
        self.start
    }

    /// Return the exclusive end address.
    pub fn end_addr(&self) -> Address<ATYPE> {
        // 自身の終了アドレス(自身に含まれる最後のアドレスの次のアドレス)を返す
        self.start + self.size()
    }

    /// Return the inclusive end address.
    pub fn end_addr_inclusive(&self) -> Address<ATYPE> {
        // 自身の終了アドレス(自身に含まれる最後のアドレス)を返す
        self.start + (self.size() - 1)
    }

    /// Check if an address is contained within this descriptor.
    pub fn contains(&self, addr: Address<ATYPE>) -> bool {
        // addrが自身の内部にあるかどうかの真理値
        (addr >= self.start_addr()) && (addr <= self.end_addr_inclusive())
    }

    /// Return a non-mutable slice of Pages.
    /// 変更不能なPageの塊を返す
    /// # Safety
    ///
    /// - Same as applies for `core::slice::from_raw_parts`.
    pub unsafe fn as_slice(&self) -> &[Page<ATYPE>] {
        core::slice::from_raw_parts(self.first_page_ptr(), self.num_pages)
    }
}

impl From<PageSliceDescriptor<Virtual>> for PageSliceDescriptor<Physical> {
    // 仮想addressのPageSliceから物理addressのPageSliceDescriptorを返す
    fn from(desc: PageSliceDescriptor<Virtual>) -> Self {
        Self {
            start: Address::new(desc.start.into_usize()),
            num_pages: desc.num_pages,
        }
    }
}

impl From<MMIODescriptor> for PageSliceDescriptor<Physical> {
    // MMIODescriptorから物理addressのPageSliceDescriptorを返す
    fn from(desc: MMIODescriptor) -> Self {
        // MMIO領域のページの開始物理address
        let start_page_addr = desc
            .start_addr
            .align_down(bsp::memory::mmu::KernelGranule::SIZE);

        // MMIO領域のPage数
        let len = ((desc.end_addr_inclusive().into_usize() - start_page_addr.into_usize())
            >> bsp::memory::mmu::KernelGranule::SHIFT)
            + 1;

        Self {
            start: start_page_addr,
            num_pages: len,
        }
    }
}

//------------------------------------------------------------------------------
// MMIODescriptor
//------------------------------------------------------------------------------

// MMIODescriptorの実装
impl MMIODescriptor {
    /// Create an instance.
    /// 開始物理addressと大きさからMMIODescriptorを作成
    pub const fn new(start_addr: Address<Physical>, size: usize) -> Self {
        assert!(size > 0);

        Self { start_addr, size }
    }

    /// Return the start address.
    /// MMIO領域の開始物理address
    pub const fn start_addr(&self) -> Address<Physical> {
        self.start_addr
    }

    /// Return the inclusive end address.
    /// MMIO領域内の一番最後の物理address
    pub fn end_addr_inclusive(&self) -> Address<Physical> {
        self.start_addr + (self.size - 1)
    }

    /// Return the size.
    /// MMIO領域の大きさ(bytes)
    pub const fn size(&self) -> usize {
        self.size
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Check if the size of `struct Page` is as expected.
    /// ページの大きさが正しいことを確認
    #[kernel_test]
    fn size_of_page_equals_granule_size() {
        assert_eq!(
            core::mem::size_of::<Page<Physical>>(),
            bsp::memory::mmu::KernelGranule::SIZE
        );
    }
}

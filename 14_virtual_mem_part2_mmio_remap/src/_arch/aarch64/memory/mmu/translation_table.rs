// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//! Architectural translation table.
//!
//! Only 64 KiB granule is supported.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::memory::mmu::translation_table::arch_translation_table

use crate::{
    bsp, memory,
    memory::{
        mmu::{
            arch_mmu::{Granule512MiB, Granule64KiB},
            AccessPermissions, AttributeFields, MemAttributes, Page, PageSliceDescriptor,
        },
        Address, Physical, Virtual,
    },
    // 新しいcrate memory::mmu::Page, memory::mmu::PageSliceDescriptor, memory::Address, memory::Physical, memory::Virtual
};
use core::convert;
use register::{register_bitfields, InMemoryRegister};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// A table descriptor, as per ARMv8-A Architecture Reference Manual Figure D5-15.
register_bitfields! {u64,
    STAGE1_TABLE_DESCRIPTOR [
        /// Physical address of the next descriptor.
        NEXT_LEVEL_TABLE_ADDR_64KiB OFFSET(16) NUMBITS(32) [], // [47:16]

        TYPE  OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

// A level 3 page descriptor, as per ARMv8-A Architecture Reference Manual Figure D5-17.
register_bitfields! {u64,
    STAGE1_PAGE_DESCRIPTOR [
        /// Unprivileged execute-never.
        UXN      OFFSET(54) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Privileged execute-never.
        PXN      OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Physical address of the next table descriptor (lvl2) or the page descriptor (lvl3).
        OUTPUT_ADDR_64KiB OFFSET(16) NUMBITS(32) [], // [47:16]

        /// Access flag.
        AF       OFFSET(10) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Shareability field.
        SH       OFFSET(8) NUMBITS(2) [
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],

        /// Access Permissions.
        AP       OFFSET(6) NUMBITS(2) [
            RW_EL1 = 0b00,
            RW_EL1_EL0 = 0b01,
            RO_EL1 = 0b10,
            RO_EL1_EL0 = 0b11
        ],

        /// Memory attributes index into the MAIR_EL1 register.
        AttrIndx OFFSET(2) NUMBITS(3) [],

        TYPE     OFFSET(1) NUMBITS(1) [
            Reserved_Invalid = 0,
            Page = 1
        ],

        VALID    OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

/// A table descriptor for 64 KiB aperture.
///
/// The output points to the next table.
#[derive(Copy, Clone)]
#[repr(C)]
struct TableDescriptor {
    value: u64,
}

/// A page descriptor with 64 KiB aperture.
///
/// The output points to physical memory.
#[derive(Copy, Clone)]
#[repr(C)]
struct PageDescriptor {
    value: u64,
}

trait StartAddr {
    // u64を返すphys_start_addr_u64とusizeを返すphys_start_addr_usizeの2つあったのをAddress<Physical>を返すやつに統合
    fn phys_start_addr(&self) -> Address<Physical>;
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Big monolithic struct for storing the translation tables. Individual levels must be 64 KiB
/// aligned, so the lvl3 is put first.
#[repr(C)]
#[repr(align(65536))]
pub struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
    /// Page descriptors, covering 64 KiB windows per entry.
    lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

    /// Table descriptors, covering 512 MiB windows.
    lvl2: [TableDescriptor; NUM_TABLES],

    /// Index of the next free MMIO page.
    /// 次の空きMMIO pageのindex
    cur_l3_mmio_index: usize,

    /// Have the tables been initialized?
    /// Tablesが初期化されているかどうか
    initialized: bool,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

// The binary is still identity mapped, so we don't need to convert here.
impl<T, const N: usize> StartAddr for [T; N] {
    // u64を返すphys_start_addr_u64とusizeを返すphys_start_addr_usizeの2つあったのをAddress<Physical>を返すやつに統合
    fn phys_start_addr(&self) -> Address<Physical> {
        Address::new(self as *const _ as usize)
    }
}

impl TableDescriptor {
    /// Create an instance.
    ///
    /// Descriptor is invalid by default.
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    /// Create an instance pointing to the supplied address.
    /// 引数をusize型で受け取るようにしていたのをAddress<Phycical>型に変更
    pub fn from_next_lvl_table_addr(phys_next_lvl_table_addr: Address<Physical>) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_TABLE_DESCRIPTOR::Register>::new(0);

        // into_usize()でAddress<Physical>をusizeに変更
        let shifted = phys_next_lvl_table_addr.into_usize() >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_64KiB.val(shifted as u64)
                + STAGE1_TABLE_DESCRIPTOR::TYPE::Table
                + STAGE1_TABLE_DESCRIPTOR::VALID::True,
        );

        TableDescriptor { value: val.get() }
    }
}

/// Convert the kernel's generic memory attributes to HW-specific attributes of the MMU.
impl convert::From<AttributeFields>
    for register::FieldValue<u64, STAGE1_PAGE_DESCRIPTOR::Register>
{
    fn from(attribute_fields: AttributeFields) -> Self {
        // Memory attributes.
        let mut desc = match attribute_fields.mem_attributes {
            MemAttributes::CacheableDRAM => {
                STAGE1_PAGE_DESCRIPTOR::SH::InnerShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(memory::mmu::arch_mmu::mair::NORMAL)
            }
            MemAttributes::Device => {
                STAGE1_PAGE_DESCRIPTOR::SH::OuterShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(memory::mmu::arch_mmu::mair::DEVICE)
            }
        };

        // Access Permissions.
        desc += match attribute_fields.acc_perms {
            AccessPermissions::ReadOnly => STAGE1_PAGE_DESCRIPTOR::AP::RO_EL1,
            AccessPermissions::ReadWrite => STAGE1_PAGE_DESCRIPTOR::AP::RW_EL1,
        };

        // The execute-never attribute is mapped to PXN in AArch64.
        desc += if attribute_fields.execute_never {
            STAGE1_PAGE_DESCRIPTOR::PXN::True
        } else {
            STAGE1_PAGE_DESCRIPTOR::PXN::False
        };

        // Always set unprivileged exectue-never as long as userspace is not implemented yet.
        desc += STAGE1_PAGE_DESCRIPTOR::UXN::True;

        desc
    }
}

impl PageDescriptor {
    /// Create an instance.
    ///
    /// Descriptor is invalid by default.
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    /// Create an instance.
    /// 引数phys_output_addrをusize型で受け取っていたのをPage<Physical>型に変更
    pub fn from_output_addr(
        phys_output_addr: *const Page<Physical>,
        attribute_fields: &AttributeFields,
    ) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(0);

        let shifted = phys_output_addr as u64 >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted)
                + STAGE1_PAGE_DESCRIPTOR::AF::True
                + STAGE1_PAGE_DESCRIPTOR::TYPE::Page
                + STAGE1_PAGE_DESCRIPTOR::VALID::True
                + (*attribute_fields).into(),
        );

        Self { value: val.get() }
    }

    /// Returns the valid bit.
    /// 今回追加された関数
    /// このpageが有効かどうか
    fn is_valid(&self) -> bool {
        InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(self.value)
            .is_set(STAGE1_PAGE_DESCRIPTOR::VALID)
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

// 今回追加されたimpl
// memory::mmu::AddressSpace<AS_SIZE>構造体にmemory::mmu::AssociatedTranslationTable traitを実装
impl<const AS_SIZE: usize> memory::mmu::AssociatedTranslationTable
    for memory::mmu::AddressSpace<AS_SIZE>
where
    [u8; Self::SIZE >> Granule512MiB::SHIFT]: Sized,
{
    type TableStartFromBottom = FixedSizeTranslationTable<{ Self::SIZE >> Granule512MiB::SHIFT }>;
}

impl<const NUM_TABLES: usize> FixedSizeTranslationTable<NUM_TABLES> {
    // Reserve the last 256 MiB of the address space for MMIO mappings.
    // MMIO領域のL2 tableとL3 tableにおけるindex
    // 8GiBの仮想address空間の最後の256MiBをMMIOとする
    const L2_MMIO_START_INDEX: usize = NUM_TABLES - 1;
    const L3_MMIO_START_INDEX: usize = 8192 / 2;

    /// Create an instance.
    #[allow(clippy::assertions_on_constants)]
    pub const fn new() -> Self {
        assert!(bsp::memory::mmu::KernelGranule::SIZE == Granule64KiB::SIZE);

        // Can't have a zero-sized address space.
        assert!(NUM_TABLES > 0);

        Self {
            lvl3: [[PageDescriptor::new_zeroed(); 8192]; NUM_TABLES],
            lvl2: [TableDescriptor::new_zeroed(); NUM_TABLES],
            cur_l3_mmio_index: 0, // 次の空きMMIO pageのindexは0
            initialized: false,   // 最初は初期化されていない
        }
    }

    /// The start address of the table's MMIO range.
    /// MMIO領域の始点となる仮想addressを返す
    #[inline(always)]
    fn mmio_start_addr(&self) -> Address<Virtual> {
        Address::new(
            (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)       // L2_MMIO_START_INDEXはL2 tableにおけるindex
                | (Self::L3_MMIO_START_INDEX << Granule64KiB::SHIFT), // L3_MMIO_START_INDEXはL3 tableにおけるindex
        )
    }

    /// The inclusive end address of the table's MMIO range.
    /// MMIO領域の終点となる仮想addressを返す
    #[inline(always)]
    fn mmio_end_addr_inclusive(&self) -> Address<Virtual> {
        Address::new(
            (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
                | (8191 << Granule64KiB::SHIFT) // 当該L3 tableの最後のindex
                | (Granule64KiB::SIZE - 1),     // 当該pageの最後のaddress
        )
    }

    /// Helper to calculate the lvl2 and lvl3 indices from an address.
    /// 仮想addressからlvl2, lvl3 tableにおけるindexを求める
    #[inline(always)]
    fn lvl2_lvl3_index_from(
        &self,
        addr: *const Page<Virtual>,
    ) -> Result<(usize, usize), &'static str> {
        let addr = addr as usize; // addrをusizeに変換
        let lvl2_index = addr >> Granule512MiB::SHIFT; // lvl2 tableにおけるindex
        let lvl3_index = (addr & Granule512MiB::MASK) >> Granule64KiB::SHIFT; // lvl3 tableにおけるindex

        if lvl2_index > (NUM_TABLES - 1) {
            return Err("Virtual page is out of bounds of translation table");
        }

        Ok((lvl2_index, lvl3_index))
    }

    /// Returns the PageDescriptor corresponding to the supplied Page.
    /// 引数addrで与えられたpageに対応するPageDescriptorを求める
    #[inline(always)]
    fn page_descriptor_from(
        &mut self,
        addr: *const Page<Virtual>,
    ) -> Result<&mut PageDescriptor, &'static str> {
        let (lvl2_index, lvl3_index) = self.lvl2_lvl3_index_from(addr)?; // 仮想addressからlvl2, lvl3のindexを求める

        Ok(&mut self.lvl3[lvl2_index][lvl3_index]) // 求めたindexからPageDescriptorを返す
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

// FixedSizeTranslationTable<NUM_TABLES>構造体に対してmemory::mmu::translation_table::interface::TranslationTableを実装
impl<const NUM_TABLES: usize> memory::mmu::translation_table::interface::TranslationTable
    for FixedSizeTranslationTable<NUM_TABLES>
{
    fn init(&mut self) {
        if self.initialized {
            // selfが既に初期化されていたら何もしない
            return;
        }

        // Populate the l2 entries.
        // lvl2 tableの初期化
        for (lvl2_nr, lvl2_entry) in self.lvl2.iter_mut().enumerate() {
            // 各lvl2_entryに対応するlvl3 tableの始点物理addressを入れていく
            let desc =
                TableDescriptor::from_next_lvl_table_addr(self.lvl3[lvl2_nr].phys_start_addr());
            *lvl2_entry = desc;
        }

        // 次の空きMMIO pageのindex
        self.cur_l3_mmio_index = Self::L3_MMIO_START_INDEX;
        // 初期化完了flagを立てる
        self.initialized = true;
    }

    // lvl2 tableの始点物理address
    fn phys_base_address(&self) -> Address<Physical> {
        self.lvl2.phys_start_addr()
    }

    // 仮想page sliceと物理page sliceを対応させる
    unsafe fn map_pages_at(
        &mut self,
        virt_pages: &PageSliceDescriptor<Virtual>,
        phys_pages: &PageSliceDescriptor<Physical>,
        attr: &AttributeFields,
    ) -> Result<(), &'static str> {
        // selfが初期化されていない時に警告
        assert!(self.initialized, "Translation tables not initialized");

        let p = phys_pages.as_slice();
        let v = virt_pages.as_slice();

        // No work to do for empty slices.
        if v.is_empty() {
            // virt_pagesが空の場合何もしない
            return Ok(());
        }

        if v.len() != p.len() {
            // 仮想pageと物理pageのsliceの要素数が違う場合全単射を作れないので諦める
            return Err("Tried to map page slices with unequal sizes");
        }

        if p.last().unwrap().as_ptr() >= bsp::memory::mmu::phys_addr_space_end_page() {
            // 物理memoryの大きさを超えた物理pageがslice内にある場合も諦める
            return Err("Tried to map outside of physical address space");
        }

        // 物理pageと仮想pageの組のイテレータを作ってforで順番に対応付けていく
        let iter = p.iter().zip(v.iter());
        for (phys_page, virt_page) in iter {
            // 仮想pageのpage_descriptor
            let page_descriptor = self.page_descriptor_from(virt_page.as_ptr())?;
            if page_descriptor.is_valid() {
                // 作成した仮想pageが既に物理pageに対応付けられている場合，諦める
                return Err("Virtual page is already mapped");
            }
            // 仮想pageのpage_descriptorに物理pageを紐づける
            *page_descriptor = PageDescriptor::from_output_addr(phys_page.as_ptr(), &attr);
        }

        Ok(())
    }

    // 新しいMMIO領域を割り当てる
    fn next_mmio_virt_page_slice(
        &mut self,
        num_pages: usize, // 要求page数
    ) -> Result<PageSliceDescriptor<Virtual>, &'static str> {
        // selfが初期化されていない時に警告
        assert!(self.initialized, "Translation tables not initialized");

        if num_pages == 0 {
            // 要求するpage数が0の場合何もしない
            return Err("num_pages == 0");
        }

        if (self.cur_l3_mmio_index + num_pages) > 8191 {
            // 残っているpage数が足りない場合諦める
            return Err("Not enough MMIO space left");
        }

        // 今回割り当てるMMIO領域の仮想address
        let addr = Address::new(
            (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
                | (self.cur_l3_mmio_index << Granule64KiB::SHIFT),
        );
        // 次の空きMMIO pageのindexを更新する
        self.cur_l3_mmio_index += num_pages;

        // 確保したMMIO領域の仮想page sliceを返す
        Ok(PageSliceDescriptor::from_addr(addr, num_pages))
    }

    // 引数で与えられた仮想page sliceにMMIO領域が含まれているかどうか
    fn is_virt_page_slice_mmio(&self, virt_pages: &PageSliceDescriptor<Virtual>) -> bool {
        // 引数で与えられた仮想page sliceの始めのpageと終わりのpage
        let start_addr = virt_pages.start_addr();
        let end_addr_inclusive = virt_pages.end_addr_inclusive();

        // slice内の仮想pageを順番に確認していく
        for i in [start_addr, end_addr_inclusive].iter() {
            if (*i >= self.mmio_start_addr()) && (*i <= self.mmio_end_addr_inclusive()) {
                // ひとつでもMMIO領域であればtrue
                return true;
            }
        }
        // MMIO領域が含まれていなければfalse
        false
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
pub type MinSizeTranslationTable = FixedSizeTranslationTable<1>;

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Check if the size of `struct TableDescriptor` is as expected.
    /// TableDescriptor構造体の大きさが期待通りであることを確認
    #[kernel_test]
    fn size_of_tabledescriptor_equals_64_bit() {
        assert_eq!(
            core::mem::size_of::<TableDescriptor>(), // TableDescriptorの大きさ
            core::mem::size_of::<u64>()              // TableDescriptorの大きさの期待値
        );
    }

    /// Check if the size of `struct PageDescriptor` is as expected.
    /// PageDescriptor構造体の大きさが期待通りであることを確認
    #[kernel_test]
    fn size_of_pagedescriptor_equals_64_bit() {
        assert_eq!(
            core::mem::size_of::<PageDescriptor>(), // PageDescriptorの大きさ
            core::mem::size_of::<u64>()             // PageDescriptorの大きさの期待値
        );
    }
}

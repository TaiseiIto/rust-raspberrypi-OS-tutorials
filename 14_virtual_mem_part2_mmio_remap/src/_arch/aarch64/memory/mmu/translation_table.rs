// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2022 Andre Richter <andre.o.richter@gmail.com>

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
    bsp,
    memory::{
        self,
        mmu::{
            arch_mmu::{Granule512MiB, Granule64KiB},
            AccessPermissions, AttributeFields, MemAttributes, MemoryRegion, PageAddress,
        },
        Address, Physical, Virtual,
    },
};
use core::convert;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
    registers::InMemoryRegister,
};

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
        let shifted = phys_next_lvl_table_addr.as_usize() >> Granule64KiB::SHIFT;
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
    for tock_registers::fields::FieldValue<u64, STAGE1_PAGE_DESCRIPTOR::Register>
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
    pub fn from_output_page_addr(
        phys_output_page_addr: PageAddress<Physical>,
        attribute_fields: &AttributeFields,
    ) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(0);

        let shifted = phys_output_page_addr.into_inner().as_usize() >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted as u64)
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
    /// Create an instance.
    #[allow(clippy::assertions_on_constants)]
    pub const fn new() -> Self {
        assert!(bsp::memory::mmu::KernelGranule::SIZE == Granule64KiB::SIZE);

        // Can't have a zero-sized address space.
        assert!(NUM_TABLES > 0);

        Self {
            lvl3: [[PageDescriptor::new_zeroed(); 8192]; NUM_TABLES],
            lvl2: [TableDescriptor::new_zeroed(); NUM_TABLES],
            initialized: false,
        }
    }

    /// Helper to calculate the lvl2 and lvl3 indices from an address.
    /// 仮想addressからlvl2, lvl3 tableにおけるindexを求める
    #[inline(always)]
    fn lvl2_lvl3_index_from_page_addr(
        &self,
        virt_page_addr: PageAddress<Virtual>,
    ) -> Result<(usize, usize), &'static str> {
        let addr = virt_page_addr.into_inner().as_usize();
        let lvl2_index = addr >> Granule512MiB::SHIFT;
        let lvl3_index = (addr & Granule512MiB::MASK) >> Granule64KiB::SHIFT;

        if lvl2_index > (NUM_TABLES - 1) {
            return Err("Virtual page is out of bounds of translation table");
        }

        Ok((lvl2_index, lvl3_index))
    }

    /// Returns the PageDescriptor corresponding to the supplied Page.
    /// 引数addrで与えられたpageに対応するPageDescriptorを求める
    /// Sets the PageDescriptor corresponding to the supplied page address.
    ///
    /// Doesn't allow overriding an already valid page.
    #[inline(always)]
    fn set_page_descriptor_from_page_addr(
        &mut self,
        virt_page_addr: PageAddress<Virtual>,
        new_desc: &PageDescriptor,
    ) -> Result<(), &'static str> {
        let (lvl2_index, lvl3_index) = self.lvl2_lvl3_index_from_page_addr(virt_page_addr)?;
        let desc = &mut self.lvl3[lvl2_index][lvl3_index];

        if desc.is_valid() {
            return Err("Virtual page is already mapped");
        }

        *desc = *new_desc;
        Ok(())
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

// FixedSizeTranslationTable<NUM_TABLES>構造体に対してmemory::mmu::translation_table::interface::
// TranslationTableを実装
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
            let phys_table_addr = self.lvl3[lvl2_nr].phys_start_addr();

            let new_desc = TableDescriptor::from_next_lvl_table_addr(phys_table_addr);
            *lvl2_entry = new_desc;
        }

        self.initialized = true;
    }

    // lvl2 tableの始点物理address
    fn phys_base_address(&self) -> Address<Physical> {
        self.lvl2.phys_start_addr()
    }

    // 仮想page sliceと物理page sliceを対応させる
    unsafe fn map_at(
        &mut self,
        virt_region: &MemoryRegion<Virtual>,
        phys_region: &MemoryRegion<Physical>,
        attr: &AttributeFields,
    ) -> Result<(), &'static str> {
        // selfが初期化されていない時に警告
        assert!(self.initialized, "Translation tables not initialized");

        if virt_region.size() != phys_region.size() {
            return Err("Tried to map memory regions with unequal sizes");
        }

        if phys_region.end_exclusive_page_addr() > bsp::memory::phys_addr_space_end_exclusive_addr()
        {
            return Err("Tried to map outside of physical address space");
        }

        let iter = phys_region.into_iter().zip(virt_region.into_iter());
        for (phys_page_addr, virt_page_addr) in iter {
            let new_desc = PageDescriptor::from_output_page_addr(phys_page_addr, attr);
            let virt_page = virt_page_addr;

            self.set_page_descriptor_from_page_addr(virt_page, &new_desc)?;
        }

        Ok(())
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

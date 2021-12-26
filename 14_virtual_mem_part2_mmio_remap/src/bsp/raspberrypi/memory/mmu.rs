// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Management Unit.

use crate::{
    common,
    memory::{
        mmu as generic_mmu,
        mmu::{
            AccessPermissions, AddressSpace, AssociatedTranslationTable, AttributeFields,
            MemAttributes, Page, PageSliceDescriptor, TranslationGranule,
        },
        Physical, Virtual,
    },
    synchronization::InitStateLock,
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// KernelTranslationTable型の定義
type KernelTranslationTable =
    <KernelVirtAddrSpace as AssociatedTranslationTable>::TableStartFromBottom;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The translation granule chosen by this BSP. This will be used everywhere else in the kernel to
/// derive respective data structures and their sizes. For example, the `crate::memory::mmu::Page`.
/// BSPに応じて決まるpagingの粒度で，`crate::memory::mmu::Page`などといったそれぞれのdata構造とその大きさを得るためにkernel内の他の全ての場所で使われます．
pub type KernelGranule = TranslationGranule<{ 64 * 1024 }>;

/// The kernel's virtual address space defined by this BSP.
/// このBSPで定義されるkernelの仮想address空間(8GiB)
pub type KernelVirtAddrSpace = AddressSpace<{ 8 * 1024 * 1024 * 1024 }>;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

/// The kernel translation tables.
///
/// It is mandatory that InitStateLock is transparent.
/// That is, `size_of(InitStateLock<KernelTranslationTable>) == size_of(KernelTranslationTable)`.
/// There is a unit tests that checks this porperty.
/// InitStateLockは透過性がなければならない
/// つまり，`size_of(InitStateLock<KernelTranslationTable>) == size_of(KernelTranslationTable)`だ
/// これを確認する単体testがある
static KERNEL_TABLES: InitStateLock<KernelTranslationTable> =
    InitStateLock::new(KernelTranslationTable::new());

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// Helper function for calculating the number of pages the given parameter spans.
/// 与えられたsizeをpageに分割したときのpage数を計算する関数
const fn size_to_num_pages(size: usize) -> usize {
    // sizeは正でなければならない
    assert!(size > 0);
    // sizeはKernelGranule::SIZEの倍数でなければならない
    assert!(size % KernelGranule::SIZE == 0);

    // sizeをpageに分割したときのpage数
    size >> KernelGranule::SHIFT
}

/// The Read+Execute (RX) pages of the kernel binary.
/// kernelの読み実行可能pagesの仮想PageSliceDescriptorを取得する関数
fn virt_rx_page_desc() -> PageSliceDescriptor<Virtual> {
    // RX領域の大きさからそのpage数を取得
    let num_pages = size_to_num_pages(super::rx_size());

    // kernelのRX領域の先頭とpage数から仮想PageSliceDescriptorを返す
    PageSliceDescriptor::from_addr(super::virt_rx_start(), num_pages)
}

/// The Read+Write (RW) pages of the kernel binary.
/// kernelの読書可能pagesの仮想PageSliceDescriptorを取得する関数
fn virt_rw_page_desc() -> PageSliceDescriptor<Virtual> {
    // RW領域の大きさからそのpage数を取得
    let num_pages = size_to_num_pages(super::rw_size());

    // kernelのRW領域の先頭とpage数から仮想PageSliceDescriptorを返す
    PageSliceDescriptor::from_addr(super::virt_rw_start(), num_pages)
}

/// The boot core's stack.
/// kernelのstackの仮想PageSliceDescriptorを取得する関数
fn virt_boot_core_stack_page_desc() -> PageSliceDescriptor<Virtual> {
    // stack領域の大きさからそのpage数を取得
    let num_pages = size_to_num_pages(super::boot_core_stack_size());

    // kernelのstack領域の先頭とpage数から仮想PageSliceDescriptorを返す
    PageSliceDescriptor::from_addr(super::virt_boot_core_stack_start(), num_pages)
}

// The binary is still identity mapped, so we don't need to convert in the following.
// kernel部分のｍemoryはまだ恒等写像なので，仮想addressから物理addressに変換する必要なし

/// The Read+Execute (RX) pages of the kernel binary.
/// kernelの読書可能pagesの物理PageSliceDescriptorを取得する関数
fn phys_rx_page_desc() -> PageSliceDescriptor<Physical> {
    virt_rx_page_desc().into()
}

/// The Read+Write (RW) pages of the kernel binary.
/// kernelの読書可能pagesの物理PageSliceDescriptorを取得する関数
fn phys_rw_page_desc() -> PageSliceDescriptor<Physical> {
    virt_rw_page_desc().into()
}

/// The boot core's stack.
/// kernelのstackの物理PageSliceDescriptorを取得する関数
fn phys_boot_core_stack_page_desc() -> PageSliceDescriptor<Physical> {
    virt_boot_core_stack_page_desc().into()
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the kernel's translation tables.
/// kernelのtranslation tablesの参照を取得する関数
pub fn kernel_translation_tables() -> &'static InitStateLock<KernelTranslationTable> {
    // kernelのtranslation tablesの参照を返す
    &KERNEL_TABLES
}

/// The boot core's stack guard page.
/// boot coreのstack guard pageのPageSliceDescriptorを取得する関数
pub fn virt_boot_core_stack_guard_page_desc() -> PageSliceDescriptor<Virtual> {
    // boot coreのstack guard領域の大きさからそのpage数を取得
    let num_pages = size_to_num_pages(super::boot_core_stack_guard_page_size());
    // boot coreのstack guard領域の先頭とpage数からそのPageSliceDescriptorを返す
    PageSliceDescriptor::from_addr(super::virt_boot_core_stack_guard_page_start(), num_pages)
}

/// Pointer to the last page of the physical address space.
/// 物理address空間の最後のpageを返す関数
pub fn phys_addr_space_end_page() -> *const Page<Physical> {
    common::align_down(
        // 物理address空間の最後をusizeにして渡す
        super::phys_addr_space_end().into_usize(),
        // KernelのPage粒度
        KernelGranule::SIZE,
    ) as *const Page<_>
}

/// Map the kernel binary.
/// kernel領域をmapする
/// # Safety
///
/// - Any miscalculation or attribute error will likely be fatal. Needs careful manual checking.
pub unsafe fn kernel_map_binary() -> Result<(), &'static str> {
    // kernel codeとRead Only dataをmap
    generic_mmu::kernel_map_pages_at(
        "Kernel code and RO data",
        // 仮想PageDescriptor
        &virt_rx_page_desc(),
        // 物理PageDescriptor
        &phys_rx_page_desc(),
        // 領域のmemory属性を指定
        &AttributeFields {
            // Cachable
            mem_attributes: MemAttributes::CacheableDRAM,
            // Read Only
            acc_perms: AccessPermissions::ReadOnly,
            // 実行可能
            execute_never: false,
        },
    )?;

    // kernelのdata領域とbss領域をmap
    generic_mmu::kernel_map_pages_at(
        "Kernel data and bss",
        // 仮想PageDescriptor
        &virt_rw_page_desc(),
        // 物理PageDescriptor
        &phys_rw_page_desc(),
        &AttributeFields {
            // Cachable
            mem_attributes: MemAttributes::CacheableDRAM,
            // Read & Write
            acc_perms: AccessPermissions::ReadWrite,
            // 実行不可
            execute_never: true,
        },
    )?;

    // kernelのboot core stack領域をmap
    generic_mmu::kernel_map_pages_at(
        "Kernel boot-core stack",
        // 仮想PageDescriptor
        &virt_boot_core_stack_page_desc(),
        // 物理PageDescriptor
        &phys_boot_core_stack_page_desc(),
        &AttributeFields {
            // Cachable
            mem_attributes: MemAttributes::CacheableDRAM,
            // Read & Write
            acc_perms: AccessPermissions::ReadWrite,
            // 実行不可
            execute_never: true,
        },
    )?;

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Check alignment of the kernel's virtual memory layout sections.
    /// kernelの仮想memory領域が64KiB alignedであることを確認
    #[kernel_test]
    fn virt_mem_layout_sections_are_64KiB_aligned() {
        // code領域，data，bss領域，stack領域それぞれについて
        for i in [
            virt_rx_page_desc,
            virt_rw_page_desc,
            virt_boot_core_stack_page_desc,
        ]
        .iter()
        {
            // 先頭addressと末尾addressを取得
            let start: usize = i().start_addr().into_usize();
            let end: usize = i().end_addr().into_usize();

            // 先頭addressと末尾addressがそれぞれ64KiB alignedで，startの後にendが来ることを確認
            assert_eq!(start % KernelGranule::SIZE, 0);
            assert_eq!(end % KernelGranule::SIZE, 0);
            assert!(end >= start);
        }
    }

    /// Ensure the kernel's virtual memory layout is free of overlaps.
    /// kernelの仮想memory layoutに，互いに重なり合っている部分がないことを確認
    #[kernel_test]
    fn virt_mem_layout_has_no_overlaps() {
        // code領域，data，bss領域，stack領域それぞれの組について
        let layout = [
            virt_rx_page_desc(),
            virt_rw_page_desc(),
            virt_boot_core_stack_page_desc(),
        ];

        for (i, first_range) in layout.iter().enumerate() {
            for second_range in layout.iter().skip(i + 1) {
                // 2つの領域の組に重なっている部分がないことを確認
                assert!(!first_range.contains(second_range.start_addr()));
                assert!(!first_range.contains(second_range.end_addr_inclusive()));
                assert!(!second_range.contains(first_range.start_addr()));
                assert!(!second_range.contains(first_range.end_addr_inclusive()));
            }
        }
    }

    /// Check if KERNEL_TABLES is in .bss.
    /// KERNEL_TABLESが.bssに配置されていることを確認
    #[kernel_test]
    fn kernel_tables_in_bss() {
        // bss領域を取得
        let bss_range = super::super::bss_range_inclusive();
        // kernel tablesのaddressを取得
        let kernel_tables_addr = &KERNEL_TABLES as *const _ as usize as *mut u64;

        // kernel tablesのaddressが.bss領域内にあることを確認
        assert!(bss_range.contains(&kernel_tables_addr));
    }
}

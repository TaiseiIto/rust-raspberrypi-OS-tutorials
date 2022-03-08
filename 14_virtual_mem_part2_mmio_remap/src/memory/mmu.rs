// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management Unit.

#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/memory/mmu.rs"]
mod arch_mmu;

mod alloc;
mod mapping_record;
mod translation_table;
mod types;

use crate::{
    bsp,
    memory::{Address, Physical, Virtual},
    synchronization, warn,
};
use core::{fmt, num::NonZeroUsize};

pub use types::*;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// MMU enable errors variants.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum MMUEnableError {
    AlreadyEnabled,
    Other(&'static str),
}

/// Memory Management interfaces.
pub mod interface {
    use super::*;

    /// MMU functions.
    /// MMUが実装すべき機能
    pub trait MMU {
        /// Turns on the MMU for the first time and enables data and instruction caching.
        /// MMUを起動しデータと命令のキャッシュを有効にする
        /// # Safety
        ///
        /// - Changes the HW's global state.
        unsafe fn enable_mmu_and_caching(
            &self,
            phys_tables_base_addr: Address<Physical>, // 今回追加された引数
        ) -> Result<(), MMUEnableError>;

        /// Returns true if the MMU is enabled, false otherwise.
        /// MMUが起動しているかどうかの真理値
        fn is_enabled(&self) -> bool;
    }
}

/// Describes the characteristics of a translation granule.
pub struct TranslationGranule<const GRANULE_SIZE: usize>;

/// Describes properties of an address space.
pub struct AddressSpace<const AS_SIZE: usize>;

/// Intended to be implemented for [`AddressSpace`].
/// 今回追加された未実装のtrait
/// AddressSpace構造体に実装予定
pub trait AssociatedTranslationTable {
    /// A translation table whose address range is:
    ///
    /// [AS_SIZE - 1, 0]
    type TableStartFromBottom;
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------
use interface::MMU;
use synchronization::interface::*;
use translation_table::interface::TranslationTable;

/// kernelのtranslation tableにpagesをmapする
/// Query the BSP for the reserved virtual addresses for MMIO remapping and initialize the kernel's
/// MMIO VA allocator with it.
fn kernel_init_mmio_va_allocator() {
    let region = bsp::memory::mmu::virt_mmio_remap_region();

    alloc::kernel_mmio_va_allocator().lock(|allocator| allocator.initialize(region));
}

/// Map a region in the kernel's translation tables.
///
/// No input checks done, input is passed through to the architectural implementation.
/// MMIO領域のmappingにも使用するため，
/// 与えられた引数がMMIO領域でないことはこの関数の呼び出し元が保証する必要がある # Safety
///
/// - See `map_at()`.
/// - Does not prevent aliasing.
unsafe fn kernel_map_at_unchecked(
    name: &'static str,
    virt_region: &MemoryRegion<Virtual>,
    phys_region: &MemoryRegion<Physical>,
    attr: &AttributeFields,
) -> Result<(), &'static str> {
    // kernelのtranslation tableに新たな仮想pageを新たな物理pageに対応付ける
    bsp::memory::mmu::kernel_translation_tables()
        .write(|tables| tables.map_at(virt_region, phys_region, attr))?;

    // mapping内容を記録し，エラーが返ってきたらエラーメッセージを表示
    if let Err(x) = mapping_record::kernel_add(name, virt_region, phys_region, attr) {
        warn!("{}", x);
    }

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl fmt::Display for MMUEnableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MMUEnableError::AlreadyEnabled => write!(f, "MMU is already enabled"),
            MMUEnableError::Other(x) => write!(f, "{}", x),
        }
    }
}

impl<const GRANULE_SIZE: usize> TranslationGranule<GRANULE_SIZE> {
    /// The granule's size.
    pub const SIZE: usize = Self::size_checked();

    /// The granule's mask.
    /// AND演算でgranule内での相対位置を取り出すためのマスク
    pub const MASK: usize = Self::SIZE - 1;

    /// The granule's shift, aka log2(size).
    pub const SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(GRANULE_SIZE.is_power_of_two());

        GRANULE_SIZE
    }
}

impl<const AS_SIZE: usize> AddressSpace<AS_SIZE> {
    /// The address space size.
    pub const SIZE: usize = Self::size_checked();

    /// The address space shift, aka log2(size).
    pub const SIZE_SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(AS_SIZE.is_power_of_two());

        // Check for architectural restrictions as well.
        Self::arch_address_space_size_sanity_checks();

        AS_SIZE
    }
}

/// Raw mapping of a virtual to physical region in the kernel translation tables.
/// kernel translation tablesで仮想pageを物理pageに対応付ける
///
/// Prevents mapping into the MMIO range of the tables.
/// MMIO領域のmappingはエラーを返して防止する
/// # Safety
///
/// - See `kernel_map_at_unchecked()`.
/// - Does not prevent aliasing. Currently, the callers must be trusted.
pub unsafe fn kernel_map_at(
    name: &'static str,
    virt_region: &MemoryRegion<Virtual>,
    phys_region: &MemoryRegion<Physical>,
    attr: &AttributeFields,
) -> Result<(), &'static str> {
    // 引数で与えられた仮想pageがMMIO領域でないことを確認
    if bsp::memory::mmu::virt_mmio_remap_region().overlaps(virt_region) {
        return Err("Attempt to manually map into MMIO region");
    }

    // 仮想pageを物理pageにメモリ属性を指定して対応付ける
    kernel_map_at_unchecked(name, virt_region, phys_region, attr)?;

    Ok(())
}

/// MMIO remapping in the kernel translation tables.
/// kernel translation tablesでMMIO領域をmapする
/// Typically used by device drivers.
/// この関数はDevice driverによって使用される
/// # Safety
///
/// - Same as `kernel_map_at_unchecked()`, minus the aliasing part.
pub unsafe fn kernel_map_mmio(
    name: &'static str,
    mmio_descriptor: &MMIODescriptor,
) -> Result<Address<Virtual>, &'static str> {
    // MMIO領域の物理pages
    let phys_region = MemoryRegion::from(*mmio_descriptor);
    // MMIO領域のページ内における相対開始address
    let offset_into_start_page = mmio_descriptor.start_addr().offset_into_page();

    // Check if an identical region has been mapped for another driver. If so, reuse it.
    // mapping要求されたMMIO領域の物理pagesがすでに別のdriverにmapされている場合，それを再利用する
    let virt_addr = if let Some(addr) =
        mapping_record::kernel_find_and_insert_mmio_duplicate(mmio_descriptor, name)
    {
        // 当該MMIO領域の仮想addressを返す
        addr
    // Otherwise, allocate a new region and map it.
    // そうでない場合，新しくMMIO領域をmappingする
    } else {
        // 未使用の仮想pagesを探す
        let num_pages = match NonZeroUsize::new(phys_region.num_pages()) {
            None => return Err("Requested 0 pages"),
            Some(x) => x,
        };

        let virt_region =
            alloc::kernel_mmio_va_allocator().lock(|allocator| allocator.alloc(num_pages))?;

        // 新しい仮想pagesを割り当てる
        kernel_map_at_unchecked(
            name,
            &virt_region,
            &phys_region,
            &AttributeFields {
                mem_attributes: MemAttributes::Device,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
        )?;

        virt_region.start_addr()
    };

    // MMIO領域の開始仮想address
    Ok(virt_addr + offset_into_start_page)
}

/// Map the kernel's binary. Returns the translation table's base address.
/// kernel領域をmapし，kernel translation tableの開始物理addressを返す
/// # Safety
///
/// - See [`bsp::memory::mmu::kernel_map_binary()`].
pub unsafe fn kernel_map_binary() -> Result<Address<Physical>, &'static str> {
    // kernel's translation tableの開始物理address
    let phys_kernel_tables_base_addr =
        bsp::memory::mmu::kernel_translation_tables().write(|tables| {
            tables.init();
            tables.phys_base_address()
        });
    // kernel領域をmapする
    bsp::memory::mmu::kernel_map_binary()?;

    // kernel's translation tableの開始物理addressを返す
    Ok(phys_kernel_tables_base_addr)
}

/// Enable the MMU and data + instruction caching.
/// MMUを起動し，dataと命令のキャッシュを有効にする
/// # Safety
///
/// - Crucial function during kernel init. Changes the the complete memory view of the processor.
/// - kernel初期化中の重要な関数．Processorの全てのmemory構造を書き換える
pub unsafe fn enable_mmu_and_caching(
    phys_tables_base_addr: Address<Physical>,
) -> Result<(), MMUEnableError> {
    // アーキテクチャ固有のMMU起動処理を呼び出す
    arch_mmu::mmu().enable_mmu_and_caching(phys_tables_base_addr)
}

/// Finish initialization of the MMU subsystem.
pub fn post_enable_init() {
    kernel_init_mmio_va_allocator();
}

/// Human-readable print of all recorded kernel mappings.
/// kernel mappingsを読みやすいように表示する
pub fn kernel_print_mappings() {
    mapping_record::kernel_print()
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::mmu::{AccessPermissions, MemAttributes, PageAddress};
    use test_macros::kernel_test;

    /// Check that you cannot map into the MMIO VA range from kernel_map_at().
    #[kernel_test]
    fn no_manual_mmio_map() {
        let phys_start_page_addr: PageAddress<Physical> = PageAddress::from(0);
        let phys_end_exclusive_page_addr: PageAddress<Physical> =
            phys_start_page_addr.checked_offset(5).unwrap();
        let phys_region = MemoryRegion::new(phys_start_page_addr, phys_end_exclusive_page_addr);

        let num_pages = NonZeroUsize::new(phys_region.num_pages()).unwrap();
        let virt_region = alloc::kernel_mmio_va_allocator()
            .lock(|allocator| allocator.alloc(num_pages))
            .unwrap();

        let attr = AttributeFields {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        };

        unsafe {
            assert_eq!(
                kernel_map_at("test", &virt_region, &phys_region, &attr),
                Err("Attempt to manually map into MMIO region")
            )
        };
    }
}

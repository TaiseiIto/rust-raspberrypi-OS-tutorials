// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2022 Andre Richter <andre.o.richter@gmail.com>

//! Translation table.

// 今回追加されたファイル

#[cfg(target_arch = "aarch64")]
#[path = "../../_arch/aarch64/memory/mmu/translation_table.rs"]
mod arch_translation_table;

use super::{AttributeFields, MemoryRegion};
use crate::memory::{Address, Physical, Virtual};

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
#[cfg(target_arch = "aarch64")]
pub use arch_translation_table::FixedSizeTranslationTable;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Translation table interfaces.
pub mod interface {
    use super::*;

    /// Translation table operations.
    /// Translation tableが備えるべき操作
    pub trait TranslationTable {
        /// Anything that needs to run before any of the other provided functions can be used.
        /// 他の関数を使う前に実行しておくべきことをこの関数に記述する
        /// # Safety
        ///
        /// - Implementor must ensure that this function can run only once or is harmless if invoked
        ///   multiple times.
        /// - このtraitの実装はこの関数が一回のみ実行されること．
        ///   そうでなくても複数回実行が無害であることを保証しなければならない．
        fn init(&mut self);

        /// The translation table's base address to be used for programming the MMU.
        /// MMUをprogramするためのtranslation tableの物理base addressを返すmethod
        fn phys_base_address(&self) -> Address<Physical>;

        /// Map the given virtual pages to the given physical pages.
        /// Map the given virtual memory region to the given physical memory region.
        /// 与えられた仮想pageを与えられた物理pageにmappingする.
        ///
        /// # Safety
        ///
        /// - Using wrong attributes can cause multiple issues of different nature in the system.
        /// - It is not required that the architectural implementation prevents aliasing. That is,
        ///   mapping to the same physical memory using multiple virtual addresses, which would
        ///   break Rust's ownership assumptions. This should be protected against in the kernel's
        ///   generic MMU code.
        /// - 間違った属性を使うとシステムごとにいろいろと問題が発生する．
        /// - アーキテクチャ固有の実装が，Rustの所有権規定を壊すことに繋がる複数の仮想addressを同じ物理memoryへmappingするaliasing防止を実装する必要はない．
        ///   この問題はkernelのgeneric MMU codeにおいて保護されるべきである．
        unsafe fn map_at(
            &mut self,
            virt_region: &MemoryRegion<Virtual>,
            phys_region: &MemoryRegion<Physical>,
            attr: &AttributeFields,
        ) -> Result<(), &'static str>;
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::mmu::{AccessPermissions, MemAttributes, PageAddress};
    use arch_translation_table::MinSizeTranslationTable;
    use interface::TranslationTable;
    use test_macros::kernel_test;

    /// Sanity checks for the TranslationTable implementation.
    /// TranslationTable traitの実装が正常であることを確認するテスト
    #[kernel_test]
    fn translationtable_implementation_sanity() {
        // This will occupy a lot of space on the stack.
        // MinSizeTranslationTableが_arch/aarch64/memory/mmu/translation_table.
        // rsの中でTranslationTable traitを実装している．
        let mut tables = MinSizeTranslationTable::new();

        // 初期化
        tables.init();

        let virt_start_page_addr: PageAddress<Virtual> = PageAddress::from(0);
        let virt_end_exclusive_page_addr: PageAddress<Virtual> =
            virt_start_page_addr.checked_offset(5).unwrap();

        let phys_start_page_addr: PageAddress<Physical> = PageAddress::from(0);
        let phys_end_exclusive_page_addr: PageAddress<Physical> =
            phys_start_page_addr.checked_offset(5).unwrap();

        let virt_region = MemoryRegion::new(virt_start_page_addr, virt_end_exclusive_page_addr);
        let phys_region = MemoryRegion::new(phys_start_page_addr, phys_end_exclusive_page_addr);

        let attr = AttributeFields {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        };

        unsafe { assert_eq!(tables.map_at(&virt_region, &phys_region, &attr), Ok(())) };
    }
}

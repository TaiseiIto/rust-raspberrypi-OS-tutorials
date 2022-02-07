// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//! Translation table.

// 今回追加されたファイル

#[cfg(target_arch = "aarch64")]
#[path = "../../_arch/aarch64/memory/mmu/translation_table.rs"]
mod arch_translation_table;

use crate::memory::{
    mmu::{AttributeFields, PageSliceDescriptor},
    Address, Physical, Virtual,
};

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
        /// - このtraitの実装はこの関数が一回のみ実行されること．そうでなくても複数回実行が無害であることを保証しなければならない．
        fn init(&mut self);

        /// The translation table's base address to be used for programming the MMU.
        /// MMUをprogramするためのtranslation tableの物理base addressを返すmethod
        fn phys_base_address(&self) -> Address<Physical>;

        /// Map the given virtual pages to the given physical pages.
        /// 与えられた仮想pageを与えられた物理pageにmappingする.
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
        unsafe fn map_pages_at(
            &mut self,
            virt_pages: &PageSliceDescriptor<Virtual>,
            phys_pages: &PageSliceDescriptor<Physical>,
            attr: &AttributeFields,
        ) -> Result<(), &'static str>;

        /// Obtain a free virtual page slice in the MMIO region.
        /// MMIO領域から未使用の仮想page sliceを入手する．
        /// The "MMIO region" is a distinct region of the implementor's choice, which allows
        /// differentiating MMIO addresses from others. This can speed up debugging efforts.
        /// Ideally, those MMIO addresses are also standing out visually so that a human eye can
        /// identify them. For example, by allocating them from near the end of the virtual address
        /// space.
        /// MMIO addressesを他から識別するためのMMIO領域は実装者の選択によって異なる．
        /// これはdebug速度を上げる．
        /// 理想的には，これらのMMIO addressesは目立っていて，人の目でそれを識別できるとよい．
        /// 例えば，これらを仮想address空間の最後に近い領域を割り当てるとか
        fn next_mmio_virt_page_slice(
            &mut self,
            num_pages: usize,
        ) -> Result<PageSliceDescriptor<Virtual>, &'static str>;

        /// Check if a virtual page splice is in the "MMIO region".
        /// 与えられた仮想page sliceがMMIO領域にあるかどうか
        fn is_virt_page_slice_mmio(&self, virt_pages: &PageSliceDescriptor<Virtual>) -> bool;
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bsp;
    use arch_translation_table::MinSizeTranslationTable; // これが_arch/aarch64/memory/mmu/translation_table.rsの中でTranslationTable traitを実装している．
    use interface::TranslationTable;
    use test_macros::kernel_test;

    /// Sanity checks for the TranslationTable implementation.
    /// TranslationTable traitの実装が正常であることを確認するテスト
    #[kernel_test]
    fn translationtable_implementation_sanity() {
        // This will occupy a lot of space on the stack.
        // MinSizeTranslationTableが_arch/aarch64/memory/mmu/translation_table.rsの中でTranslationTable traitを実装している．
        let mut tables = MinSizeTranslationTable::new();

        // 初期化
        tables.init();

        // 与えられたMMIO領域から未使用の仮想page sliceを入手
        let x = tables.next_mmio_virt_page_slice(0);
        // 仮想page sliceを入手できたことを確認
        assert!(x.is_err());

        let x = tables.next_mmio_virt_page_slice(1_0000_0000);
        // 仮想page sliceを入手できたことを確認
        assert!(x.is_err());

        let x = tables.next_mmio_virt_page_slice(2).unwrap();
        // 仮想page sliceの大きさを確認
        assert_eq!(x.size(), bsp::memory::mmu::KernelGranule::SIZE * 2);

        // 入手した仮想page sliceがMMIO領域にあることを確認
        assert_eq!(tables.is_virt_page_slice_mmio(&x), true);

        // 先頭addressがMMIO領域でないことを確認
        assert_eq!(
            tables.is_virt_page_slice_mmio(&PageSliceDescriptor::from_addr(Address::new(0), 1)),
            false
        );
    }
}

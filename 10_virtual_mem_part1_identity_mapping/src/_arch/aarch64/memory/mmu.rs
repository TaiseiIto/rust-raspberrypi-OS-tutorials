// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management Unit Driver.
//!
//! Only 64 KiB granule is supported.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::memory::mmu::arch_mmu

use crate::{
    bsp, memory,
    memory::mmu::{translation_table::KernelTranslationTable, TranslationGranule},
};
use core::intrinsics::unlikely;
use cortex_a::{barrier, regs::*};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// Memory Management Unit type.
struct MemoryManagementUnit;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub type Granule512MiB = TranslationGranule<{ 512 * 1024 * 1024 }>;
pub type Granule64KiB = TranslationGranule<{ 64 * 1024 }>;

/// Constants for indexing the MAIR_EL1.
#[allow(dead_code)]
pub mod mair {
    pub const DEVICE: u64 = 0;
    pub const NORMAL: u64 = 1;
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

/// The kernel translation tables.
///
/// # Safety
///
/// - Supposed to land in `.bss`. Therefore, ensure that all initial member values boil down to "0".
static mut KERNEL_TABLES: KernelTranslationTable = KernelTranslationTable::new();

static MMU: MemoryManagementUnit = MemoryManagementUnit;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl<const AS_SIZE: usize> memory::mmu::AddressSpace<AS_SIZE> {
    /// Checks for architectural restrictions.
    pub const fn arch_address_space_size_sanity_checks() {
        // Size must be at least one full 512 MiB table.
        assert!((AS_SIZE % Granule512MiB::SIZE) == 0);

        // Check for 48 bit virtual address size as maximum, which is supported by any ARMv8
        // version.
        assert!(AS_SIZE <= (1 << 48));
    }
}

impl MemoryManagementUnit {
    /// Setup function for the MAIR_EL1 register.
    fn set_up_mair(&self) {
        // Define the memory types being mapped.
        // MAIR_EL1レジスタはAttr0~Attr7までそれぞれ8ビット計64ビットからなり，8種類のメモリ属性を定義できる
        // https://developer.arm.com/documentation/ddi0595/2021-06/AArch64-Registers/MAIR-EL1--Memory-Attribute-Indirection-Register--EL1-?lang=en
        MAIR_EL1.write(
            // Attribute 1 - Cacheable normal DRAM.
            // 通常のメモリ領域に対して与えられる属性
            // 0b11110000
            // write backは，キャッシュへの書き込みが終わったら次の処理に進み，空き時間にメモリに書き込む方式っぽい
            // それに対してwrite throughは，キャッシュへ書き込むのと同時にメモリにも書き込む方式っぽい
            // transient属性は，キャッシュに保存された情報が割とすぐにメモリに書き込まれる方式っぽい(以下を参照)
            // https://developer.arm.com/documentation/ddi0406/c/Application-Level-Architecture/Application-Level-Memory-Model/Memory-types-and-attributes-and-the-memory-order-model/Normal-memory?lang=en#CHDHGEDG
            MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc + 
            // 0b00001111
        MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +

        // Attribute 0 - Device.
        // Deviceにmapされているメモリ領域に対して与えられる属性
        // 0b00000100
        // 以下を参照
        // https://developer.arm.com/documentation/den0024/a/Memory-Ordering/Memory-types/Device-memory
        // Gathering属性を有効にすると，連続するアドレスへの書き込みを一気にやってくれるらしい．例えば連続する2バイトに順番に書き込むコードが実行されるとひとつのhalf-word書き込みになるとか．
        // Reordering属性を有効にすると，同じデバイスへの連続したアクセスが，順番を変えて実行されるらしい
        // EarlyWriteAck属性を有効にすると，デバイスへの書き込みに対するAckが，デバイスからではなくそれ以前の中間バッファから送られる．人体でいう反射みたいな感じか．
        MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
    }

    /// Configure various settings of stage 1 of the EL1 translation regime.
    /// キャッシュの動作に関するTCR_EL1(Translation Control Register)レジスタの設定
    fn configure_translation_control(&self) {
        // メモリ空間の大きさの対数
        let t0sz = (64 - bsp::memory::mmu::KernelAddrSpace::SIZE_SHIFT) as u64;
        // https://developer.arm.com/documentation/ddi0595/2021-06/AArch64-Registers/TCR-EL1--Translation-Control-Register--EL1-
        TCR_EL1.write(
            TCR_EL1::TBI0::Used
                + TCR_EL1::IPS::Bits_40 // Intermediate Physical address Size = 1TiB
                + TCR_EL1::TG0::KiB_64 // Granule size for the TTBR0_EL1 = 64KiB
                + TCR_EL1::SH0::Inner // Shareability attribute Inner Shareableについてはhttps://developer.arm.com/documentation/den0024/a/Memory-Ordering/Memory-attributes/Cacheable-and-shareable-memory-attributes
                // Inner cache とはL1 cacheのこと
                // Outer cache とはL2 cacheのこと
                + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Outer cacheability attribute 後で書き込む．
                // Write Alloc は，write miss (書き込みを行いたい領域がキャッシュ上になかった場合)における処理のひとつで，その領域をキャッシュに読み込んでからキャッシュに書き込む
                // No Write Allocでは，書き込みを行いたい領域をキャッシュに読み込まず，直接メモリに書き込む
                // Read Allocも同様に，読み込みたい領域がキャッシュ上になかった場合，その領域をキャッシュに読み込んでからキャッシュを読み込む
                // No Read Allocは同様の場合に，その領域をキャッシュに読み込むことなくメモリから直接読み込む
                + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Inner cacheability attribute
                + TCR_EL1::EPD0::EnableTTBR0Walks // TTBR0_EL1アクセスしたい仮想アドレスがTLB (Translation Lookaside Buffer)になかった場合，Translation Table Walkを実行する．このビットを反転させると，同様の場合にTranslation faultを発生させる
                + TCR_EL1::A1::TTBR0 // TTBR0_EL1 defined the ASID (Address Space Identifier)
                + TCR_EL1::T0SZ.val(t0sz) // memory address space size
                + TCR_EL1::EPD1::DisableTTBR1Walks,// TTBR1_EL1でアクセスしたい仮想アドレスがTLB (Translation Lookaside Buffer)になかった場合，Translation Table Walkを実行する．このビットを反転させると，同様の場合にTranslation faultを発生させる
        );
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the MMU instance.
pub fn mmu() -> &'static impl memory::mmu::interface::MMU {
    &MMU
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use memory::mmu::MMUEnableError;

// MemoryManagementUnitiにmemory/mmu.rsのMMU traitを実装
impl memory::mmu::interface::MMU for MemoryManagementUnit {

    // kernelの初期化中に呼び出される．`BSP`で実装されている`virt_mem_layout()`からtranslation tablesを取得し，当該MMUをinstall/activateすることを期待する．
    // _arch/aarch64/memory/mmu.rsで実装されている．
    unsafe fn enable_mmu_and_caching(&self) -> Result<(), MMUEnableError> {

        // 多重に初期化することを防ぐ
        if unlikely(self.is_enabled()) {
            return Err(MMUEnableError::AlreadyEnabled);
        }

        // Fail early if translation granule is not supported.
        // ID_AA64MMFR0_EL1(Memory Model Feature Register)については以下を参照
        // ID_AA64MMFR0_EL1の24bit目から27bit目がTGran64で，これが0ならば64KB単位のmemory translationに対応しているっぽい
        // https://developer.arm.com/documentation/100403/0200/register-descriptions/aarch64-system-registers/id-aa64mmfr0-el1--aarch64-memory-model-feature-register-0--el1
        if unlikely(!ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported)) {
            return Err(MMUEnableError::Other(
                "Translation granule not supported in HW",
            ));
        }

        // Prepare the memory attribute indirection register.
        // MAIR_EL1レジスタに必要なメモリ属性を定義する
        // 上にこの関数の実装がある
        self.set_up_mair();

        // Populate translation tables.
        // KENRLE_TABLESは_arch/aarch64/memory/mmu/translation_table.rsで定義されるKernelTranslationTable型LVL2のtranslation table
        // populate_tt_entries()でlvl2,lvl3の全entryを初期化
        KERNEL_TABLES
            .populate_tt_entries()
            .map_err(|e| MMUEnableError::Other(e))?; // 失敗したときだけeをMMUEnableError::Other(e)にする

        // Set the "Translation Table Base Register".
        // x86のCR3的なやつを設定
        TTBR0_EL1.set_baddr(KERNEL_TABLES.phys_base_address());

        // キャッシュの動作に関するTCR_EL1(Translation Control Register)レジスタの設定
        self.configure_translation_control();

        // Switch the MMU on.
        // 命令同期バリア命令ISB(Instruction Synchronization Barrier)でCPU上で実行されている命令列のパイプラインをフラッシュする
        // First, force all previous changes to be seen before the MMU is enabled.
        barrier::isb(barrier::SY);

        // Enable the MMU and turn on data and instruction caching.
        // SCTLR_EL1(System Control Register)
        // https://developer.arm.com/documentation/ddi0595/2021-06/AArch64-Registers/SCTLR-EL1--System-Control-Register--EL1-
        // SCTLR::Mは0ビット目で，これを1にするとEL1&0のstage 1のaddress translationが有効になる
        // SCTLR::Cは2ビット目で，Stage 1 Cacheability, for data accesses
        // SCTLR::Iは12ビット目で，Stage 1 instruction access cacheability control, for accesses at EL0 and EL1
        SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);

        // Force MMU init to complete before next instruction.
        barrier::isb(barrier::SY);

        Ok(())
    }

    #[inline(always)]
    fn is_enabled(&self) -> bool {
        SCTLR_EL1.matches_all(SCTLR_EL1::M::Enable)
    }
}

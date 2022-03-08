// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! GICC Driver - GIC CPU interface.

// 新しいcrate synchronization::InitStateLockを追加
use crate::{
    bsp::device_driver::common::MMIODerefWrapper, exception, synchronization::InitStateLock,
};
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

register_bitfields! {
    u32,

    /// CPU Interface Control Register
    CTLR [
        Enable OFFSET(0) NUMBITS(1) []
    ],

    /// Interrupt Priority Mask Register
    PMR [
        Priority OFFSET(0) NUMBITS(8) []
    ],

    /// Interrupt Acknowledge Register
    IAR [
        InterruptID OFFSET(0) NUMBITS(10) []
    ],

    /// End of Interrupt Register
    EOIR [
        EOIINTID OFFSET(0) NUMBITS(10) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x000 => CTLR: ReadWrite<u32, CTLR::Register>),
        (0x004 => PMR: ReadWrite<u32, PMR::Register>),
        (0x008 => _reserved1),
        (0x00C => IAR: ReadWrite<u32, IAR::Register>),
        (0x010 => EOIR: ReadWrite<u32, EOIR::Register>),
        (0x014  => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MMIODerefWrapper<RegisterBlock>;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Representation of the GIC CPU interface.
/// 前回は生のRegistersだったのをInitStateLockで包んでいる
pub struct GICC {
    registers: InitStateLock<Registers>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
// 新しいcrate synchronization::interface::ReadWriteExを追加
use crate::synchronization::interface::ReadWriteEx;

impl GICC {
    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            // 前回は生のRegistersだったのをInitStateLockで包んでいる
            registers: InitStateLock::new(Registers::new(mmio_start_addr)),
        }
    }

    // 今回追加された関数
    // MMIO領域の先頭addressを設定
    pub unsafe fn set_mmio(&self, new_mmio_start_addr: usize) {
        self.registers
            .write(|regs| *regs = Registers::new(new_mmio_start_addr));
    }

    /// Accept interrupts of any priority.
    /// 任意の優先度の割り込みを受け入れる
    /// Quoting the GICv2 Architecture Specification:
    /// 以下にGICv2 Architectureの仕様書を引用する
    ///   "Writing 255 to the GICC_PMR always sets it to the largest supported priority field
    ///    value."
    ///   "GICC_PMRに255を書き込むと，supportされている最大の優先度場値に設定されます"
    /// # Safety
    ///
    /// - GICC MMIO registers are banked per CPU core. It is therefore safe to have `&self` instead
    ///   of `&mut self`.
    /// - GICC MMIO registersはCPU core毎に積まれているので，`&mut self`ではなく`&self`を持つのが安全だ．
    pub fn priority_accept_all(&self) {
        self.registers.read(|regs| {
            // GICC_PMR(Interrupt Priority Mask Register)に255を書き込んで，任意の優先度の割り込みを受け入れるようにする．
            // 割り込みの優先度は小さな値ほど優先され，大きな値ほど優先されなくなる
            // このregisterを255に設定することで，255以下の優先度の値を持っている割り込みをすべて受け入れる
            // https://developer.arm.com/documentation/ihi0048/b/Programmers--Model/CPU-interface-register-descriptions/Interrupt-Priority-Mask-Register--GICC-PMR?lang=en
            regs.PMR.write(PMR::Priority.val(255)); // Comment in arch spec.
        });
    }

    /// Enable the interface - start accepting IRQs.
    ///
    /// # Safety
    ///
    /// - GICC MMIO registers are banked per CPU core. It is therefore safe to have `&self` instead
    ///   of `&mut self`.
    pub fn enable(&self) {
        self.registers.read(|regs| {
            regs.CTLR.write(CTLR::Enable::SET);
        });
    }

    /// Extract the number of the highest-priority pending IRQ.
    ///
    /// Can only be called from IRQ context, which is ensured by taking an `IRQContext` token.
    ///
    /// # Safety
    ///
    /// - GICC MMIO registers are banked per CPU core. It is therefore safe to have `&self` instead
    ///   of `&mut self`.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn pending_irq_number<'irq_context>(
        &self,
        _ic: &exception::asynchronous::IRQContext<'irq_context>,
    ) -> usize {
        self.registers
            .read(|regs| regs.IAR.read(IAR::InterruptID) as usize)
    }

    /// Complete handling of the currently active IRQ.
    ///
    /// Can only be called from IRQ context, which is ensured by taking an `IRQContext` token.
    ///
    /// To be called after `pending_irq_number()`.
    ///
    /// # Safety
    ///
    /// - GICC MMIO registers are banked per CPU core. It is therefore safe to have `&self` instead
    ///   of `&mut self`.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn mark_comleted<'irq_context>(
        &self,
        irq_number: u32,
        _ic: &exception::asynchronous::IRQContext<'irq_context>,
    ) {
        self.registers.read(|regs| {
            regs.EOIR.write(EOIR::EOIINTID.val(irq_number));
        });
    }
}

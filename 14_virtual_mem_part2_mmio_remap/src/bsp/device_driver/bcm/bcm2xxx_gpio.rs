// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! GPIO Driver.

// 新しいcrate memory, core::sync::atomic::{AtomicUsize, Ordering}を追加
use crate::{
    bsp::device_driver::common::MMIODerefWrapper, driver, memory, synchronization,
    synchronization::IRQSafeNullLock,
};
use core::sync::atomic::{AtomicUsize, Ordering};
use register::{mmio::*, register_bitfields, register_structs};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// GPIO registers.
//
// Descriptions taken from
// - https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
// - https://datasheets.raspberrypi.org/bcm2711/bcm2711-peripherals.pdf
register_bitfields! {
    u32,

    /// GPIO Function Select 1
    GPFSEL1 [
        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART RX

        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART TX
        ]
    ],

    /// GPIO Pull-up/down Register
    ///
    /// BCM2837 only.
    GPPUD [
        /// Controls the actuation of the internal pull-up/down control line to ALL the GPIO pins.
        PUD OFFSET(0) NUMBITS(2) [
            Off = 0b00,
            PullDown = 0b01,
            PullUp = 0b10
        ]
    ],

    /// GPIO Pull-up/down Clock Register 0
    ///
    /// BCM2837 only.
    GPPUDCLK0 [
        /// Pin 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ],

    /// GPIO Pull-up / Pull-down Register 0
    ///
    /// BCM2711 only.
    GPIO_PUP_PDN_CNTRL_REG0 [
        /// Pin 15
        GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01
        ],

        /// Pin 14
        GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01
        ]
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => _reserved1),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => _reserved2),
        (0x94 => GPPUD: ReadWrite<u32, GPPUD::Register>),
        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
        (0x9C => _reserved3),
        (0xE4 => GPIO_PUP_PDN_CNTRL_REG0: ReadWrite<u32, GPIO_PUP_PDN_CNTRL_REG0::Register>),
        (0xE8 => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MMIODerefWrapper<RegisterBlock>;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub struct GPIOInner {
    registers: Registers,
}

// Export the inner struct so that BSPs can use it for the panic handler.
pub use GPIOInner as PanicGPIO;

/// Representation of the GPIO HW.
pub struct GPIO {
    // MMIODescriptorをGPIOの要素に追加
    mmio_descriptor: memory::mmu::MMIODescriptor,
    // MMIO領域の先頭仮想addressをGPIOの要素に追加
    virt_mmio_start_addr: AtomicUsize,
    inner: IRQSafeNullLock<GPIOInner>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl GPIOInner {
    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
        }
    }

    /// Init code.
    /// GPIOInnerの初期化
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub unsafe fn init(&mut self, new_mmio_start_addr: Option<usize>) -> Result<(), &'static str> {
        if let Some(addr) = new_mmio_start_addr {
            // new_mmio_start_addrがSome(addr)にmatchしたとき，registersにMMIO領域の先頭addressを設定する
            self.registers = Registers::new(addr);
        }

        Ok(())
    }

    /// Disable pull-up/down on pins 14 and 15.
    #[cfg(feature = "bsp_rpi3")]
    fn disable_pud_14_15_bcm2837(&mut self) {
        use crate::{time, time::interface::TimeManager};
        use core::time::Duration;

        // The Linux 2837 GPIO driver waits 1 µs between the steps.
        const DELAY: Duration = Duration::from_micros(1);

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        time::time_manager().spin_for(DELAY);

        self.registers
            .GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK15::AssertClock + GPPUDCLK0::PUDCLK14::AssertClock);
        time::time_manager().spin_for(DELAY);

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        self.registers.GPPUDCLK0.set(0);
    }

    /// Disable pull-up/down on pins 14 and 15.
    #[cfg(feature = "bsp_rpi4")]
    fn disable_pud_14_15_bcm2711(&mut self) {
        self.registers.GPIO_PUP_PDN_CNTRL_REG0.write(
            GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullUp
                + GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullUp,
        );
    }

    /// Map PL011 UART as standard output.
    ///
    /// TX to pin 14
    /// RX to pin 15
    pub fn map_pl011_uart(&mut self) {
        // Select the UART on pins 14 and 15.
        self.registers
            .GPFSEL1
            .modify(GPFSEL1::FSEL15::AltFunc0 + GPFSEL1::FSEL14::AltFunc0);

        // Disable pull-up/down on pins 14 and 15.
        #[cfg(feature = "bsp_rpi3")]
        self.disable_pud_14_15_bcm2837();

        #[cfg(feature = "bsp_rpi4")]
        self.disable_pud_14_15_bcm2711();
    }
}

impl GPIO {
    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide correct MMIO descriptors.
    /// 引数をMMIOの先頭addressで渡していたのを，MMIODescriptorで渡すように変更
    pub const unsafe fn new(mmio_descriptor: memory::mmu::MMIODescriptor) -> Self {
        Self {
            // mmio_descriptorとvirt_mmio_start_addrが今回追加された要素
            mmio_descriptor,
            virt_mmio_start_addr: AtomicUsize::new(0),
            // MMIODescriptorからMMIOの先頭addressを取り出してGPIOInnerを作成してIRQSafeNullLockで包んでいる
            inner: IRQSafeNullLock::new(GPIOInner::new(mmio_descriptor.start_addr().into_usize())),
        }
    }

    /// Concurrency safe version of `GPIOInner.map_pl011_uart()`
    pub fn map_pl011_uart(&self) {
        self.inner.lock(|inner| inner.map_pl011_uart())
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use synchronization::interface::Mutex;

impl driver::interface::DeviceDriver for GPIO {
    fn compatible(&self) -> &'static str {
        "BCM GPIO"
    }

    // GPIOの初期化
    unsafe fn init(&self) -> Result<(), &'static str> {
        // MMIOの先頭仮想addressを取得
        let virt_addr = memory::mmu::kernel_map_mmio(self.compatible(), &self.mmio_descriptor)?;

        self.inner
            .lock(|inner| inner.init(Some(virt_addr.into_usize())))?;

        self.virt_mmio_start_addr
            .store(virt_addr.into_usize(), Ordering::Relaxed);

        Ok(())
    }

    // GPIOからMMIOの先頭仮想addressを取得する関数
    fn virt_mmio_start_addr(&self) -> Option<usize> {
        // MMIOの先頭仮想addressを取得
        let addr = self.virt_mmio_start_addr.load(Ordering::Relaxed);

        if addr == 0 {
            // ヌルポはNoneで返す
            return None;
        }

        // 取得したMMIOの先頭仮想addressを返す
        Some(addr)
    }
}

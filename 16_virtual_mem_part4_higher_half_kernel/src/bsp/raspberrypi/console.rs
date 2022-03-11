// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! BSP console facilities.

use crate::{bsp::device_driver, console, cpu, driver};
use core::fmt;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// In case of a panic, the panic handler uses this function to take a last shot at printing
/// something before the system is halted.
///
/// We try to init panic-versions of the GPIO and the UART. The panic versions are not protected
/// with synchronization primitives, which increases chances that we get to print something, even
/// when the kernel's default GPIO or UART instances happen to be locked at the time of the panic.
/// 
/// Panic時に，panic handlerがこの関数を読んで何かしら出力してからsystemは停止する．
/// Kernelが普段使っているGPIOとUARTがlockされていたとしても出力できるように，同期原子性を保証しないPanic版のGPIOとUARTを初期化する．
///
/// # Safety
///
/// - Use only for printing during a panic.
/// 
/// 実機で実行するときの本番用のpanic_console_out
#[cfg(not(feature = "test_build"))]
pub unsafe fn panic_console_out() -> impl fmt::Write {
    use driver::interface::DeviceDriver;

    // If remapping of the driver's MMIO hasn't already happened, we won't be able to print. Just
    // park the CPU core in this case.
    // DriverのMMIO領域がまだremapされていなければ印字できない．
    // その場合CPUを停止する．
    // 前回までは，remapされていない場合物理addressを代わりに使っていた．
    let gpio_mmio_start_addr = match super::GPIO.virt_mmio_start_addr() {
        None => cpu::wait_forever(), // GPIOのMMIO領域がまだremapされていないので停止
        Some(x) => x, // GPIOのMMIO領域の先頭仮想address
    };

    let uart_mmio_start_addr = match super::PL011_UART.virt_mmio_start_addr() {
        None => cpu::wait_forever(), // UARTのMMIO領域がまだremapされていないので停止
        Some(x) => x, // UARTのMMIO領域の先頭仮想address
    };

    // Panic時に普通のGPIO, UART device driverがlockされていても出力できるように，
    // 原子性を保証しないPanic時専用のdevice driverを作る
    let mut panic_gpio = device_driver::PanicGPIO::new(gpio_mmio_start_addr);
    let mut panic_uart = device_driver::PanicUart::new(uart_mmio_start_addr);

    panic_gpio
        .init(None)
        .unwrap_or_else(|_| cpu::wait_forever());
    panic_gpio.map_pl011_uart();
    panic_uart
        .init(None)
        .unwrap_or_else(|_| cpu::wait_forever());

    panic_uart
}

/// Reduced version for test builds.
/// QEMUで実行するときのtest用のpanic_console_out
#[cfg(feature = "test_build")]
pub unsafe fn panic_console_out() -> impl fmt::Write {
    use driver::interface::DeviceDriver;

    let uart_mmio_start_addr = match super::PL011_UART.virt_mmio_start_addr() {
        None => cpu::wait_forever(), // ここでも，UART MMIOがremapされていなかったら停止するようにしている．
        Some(x) => x,
    };
    let mut panic_uart = device_driver::PanicUart::new(uart_mmio_start_addr);

    panic_uart
        .init(None)
        .unwrap_or_else(|_| cpu::qemu_exit_failure());

    panic_uart
}

/// Return a reference to the console.
pub fn console() -> &'static impl console::interface::All {
    &super::PL011_UART
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

/// Minimal code needed to bring up the console in QEMU (for testing only). This is often less steps
/// than on real hardware due to QEMU's abstractions.
#[cfg(feature = "test_build")]
pub fn qemu_bring_up_console() {
    use driver::interface::DeviceDriver;

    // Calling the UART's init ensures that the BSP's instance of the UART does remap the MMIO
    // addresses.
    unsafe {
        super::PL011_UART
            .init()
            .unwrap_or_else(|_| cpu::qemu_exit_failure());
    }
}

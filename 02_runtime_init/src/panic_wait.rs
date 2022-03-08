// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! A panic handler that infinitely waits.

use crate::cpu;
use core::panic::PanicInfo;

/// main.rsのkernel_initからここに飛ぶ
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // ./_arch/aarch64/cpu.rsのwait_foreverに飛ぶ
    cpu::wait_forever()
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! A panic handler that infinitely waits.

use crate::{cpu, println};
use core::panic::PanicInfo;

// QEMUの簡易UARTでerror messageを表示して停止するよ．
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(args) = info.message() {
        // 引数でerror messageが渡されている場合，それと一緒にKernel panicを表示するよ．
        println!("\nKernel panic: {}", args);
    } else {
        // 引数でerror messageが渡されていない場合，Kernel panicだけ表示するよ．
        println!("\nKernel panic!");
    }
    //_arch/__arch_name__/cpu.rsのwait_foreverに飛んでcpuを停止させるよ．
    cpu::wait_forever()
}

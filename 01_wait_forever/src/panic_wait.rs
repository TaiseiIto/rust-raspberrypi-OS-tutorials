// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! A panic handler that infinitely waits.

use core::panic::PanicInfo;

//例外処理のようなものだろうか.今はまだ使わないのでunimplementedになってる
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unimplemented!()
}

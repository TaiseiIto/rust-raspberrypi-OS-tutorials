// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSP console facilities.

use crate::console;
use core::fmt;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// A mystical, magical device for generating QEMU output out of the void.
struct QEMUOutput; /* QEMUで簡易的なUARTを実現する怪しげな仮想deviceを表す構造体 */

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
/// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
/// we get `write_fmt()` automatically.
///
/// `core::fmt::Write`を実装することによって`kernel`の`print!`や`println!` macrosで利用される
/// `format_args!` macrosが使えるようになる．
/// `write_str()`を実装することによって自動的に`write_fmt()`が使えるようになる．
///
/// See [`src/print.rs`].
///
/// [`src/print.rs`]: ../../print/index.html
impl fmt::Write for QEMUOutput {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        //QEMUの怪しげな仮想UARTに1文字ずつ書き込む
        for c in s.chars() {
            unsafe {
                core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
            }
        }

        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the console.
// ../../print.rsの_print関数から呼ばれる関数
// 仮想簡易UARTでQEMUに出力するときに呼ばれる
pub fn console() -> impl console::interface::Write {
    // QEMUで簡易的なUARTを実現する怪しげな仮想deviceを表す構造体を返す．
    QEMUOutput {}
}

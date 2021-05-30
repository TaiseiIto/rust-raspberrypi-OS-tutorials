// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Printing.

use crate::{bsp, console};
use core::fmt;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

// QEMUの簡易UARTで書式で表示するよ．
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use console::interface::Write;
    // bsp/raspberrypi/console.rsのconsole関数がQEMUの怪しげなdeviceを返す
    // その怪しげなdeviceに，同様にbsp/raspberrypi/console.rsで定義されているwrite_fmtを呼び出して1文字列argsを書き込む．
    bsp::console::console().write_fmt(args).unwrap();
}

/// Prints without a newline.
/// QEMUの簡易UARTで改行なしで表示するmacroだよ．
/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
#[macro_export]
macro_rules! print {
    // 上の関数を実行して改行なしの書式で表示するよ．
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

/// Prints with a newline.
/// QEMUの簡易UARTで1行表示するよ．
/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
#[macro_export]
macro_rules! println {
    // 引数がない場合改行だけ表示するmacroだよ．
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        // 上の関数を実行して引数がある場合改行付きの書式で表示するよ．
        $crate::print::_print(format_args_nl!($($arg)*));
    })
}

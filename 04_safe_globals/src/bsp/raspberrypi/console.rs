// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! BSP console facilities.

use crate::{console, synchronization, synchronization::NullLock};
use core::fmt;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// A mystical, magical device for generating QEMU output out of the void.
///
/// The mutex protected part.
struct QEMUOutputInner {
    chars_written: usize, // consoleが今までに出力した文字数
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The main struct.
/// QEMUOutput
///  inner: NullLock<QEMUOutputInner> Mutexの仕組みを提供してる構造体
///   data: Unsafecell<QemuOutputInner> mutableな参照をいくらでも作れるようにするやつ
///    QEMUOutputInner Mutexで保護されるやつ
///     chars_written: usize cnsoleが今までに出力した文字数
pub struct QEMUOutput {
    inner: NullLock<QEMUOutputInner>,
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

// ずっとそこにいてくれるQEMUOutput
static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl QEMUOutputInner {
    const fn new() -> QEMUOutputInner {
        QEMUOutputInner { chars_written: 0 } // 出力文字数を0で初期化
    }

    /// Send a character.
    fn write_char(&mut self, c: char) {
        unsafe {
            core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
        }

        self.chars_written += 1; // 出力文字数を数える
    }
}

/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
/// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
/// we get `write_fmt()` automatically.
///
/// The function takes an `&mut self`, so it must be implemented for the inner struct.
///
/// See [`src/print.rs`].
///
/// [`src/print.rs`]: ../../print/index.html
impl fmt::Write for QEMUOutputInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            // Convert newline to carrige return + newline.
            // CRLFで改行させてる
            if c == '\n' {
                self.write_char('\r')
            }

            self.write_char(c);
        }
        // 空のタプルでOKを返す
        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl QEMUOutput {
    /// Create a new instance.
    pub const fn new() -> QEMUOutput {
        QEMUOutput {
            inner: NullLock::new(QEMUOutputInner::new()),
        }
    }
}

/// Return a reference to the console.
pub fn console() -> &'static impl console::interface::All {
    &QEMU_OUTPUT // ずっとそこにいてくれるQEMUOutputの参照
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use synchronization::interface::Mutex;

/// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
/// serialize access.
impl console::interface::Write for QEMUOutput {
    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        // Fully qualified syntax for the call to `core::fmt::Write::write:fmt()` to increase
        // readability.
        // 「今までに出力した文字数」をロックして書き込む
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }
}

impl console::interface::Statistics for QEMUOutput {
    fn chars_written(&self) -> usize {
        // 「今までに出力した文字数」をロックしてその値を返す
        self.inner.lock(|inner| inner.chars_written)
    }
}

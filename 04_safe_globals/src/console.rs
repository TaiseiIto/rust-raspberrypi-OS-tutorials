// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! System console.

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Console interfaces.
/// Consoleが実装すべきinterfaces
pub mod interface {
    use core::fmt;

    /// Console write functions.
    /// Consoleへの書き込み
    pub trait Write {
        /// Write a Rust format string.
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
    }

    /// Console statistics.
    /// 出力した文字数を数えるよ．
    pub trait Statistics {
        /// Return the number of characters written.
        fn chars_written(&self) -> usize {
            0
        }
    }

    /// Trait alias for a full-fledged console.
    /// 上の2つのtraitsを合わせたalias
    pub trait All = Write + Statistics;
}

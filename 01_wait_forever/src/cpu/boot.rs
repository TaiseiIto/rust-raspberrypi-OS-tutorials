// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//! Boot code.

// aarch64を対象とする場合対応するboot.rsを使う的な?
#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/cpu/boot.rs"]
mod arch_boot;

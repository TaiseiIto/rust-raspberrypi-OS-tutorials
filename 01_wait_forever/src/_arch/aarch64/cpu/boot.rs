// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//! Architectural boot code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::boot::arch_boot

// Assembly counterpart to this file.
// boot.sのassemblyをこのrustにincludeする的な?
// そうするとこのfileからboot.sで定義されたsymbolを使えるようになる的な?
global_asm!(include_str!("boot.s"));

// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2022 Andre Richter <andre.o.richter@gmail.com>

//! Architectural boot code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::boot::arch_boot

// Assembly counterpart to this file.
core::arch::global_asm!(include_str!("boot.s"));

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// The Rust entry of the `kernel` binary.
/// boot coreが最初に実行するRustのentry pointだよ．
/// The function is called from the assembly `_start` function.
/// ./boot.sの_start関数からここに飛ぶよ．
/// # Safety
///
/// - The `bss` section is not initialized yet. The code must not use or reference it in any way.
/// `bss`領域がまだ初期化されていないので，ここで`bss`領域を使っちゃいけないよ．
#[no_mangle]
pub unsafe fn _start_rust() -> ! {
    crate::kernel_init()
}

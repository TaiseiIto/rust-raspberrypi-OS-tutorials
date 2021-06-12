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

use crate::runtime_init;

// Assembly counterpart to this file.
// ./boot.sを取り込むよ．
global_asm!(include_str!("boot.s"));

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// The Rust entry of the `kernel` binary.
/// 
/// The function is called from the assembly `_start` function.
/// boot coreだけが./boot.sの_start関数からここに飛んでくるよ．
/// # Safety
/// - `bss`領域はまだ初期化されてないから使っちゃだめだよ．
/// - The `bss` section is not initialized yet. The code must not use or reference it in any way.
#[no_mangle]
pub unsafe fn _start_rust() -> ! {
    // ../../../runtime_init.rsのruntime_initに飛ぶよ．
    runtime_init::runtime_init() // mainを実行する前に必要な初期化の処理だよ．
}

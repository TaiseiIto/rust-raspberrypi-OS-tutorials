// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSP Processor code.

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Used by `arch` code to find the early boot core.
#[no_mangle]
#[link_section = ".text._start_arguments"]
// ../../_arch/__arch_name__/cpu/boot.sで，これ以外の番号のcoreを停止させているよ．
pub static BOOT_CORE_ID: u64 = 0;

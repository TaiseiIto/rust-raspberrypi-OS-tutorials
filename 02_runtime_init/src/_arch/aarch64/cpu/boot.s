// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------
// Definitions
//--------------------------------------------------------------------------------------------------

// 任意のsymbolのPC相対addressを任意のregisterに読み込む疑似命令ADR_RELを定義してるっぽい
// Load the address of a symbol into a register, PC-relative.
// SymbolのaddresをProgram Counter相対でregisterに読み込む．
// The symbol must lie within +/- 4 GiB of the Program Counter.
// 従ってSymbolはProgram Counterの+/- 4GiB以内に配置されなければならない．
// # Resources
//
// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
.macro ADR_REL register, symbol
	adrp	\register, \symbol
	add	\register, \register, #:lo12:\symbol
.endm

//これは何かな?
//各coreが自分自身のcore idを取得するためのmaskらしい
.equ _core_id_mask, 0b11

//--------------------------------------------------------------------------------------------------
// Public Code 
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start() ../../bsp/raspberrypi/link.ldでここをextry pointにしてる
//------------------------------------------------------------------------------
_start:
	// Only proceed on the boot core. Park it otherwise.
	mrs	x1, MPIDR_EL1		  // move system register命令
							  // MPIDR_EL1に自身のcoreに関する情報が書かれていて，それをx1に転送する
							  // MPIDR_EL1についてはhttps://developer.arm.com/documentation/ddi0500/j/System-Control/AArch64-register-descriptions/Multiprocessor-Affinity-Register
	and	x1, x1, _core_id_mask
	ldr	x2, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
	cmp	x1, x2				  // x1(自分自身のcore id)とx2(boot core id)を比較
	b.ne	1f				  // boot core以外は1に飛んで停止する

	// If execution reaches here, it is the boot core. Now, prepare the jump to Rust code.
	// 以下boot coreのみが実行する処理．stack pointerを設定してRust codeに飛ぶ．

	// Set the stack pointer.
	ADR_REL	x0, __boot_core_stack_end_exclusive	//../../bsp/raspberrypi/link.ldで，stackはentry pointから番地の若い方向に伸びると書かれている
	mov	sp, x0

	// Jump to Rust code.
	// ./boot.rsの_start_rustに飛ぶ
	b	_start_rust

	// Infinitely wait for events (aka "park the core").
	// boot core以外はここに飛んで停止する．
1:	wfe
	b	1b

.size	_start, . - _start
.type	_start, function
.global	_start

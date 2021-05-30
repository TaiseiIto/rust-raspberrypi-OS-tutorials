// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------
// Definitions
//--------------------------------------------------------------------------------------------------

// Load the address of a symbol into a register, PC-relative.
// Program Counter相対でsymbolのaddressをregisterに読み込む疑似命令を定義するよ．
// The symbol must lie within +/- 4 GiB of the Program Counter.
// symbolはProgram Counterの+/- 4 GiBの範囲内にないといけないよ．
// # Resources
//
// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
.macro ADR_REL register, symbol
	adrp	\register, \symbol
	add	\register, \register, #:lo12:\symbol
.endm

.equ _core_id_mask, 0b11 /* 自分のcore番号を取得するためのmaskだよ */

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
.section .text._start /* ここが../../../bsp/__board_name__/link.ldで定義されてるOSのentry pointだよ */

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start: /* entry pointに関数を配置するよ *.
	// Only proceed on the boot core. Park it otherwise.
	// 自分のcore番号をx1に取得するよ
	mrs	x1, MPIDR_EL1
	and	x1, x1, _core_id_mask
	ldr	x2, BOOT_CORE_ID      // provided by ../../../bsp/__board_name__/cpu.rs
	cmp	x1, x2				  // ../../../bsp/__board_name__/cpu.rsで定義されるboot core番号と自分のcore番号を比較するよ
	b.ne	1f				  // boot core以外は1に飛んで停止するよ

	// If execution reaches here, it is the boot core. Now, prepare the jump to Rust code.
	// boot coreだけが以下の命令を実行するよ

	// Set the stack pointer.
	ADR_REL	x0, __boot_core_stack_end_exclusive /* ../../../bsp/raspberrypi/link.ldで定義されるstackの底 */
	mov	sp, x0

	// Jump to Rust code.
	b	_start_rust /* ./boot.rsの_srart_rustに飛ぶよ */

	// Infinitely wait for events (aka "park the core").
	// boot core以外のcoreはwait for eventを無限loopすることで停止するよ
1:	wfe
	b	1b

.size	_start, . - _start
.type	_start, function
.global	_start

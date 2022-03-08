// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2022 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------
// Definitions
//--------------------------------------------------------------------------------------------------

// Load the address of a symbol into a register, PC-relative.
// symbolのProgram Counter相対addressをregisterに読み込む疑似命令をここで定義するよ．
// The symbol must lie within +/- 4 GiB of the Program Counter.
// addressはPC相対だから，PC +/- 4 GiBでなければならないよ．
// # Resources
//
// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
.macro ADR_REL register, symbol
	adrp	\register, \symbol
	add	\register, \register, #:lo12:\symbol
.endm

.equ _core_id_mask, 0b11 /* 自分のcore番号を取得するためのmask */

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
.section .text._start /* ここをentry pointにすることが../../bsp/raspberrypi/link.ldに記述されてるよ． */

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start: /* entry pointに_start関数を配置するよ． */
	// Only proceed on the boot core. Park it otherwise.
	// 自分のcore番号を取得するよ
	mrs	x1, MPIDR_EL1
	and	x1, x1, _core_id_mask
	ldr	x2, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
	// If execution reaches here, it is the boot core. Now, prepare the jump to Rust code.
	// boot coreのみが以下の命令を実行するよ．
	cmp	x1, x2
	b.ne	.L_parking_loop

	// If execution reaches here, it is the boot core.

	// Initialize DRAM.
	ADR_REL	x0, __bss_start
	ADR_REL x1, __bss_end_exclusive

.L_bss_init_loop:
	cmp	x0, x1
	b.eq	.L_prepare_rust
	stp	xzr, xzr, [x0], #16
	b	.L_bss_init_loop

	// Prepare the jump to Rust code.
.L_prepare_rust:
	// Set the stack pointer.
	ADR_REL	x0, __boot_core_stack_end_exclusive // ../../../bsp/raspberrypi/link.ldで定義されているstackの底
	mov	sp, x0

	// Jump to Rust code.
	b	_start_rust // ./boot.rsで定義されているRustのentry pointに飛ぶよ．

	// Infinitely wait for events (aka "park the core").
.L_parking_loop:
	wfe
	b	.L_parking_loop

.size	_start, . - _start
.type	_start, function
.global	_start

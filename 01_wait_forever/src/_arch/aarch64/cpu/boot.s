// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start()
// ../bps/raspberry/link.ldでentry pointからこの関数に飛ぶようにしてる
//------------------------------------------------------------------------------
_start:
	// Infinitely wait for events (aka "park the core").
1:	wfe
    //指定されたlabelへの無条件分岐(1bというlabelが見当たらないが...1を指してるのか?)
	//https://developer.arm.com/documentation/100076/0100/a64-instruction-set-reference/a64-general-instructions/b?lang=en
	b	1b

//_startの大きさはここの番地-_startの番地ですよ的な
.size	_start, . - _start
//_startは関数ですよ的な
.type	_start, function
//_startをglobalに公開する的な
.global	_start

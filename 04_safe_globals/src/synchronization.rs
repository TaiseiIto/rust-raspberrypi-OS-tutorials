// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

//! Synchronization primitives.
//!
//! # Resources
//!
//!   - <https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html>
//!   - <https://stackoverflow.com/questions/59428096/understanding-the-send-trait>
//!   - <https://doc.rust-lang.org/std/cell/index.html>

use core::cell::UnsafeCell;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Synchronization interfaces.
/// 同期のinterfaces
pub mod interface {

    /// Any object implementing this trait guarantees exclusive access to the data wrapped within
    /// the Mutex for the duration of the provided closure.
    /// このtraitを実装するobjectは与えられたclosureの間Mutexでwrapされたdateへの排他的accessを保証する
    pub trait Mutex {
        /// The type of the data that is wrapped by this mutex.
        /// このmutexでwrapされるデータ型
        type Data;

        /// Locks the mutex and grants the closure temporary mutable access to the wrapped data.
        /// mutexをlockし、closure(f)がwrapされたdataに対して一時的にmutable accessできるようにする．
        /// fはFnOnce traitの実装である関数
        /// FnOnceは一度しか実行できないclosure
        /// closureはどうやらCでいう関数ポインタ的な奴らしい
        /// fはmutableなData型の参照を受け取り、R型を返す
        /// Dataの所有権をfに渡してしまわないように参照渡しにしている
        /// 流れ的には、lockして、fを実行して、R型のfの返り値をそのまま返す感じ
        fn lock<R>(&self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
    }
}

/// A pseudo-lock for teaching purposes.
/// 簡易的なlock
/// In contrast to a real Mutex implementation, does not protect against concurrent access from
/// other cores to the contained data. This part is preserved for later lessons.
/// 実際のMutexの実装とは違い，ほかのcoreからの同時accessは防げない．
/// この部分は後のlessonsまでこのままです．
/// The lock will only be used as long as it is safe to do so, i.e. as long as the kernel is
/// executing single-threaded, aka only running on a single core with interrupts disabled.
/// このlockはそうするのが安全な時のみ使われます．
/// つまりkernelが単一のthreadで実行を続ける間のみ．
/// 言い換えると単一のcoreで割り込みも無効な状態で実行する間のみ
pub struct NullLock<T>
where
    T: ?Sized, // T型はSized trait(compile時に大きさが確定する)を実装していなくてもよい
{
    // Rustにおけるobjectの借用は、mutableな参照が1つあるか、immutableな参照が複数あるか
    // 複数のthreadやprocessが同時にobjectをmutableに借りられるような仕組みとして、UnsafeCellを使っている
    data: UnsafeCell<T>, // T型をUnsafeCellでwrapしたやつ
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

// NullLock<T>にSend traitとSync traitを実装する
// 今は特に何の関数も実装されてないけど、これから実装されていくのかな？
unsafe impl<T> Send for NullLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for NullLock<T> where T: ?Sized + Send {}

// NullLock<T>のimpl
impl<T> NullLock<T> {
    /// Create an instance.
    /// T型のdataをUnsafeCellでwrapしてさらにそれをNullLockでwrapした実体を返す
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
// NullLock<T>にinterface::Mutex traitを実装する
impl<T> interface::Mutex for NullLock<T> {
    type Data = T; // T型をこのMutexで保護してほしい

    fn lock<R>(&self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
        // In a real lock, there would be code encapsulating this line that ensures that this
        // mutable reference will ever only be given out once at a time.
        // 実際のlockでは，この行を包みこのmutable参照が一回のみ与えられることを保証するcodeがある
        // self.dataはUnsafeCell
        // self.data.get()はUnsafeCellの中身に対するmutableなpointer
        // &mut *self.data.get()はUnsafeCellの中身に対するmutableな参照
        let data = unsafe { &mut *self.data.get() };
        // closure呼び出し
        f(data)
    }
}

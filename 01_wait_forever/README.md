# Tutorial 01 - Wait Forever
# 第1章 - 永遠に待機

## tl;dr
## 要約

- The project skeleton is set up.
- A small piece of assembly code runs that just halts all CPU cores executing the kernel code.

- 骨組みが設置される．
- kernel codeを実行するCPUの全てのcoreを停止させるassembly codeの小片を走らせる．

## Building

- `Makefile` targets:
    - `doc`: Generate documentation.
    - `qemu`: Run the `kernel` in QEMU
    - `clippy`これは何だ?
    - `clean`
    - `readelf`: Inspect the `ELF` output. ELF出力を検査
    - `objdump`: Inspect the assembly. assemblyを検査
    - `nm`: Inspect the symbols. symbolsを検査

## Code to look at

- `BSP`-specific `link.ld` linker script.
    - Load address at `0x8_0000`
    - Only `.text` section.
- `main.rs`: Important [inner attributes]:
    - `#![no_std]`, `#![no_main]`
- `boot.s`: Assembly `_start()` function that executes `wfe` (Wait For Event), halting all cores
  that are executing `_start()`.
- We (have to) define a `#[panic_handler]` function to make the compiler happy.
    - Make it `unimplemented!()` because it will be stripped out since it is not used.

- `BSP`別の`link.ld` linker記述
    - `0x8_0000`番地を読み込む．
    - `.text` sectionのみ
- `main.rs`; 重要[内部属性]:
    - `#![no_std]`, `#![no_main]`
- `boot.s`: Eventを待つ`wfe`を実行し，`_start()`を実行する全てのcoreを停止するassembly`_start()`関数
- compilerを大事にするために`#[panic_handler]`関数を定義しなければならない．
    - 使われていないうちはこれを取り除くためこれを`unimplemented!()`にする．

[inner attributes]: https://doc.rust-lang.org/reference/attributes.html

### Test it

In the project folder, invoke QEMU and observe the CPU core spinning on `wfe`:

project folderにて，QEMUを呼び出してCPU coreが`wfe`で回っていることを確認する．

```console
$ make qemu
[...]
IN:
0x00080000:  d503205f  wfe
0x00080004:  17ffffff  b        #0x80000
```

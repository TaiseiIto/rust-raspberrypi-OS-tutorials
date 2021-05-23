# Tutorial 01 - Wait Forever
# ��1�� - �i���ɑҋ@

## tl;dr
## �v��

- The project skeleton is set up.
- A small piece of assembly code runs that just halts all CPU cores executing the kernel code.

- ���g�݂��ݒu�����D
- kernel code�����s����CPU�̑S�Ă�core���~������assembly code�̏��Ђ𑖂点��D

## Building

- `Makefile` targets:
    - `doc`: Generate documentation.
    - `qemu`: Run the `kernel` in QEMU
    - `clippy`����͉���?
    - `clean`
    - `readelf`: Inspect the `ELF` output. ELF�o�͂�����
    - `objdump`: Inspect the assembly. assembly������
    - `nm`: Inspect the symbols. symbols������

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

- `BSP`�ʂ�`link.ld` linker�L�q
    - `0x8_0000`�Ԓn��ǂݍ��ށD
    - `.text` section�̂�
- `main.rs`; �d�v[��������]:
    - `#![no_std]`, `#![no_main]`
- `boot.s`: Event��҂�`wfe`�����s���C`_start()`�����s����S�Ă�core���~����assembly`_start()`�֐�
- compiler��厖�ɂ��邽�߂�`#[panic_handler]`�֐����`���Ȃ���΂Ȃ�Ȃ��D
    - �g���Ă��Ȃ������͂������菜�����߂����`unimplemented!()`�ɂ���D

[inner attributes]: https://doc.rust-lang.org/reference/attributes.html

### Test it

In the project folder, invoke QEMU and observe the CPU core spinning on `wfe`:

project folder�ɂāCQEMU���Ăяo����CPU core��`wfe`�ŉ���Ă��邱�Ƃ��m�F����D

```console
$ make qemu
[...]
IN:
0x00080000:  d503205f  wfe
0x00080004:  17ffffff  b        #0x80000
```

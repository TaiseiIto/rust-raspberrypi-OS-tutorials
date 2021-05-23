# Before we start
# 始める前に

The following text is a 1:1 copy of the documentation that can be found at the top of the kernel's
main source code file in each tutorial. It describes the general structure of the source code, and
tries to convey the philosophy behind the respective approach. Please read it to make yourself
familiar with what you will encounter during the tutorials. It will help you to navigate the code
better and understand the differences and additions between the separate tutorials.

以下の文章は各tutorialのkernelのmain source code fileの先頭部分の複製だ．そのsource codeの大まかな構造を述べ，各手法の背後にある思想を伝える．このtutorialsで取り組むことに慣れるために読んでください．各tutorialのcodeを読み差分と追加部分を理解する助けになるでしょう．

Please also note that the following text will reference source code files (e.g. `**/memory.rs`) or
functions that won't exist yet in the first bunch of the tutorials. They will be added gradually as
the tutorials advance.

以下の文章は`**/memory.rs`等のsource codeや，まだ存在しない関数を参照していることに留意してください．これらはtutorialsを進めていくにしたがって段階的に加えられていきます．

Have fun!

# Code organization and architecture
# codeの組織と構成

The code is divided into different *modules*, each representing a typical **subsystem** of the
`kernel`. Top-level module files of subsystems reside directly in the `src` folder. For example,
`src/memory.rs` contains code that is concerned with all things memory management.

このcodeはそれぞれがkernelのsubsystemを記述する複数のmodulesに分割されます．subsystemsの最上位のmodule filesはsrc folderの直下にあります．例えば，`src/memory.rs`はmemory管理に関する全てのcodeを含みます．

## Visibility of processor architecture code
## processor architecture codeの可読性

Some of the `kernel`'s subsystems depend on low-level code that is specific to the target processor
architecture. For each supported processor architecture, there exists a subfolder in `src/_arch`,
for example, `src/_arch/aarch64`.

`kernel`のsubsystemsには対象processor architectureごとの低水準codeに依存するものがある．`src/_arch`には対応するprocessor architectureごとに，subfolderがある．

The architecture folders mirror the subsystem modules laid out in `src`. For example, architectural
code that belongs to the `kernel`'s MMU subsystem (`src/memory/mmu.rs`) would go into
`src/_arch/aarch64/memory/mmu.rs`. The latter file is loaded as a module in `src/memory/mmu.rs`
using the `path attribute`. Usually, the chosen module name is the generic module's name prefixed
with `arch_`.

これらのarchitecture foldersは`src`の外に置かれるsubsystem modulesを写し出す．例えば，`kernel`のMMU subsystem(`src/memory/mmu.rs`)に属するarchitectural codeは，`src/_arch/aarch64/memory/mmu.rs`に通じる．後者のfileは`path attribute`を使って`src/memory/mmy.rs`内のmoduleとして読み込まれる．一般に，選択されたmodule名は先頭に`arch_`を付けた包括module名だ．

For example, this is the top of `src/memory/mmu.rs`:

```
#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/memory/mmu.rs"]
mod arch_mmu;
```

Often times, items from the `arch_ module` will be publicly reexported by the parent module. This
way, each architecture specific module can provide its implementation of an item, while the caller
must not be concerned which architecture has been conditionally compiled.

しばしば`arch_module`に由来するitemは親moduleによりpublicに再出力される．このように，呼び出し側にどのarchitectureでcompileされているのか気を遣わせないために，各architecture固有のmoduleはitemの実装を提供する．

## BSP code

`BSP` stands for Board Support Package. `BSP` code is organized under `src/bsp.rs` and contains
target board specific definitions and functions. These are things such as the board's memory map or
instances of drivers for devices that are featured on the respective board.

`BPS`とはBoard Support Packageのことです．`BSP`のcodeは`src/bps.rs`以下に構成され，対象基盤別の定義及び関数を含みます．基盤memory mapや各基盤に特有のdevice driverの実体などがあります．

Just like processor architecture code, the `BSP` code's module structure tries to mirror the
`kernel`'s subsystem modules, but there is no reexporting this time. That means whatever is provided
must be called starting from the `bsp` namespace, e.g. `bsp::driver::driver_manager()`.

processor architecture codeと同様に，`BPS` codeのmodule構造も`kernel`のsubsystem modulesを写し出しますが，今回は再出力はありません．つまり，提供されたものは`bps::driver::driver_manager()`のように必ずbps名前空間から呼び出されるということです．

## Kernel interfaces

Both `arch` and `bsp` contain code that is conditionally compiled depending on the actual target and
board for which the kernel is compiled. For example, the `interrupt controller` hardware of the
`Raspberry Pi 3` and the `Raspberry Pi 4` is different, but we want the rest of the `kernel` code to
play nicely with any of the two without much hassle.

`arch`と`bps`は対象architectureと基盤に従ってcompileされるcodeを含みます．例えば，`Raspberry Pi 3`と`Raspberry Pi 4`の割り込み制御hardwareは異なりますが，我々はそのどちらでも労力を使わずに動かすための`kernel` codeの土台を必要としています．

In order to provide a clean abstraction between `arch`, `bsp` and `generic kernel code`, `interface`
traits are provided *whenever possible* and *where it makes sense*. They are defined in the
respective subsystem module and help to enforce the idiom of *program to an interface, not an
implementation*. For example, there will be a common IRQ handling interface which the two different
interrupt controller `drivers` of both Raspberrys will implement, and only export the interface to
the rest of the `kernel`.

`arch`，`bsp`，`generic kernel code`の抽象化を提供する代わりに，`interface` traitsが可能な限り道理にかなうように提供されます．これらは各subsystem moduleで定義され，programから直接実装ではなくinterfaceを介した表現を可能にします．例えば，2種類のRaspberryそれぞれの割り込み制御器driversへの一般IRQ処理interfaceが実装され，kernelの土台へのinterfaceのみが出力されるでしょう．(仮想化によってhardwareの違いを隠蔽するということかな)

```
        +-------------------+
        | Interface (Trait) |
        |                   |
        +--+-------------+--+
           ^             ^
           |             |
           |             |
+----------+--+       +--+----------+
| kernel code |       |  bsp code   |
|             |       |  arch code  |
+-------------+       +-------------+
```

# Summary
# まとめ

For a logical `kernel` subsystem, corresponding code can be distributed over several physical
locations. Here is an example for the **memory** subsystem:

- `src/memory.rs` and `src/memory/**/*`
  - Common code that is agnostic of target processor architecture and `BSP` characteristics.
    - Example: A function to zero a chunk of memory.
  - Interfaces for the memory subsystem that are implemented by `arch` or `BSP` code.
    - Example: An `MMU` interface that defines `MMU` function prototypes.
- `src/bsp/__board_name__/memory.rs` and `src/bsp/__board_name__/memory/**/*`
  - `BSP` specific code.
  - Example: The board's memory map (physical addresses of DRAM and MMIO devices).
- `src/_arch/__arch_name__/memory.rs` and `src/_arch/__arch_name__/memory/**/*`
  - Processor architecture specific code.
  - Example: Implementation of the `MMU` interface for the `__arch_name__` processor
    architecture.

From a namespace perspective, **memory** subsystem code lives in:

- `crate::memory::*`
- `crate::bsp::memory::*`

論理`kernel` subsystemのため，対応するcodeは幾つかの場所に分配されます．**memory** subsystemの例を示します．

- `src/memory.rs`と`src/memory/**/*`
  - 不可知な対象processor architectureと`BPS`特性に共通codeです．
    - 例:ひとまとまりのmemory領域を0番地に合わせるための関数
  - `arch`や`BSP`のcodeで実装されたmemory subsystemのinterfaces
    - 例:`MMU`関数のprototype宣言を定義する`MMU`interface
- `src/bps/__board_name__/memory.rs`と`src/bps/__board_name__/memory/**/*`
  - `BPS`別のcode
  - 例:対象基盤のmemory map (DRAMとMMIO機器の物理番地)
- `src/_arch/__arch_name__/memory.rs`と`src/_arch/__arch_name__/memory/**/*`
  - Processor architecture別のcode
  - 例:`__arch_name__` processor向けの`MMU` interfaceの実装

名前空間の考え方から，**memory** subsystem codeは

- `crate::memory::*`
- `crate::bsp::memory::*`

にあります．

# Boot flow
# 起動の流れ

1. The kernel's entry point is the function `cpu::boot::arch_boot::_start()`.
    - It is implemented in `src/_arch/__arch_name__/cpu/boot.s`.

1. kernelの始点は`cpu::boot::arch_boot::_start()`関数です．
    - これは`src/_arch/__arch_name__/cpu/boot.s`に実装されています．


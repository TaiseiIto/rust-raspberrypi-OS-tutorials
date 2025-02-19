# Operating System development tutorials in Rust on the Raspberry Pi
# RustによるRaspberry PiのOS開発指導

![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/BSP-RPi3/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/BSP-RPi4/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/Unit-Tests/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/Integration-Tests/badge.svg) ![](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue)

<br/>

<img src="doc/header.jpg" height="372"> <img src="doc/minipush_demo_frontpage.gif" height="372">

## ?? Introduction

This is a tutorial series for hobby OS developers who are new to ARM's 64 bit [ARMv8-A
architecture]. The tutorials will give a guided, step-by-step tour of how to write a [monolithic]
Operating System `kernel` for an `embedded system` from scratch. They cover implementation of common
Operating Systems tasks, like writing to the serial console, setting up virtual memory and handling
HW exceptions. All while leveraging `Rust`'s unique features to provide for safety and speed.

ARM64bit architecture初心者向けのOS開発入門編です．
組み込みsystem向けの堅固なkernelを書くための手順を示します．
serial consoleへの書き込み，仮想memoryの設定，Hardware例外などOSが一般的に有する機能の実装を網羅します．
全てにおいて安全性と速度を与えるためにRustの特徴を用います．

Have fun!

_Best regards,<br>Andre ([@andre-richter])_

P.S.: Chinese :cn: versions of the tutorials were started by [@colachg] and [@readlnh]. You can find
them as [`README.CN.md`](README.CN.md) in the respective folders. They are a bit out-of-date at the
moment though.

[ARMv8-A architecture]: https://developer.arm.com/products/architecture/cpu-architecture/a-profile/docs
[monolithic]: https://en.wikipedia.org/wiki/Monolithic_kernel
[@andre-richter]: https://github.com/andre-richter
[@colachg]: https://github.com/colachg
[@readlnh]: https://github.com/readlnh

## ? Organization

- Each tutorial contains a stand-alone, bootable `kernel` binary.
- 各項目は独立した起動可能なkernel bynaryを含みます．
- Each new tutorial extends the previous one.
- 新しい項目は前の項目の拡張です．
- Each tutorial `README` will have a short `tl;dr` section giving a brief overview of the additions,
  and show the source code `diff` to the previous tutorial, so that you can conveniently inspect the
  changes/additions.
    - Some tutorials have a full-fledged, detailed text in addition to the `tl;dr` section. The
      long-term plan is that all tutorials get a full text, but for now this is exclusive to
      tutorials where I think that `tl;dr` and `diff` are not enough to get the idea.
- 各項目のREADMEには追加部分の大まかな全体像を示す短い要約があり，前項目とのsource codeの差分を示すので，変更点，追加点を簡単に確認できます．
    - 要約に加えて詳細を記述している項目もあります．長期計画的には全ての項目に詳細を書くつもりですが，今は要約と差分だけでは理解するのに不足していると考えられる項目に限られている．
- The code written in these tutorials supports and runs on the **Raspberry Pi 3** and the
  **Raspberry Pi 4**.
  - Tutorials 1 till 5 are groundwork code which only makes sense to run in `QEMU`.
  - Starting with [tutorial 5](05_drivers_gpio_uart), you can load and run the kernel on the real
    Raspberrys and observe output over `UART`.
- これらの項目に書かれたcodeはRaspberry Pi 3およびRaspberry Pi 4に対応し，走らせることができます．
  - 項目1から5まではQEMU上で走らせるための土台を作ります．
  - 項目5から，kernelをRaspberryに読み込んで走らせ，UARTによる出力を確認します．
- Although the Raspberry Pi 3 and 4 are the main target boards, the code is written in a modular
  fashion which allows for easy porting to other CPU architectures and/or boards.
  - I would really love if someone takes a shot at a **RISC-V** implementation!
- Raspberry Pi 3と4が対象基盤ですが，このcodeはほかのCPU architecturesや基盤に簡単に移植できるmodular方式で書かれる．
  - 誰かがRISC-Vの実装を作ってくれることを期待している．
- For editing, I recommend [Visual Studio Code] with [Rust Analyzer].
- 編集にはVisual Studio CodeでRust Analyzerを使うことをお勧めする．
- In addition to the tutorial text, also check out the `make doc` command in each tutorial. It lets
  you browse the extensively documented code in a convenient way.
- この解説に加えて，各項目のmake doc commandを見てみよう．多くの文書化されたcodeを簡単に眺めることができる．

### Output of `make doc`

![make doc](doc/make_doc.png)

[Visual Studio Code]: https://code.visualstudio.com
[Rust Analyzer]: https://rust-analyzer.github.io

## ? System Requirements

The tutorials are primarily targeted at **Linux**-based distributions. Most stuff will also work on **macOS**, but this is only _experimental_.

この解説は主にLinux-based distributionsを対象とします．ほとんどの要素はmaxOSなどのUnix風OSでも動きますが，実験的なものです．

### ? The tl;dr Version
### 要約版

1. [Install Docker][install_docker].
1. Dockerを導入
1. Ensure your user account is in the [docker group].
1. アカウントがdocker groupにあることを確認してください．
1. [Install Docker Desktop][install_docker].
1. (**Linux only**) Ensure your user account is in the [docker group].
1. Prepare the `Rust` toolchain. Most of it will be handled on first use through the
   [rust-toolchain](rust-toolchain) file. What's left for us to do is:
   1. If you already have a version of Rust installed:
      ```bash
      cargo install cargo-binutils rustfilt
      ```

   1. If you need to install Rust from scratch:
      ```bash
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

      source $HOME/.cargo/env
      cargo install cargo-binutils rustfilt
      ```
1. Rust製品群を用意しましょう．そのほとんどはrust製品群の最初の使用で扱われます．すべきことは以下の通り．
   1. Rustのある版が既に入っている場合
      ```bash
      cargo install cargo-binutils rustfilt
      ```
   1. 最初にRustを入れる必要がある場合
      ```bash
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

      source $HOME/.cargo/env
      cargo install cargo-binutils rustfilt
      ```

1. In case you use `Visual Studio Code`, I strongly recommend installing the [Rust Analyzer extension].
1. Visual Studio Codeを使用する場合，Rust Analyzer extensionの導入を強く勧める．
1. If you are **NOT** running Linux, some `Ruby` gems are needed as well:
1. (**macOS only**) Install a few `Ruby` gems.

   Run this in the repository root folder:

   ```bash
   bundle install --path .vendor/bundle --without development
   ```
1. Linuxを動かさない場合，何かしらのRubyのgem(Rubyのライブラリ)が必要になります．:w
   ```bash
   sudo gem install bundler
   bundle config set path '.vendor/bundle'
   bundle install
   ```

[docker group]: https://docs.docker.com/engine/install/linux-postinstall/
[Rust Analyzer extension]: https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer

### ? More Details: Eliminating Toolchain Hassle
### 詳細:製品群を除去すると困ること

This series tries to put a strong focus on user friendliness. Therefore, efforts were made to
eliminate the biggest painpoint in embedded development as much as possible: `Toolchain hassle`.

このseriesはやりやすさに焦点を当ててみます．故に，組み込み開発の最も大変な個所をできるだけ除くための努力がなされる．

Rust itself is already helping a lot in that regard, because it has built-in support for
cross-compilation. All that we need for cross-compiling from an `x86` host to the Raspberry Pi's
`AArch64` architecture will be automatically installed by `rustup`. However, besides the Rust
compiler, we will use some more tools. Among others:

この点に関してRustはcross-compilationのbuilt-in supportを持っており，すでに多くの支援をしている．x86 hostからRaspberry PiのAArch64へのcross-compilingに必要なものはrustupによって自動的に導入される．しかし，Rust compilerに加えて，以下のような製品を使う．

- `QEMU` to emulate our kernel on the host system.
- A self-made tool called `Minipush` to load a kernel onto the Raspberry Pi on-demand over `UART`.
- `OpenOCD` and `GDB` for debugging on the target.

- host system上で作成するkernelを模倣するためのQEMU
- UARTの要求に従ってkernelをRaspberry Piに読み込むためのMinipushと呼ばれる自作製品
- 対象をdebugするためのOpenOCDとGDB

There is a lot that can go wrong while installing and/or compiling the correct version of each tool
on your host machine. For example, your distribution might not provide the latest version that is
needed. Or you are missing some hard-to-get dependencies for the compilation of one of these tools.

host machineへの各製品の正しい版のinstallingやcompilingで多くの間違えうる箇所がある．例えば，distributionは必要な製品の最新版を提供しないかもしれない．若しくはこれらの製品のcompilationのための何らかの依存関係を失うかもしれない．

This is why we will make use of [Docker][install_docker] whenever possible. We are providing an
accompanying container that has all the needed tools or dependencies pre-installed, and it gets
pulled in automagically once it is needed. If you want to know more about Docker and peek at the
provided container, please refer to the repository's [docker](docker) folder.

これができる限りDockerを使う理由だ．必要な製品と依存関係を事前に導入した付属のcontainerを提供する．これは一度自動的に押し込まれる．Dockerとこのcontainerについての詳細は，docker folderを参照してください．

[install_docker]: https://docs.docker.com/get-docker/

## ? USB Serial Output
## USB Serial出力

Since the kernel developed in the tutorials runs on the real hardware, it is highly recommended to
get a USB serial cable to get the full experience.

このtutorialsで開発されるkernelは実機上で動くため，経験のためUSB serial cableが勧められる．

- You can find USB-to-serial cables that should work right away at [\[1\]] [\[2\]], but many others
  will work too. Ideally, your cable is based on the `CP2102` chip.
- [\[1\]] [\[2\]]にUSBからserialへのすぐに動作するcablesがあるが，多くの他の物も同様に動作する．cableはCP2102 chipに準拠していることが望ましい．
- You connect it to `GND` and GPIO pins `14/15` as shown below.
- 以下に示すようにcableをGNDとGPIO pins `14/15`に繋げる．
- [Tutorial 5](05_drivers_gpio_uart) is the first where you can use it. Check it out for
  instructions on how to prepare the SD card to boot your self-made kernel from it.
- これは[Tutorial 5](05_drivers_gpio_uart)で最初に使われる．自分で作ったkernelを起動するためにSD cardを準備する命令を確認しよう．
- Starting with [tutorial 6](06_uart_chainloader), booting kernels on your Raspberry is getting
  _really_ comfortable. In this tutorial, a so-called `chainloader` is developed, which will be the
  last file you need to manually copy on the SD card for a while. It will enable you to load the
  tutorial kernels during boot on demand over `UART`.
- [tutorial 6](06_uart_chainloader)からは，Raspberry上でのkernelsの起動が快適になる．この項目で，手動でSD cardにcopyする最後のfileとなるいわゆる`chainloader`が開発される．起動している間`UART`経由の要求でtutorial kernelsを読み込めるようになる．

![UART wiring diagram](doc/wiring.png)

[\[1\]]: https://www.amazon.de/dp/B0757FQ5CX/ref=cm_sw_r_tw_dp_U_x_ozGRDbVTJAG4Q
[\[2\]]: https://www.adafruit.com/product/954

## ? Acknowledgements
## 謝辞

The original version of the tutorials started out as a fork of [Zoltan
Baldaszti](https://github.com/bztsrc)'s awesome [tutorials on bare metal programming on
RPi3](https://github.com/bztsrc/raspi3-tutorial) in `C`. Thanks for giving me a head start!

このtutorialsは[Zoltan Baldaszti](https://github.com/bztsrc)によるC言語を使った[tutorials on bare metal programming on RPi3](https://github.com/bztsrc/raspi3-tutorial)をもとにしている．有益な種を下さったことに感謝申し上げます．

## License
## 利用許諾条件

Licensed under either of
以下の2つのlicenseと利用者の付加条件により条件付けされます．

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


### Contribution
### 貢献

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

明確に異なる立場をとらない限り，Apache-2.0 licenseに定義されるように任意の貢献は意図的に提示され，追加の条件や制約なしに上記の通り二重に条件付けされる．


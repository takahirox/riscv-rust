# riscv-rust

riscv-rust is a [RISC-V](https://riscv.org/) processor emulator project written in Rust.

## Demo

You can run Linux or xv6 on the emulator in your browser. [Online demo is here](https://takahirox.github.io/riscv-rust/wasm/web/index.html)

## Screenshots

![animation](./screenshots/animation.gif)

## Document

https://docs.rs/riscv_emu_rust/0.1.0/riscv_emu_rust/

## Features

- Emulate RISC-V processor and peripheral devices
- Stable as [Linux](https://risc-v-getting-started-guide.readthedocs.io/en/latest/linux-qemu.html) and [xv6-riscv](https://github.com/mit-pdos/xv6-riscv) run on it
- Runnable locally
- Also runnable in browser with WebAssembly

## Instructions/Features support status

- [x] RV32/64I
- [x] RV32/64M
- [x] RV32/64F (partially)
- [x] RV32/64D (partially)
- [ ] RV32/64Q
- [x] RV32/64A (partially)
- [x] RV64C/32C (partially)
- [x] RV32/64Zifencei (partially)
- [x] RV32/64Zicsr (partially)
- [x] CSR (partially)
- [x] SV32/39
- [ ] SV48
- [x] Privileged instructions (partially)
- [ ] PMP

etc...


## How to import into your Rust project

This module is released at [crates.io](https://crates.io/crates/riscv_emu_rust
). Add the following line into Cargo.toml of your Rust project.

```
[dependencies]
riscv_emu_rust = "0.1.0"
```

Refer to [cli/src/main.rs](https://github.com/takahirox/riscv-rust/blob/master/cli/src/main.rs) as sample code.

## How to build core library

```sh
$ git clone https://github.com/takahirox/riscv-rust.git
$ cd riscv-rust
$ cargo build --release
```

## How to run Linux or xv6 as CLI application

```sh
$ cd riscv-rust/cli
# Run Linux
$ cargo run --release ../resources/linux/bbl -f ../resources/linux/busybear.bin -d ../resources/linux/dtb.dtb
# Run xv6
$ cargo run --release ../resources/xv6/kernel -f ../resources/xv6/fs.img
```

## How to build WebAssembly and run in the browser

Prerequirements
- Install [wasm-bindgen client](https://rustwasm.github.io/docs/wasm-bindgen/)

```sh
$ cd riscv-rust/wasm
$ bash build.sh
# boot local server and access riscv-rust/wasm/web/index.html
```

## How to install WebAssembly npm package

```sh
$ npm install riscv_emu_rust_wasm
```

## How to run riscv-tests

Prerequirements
- Install [riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain)
- Install [riscv-tests](https://github.com/riscv/riscv-tests)

```sh
$ cd riscv-rust/cli
$ cargo run $path_to_riscv_tets/isa/rv32ui-p-add -n
```

## Links

### Linux RISC-V port

[Linux RISC-V port](https://risc-v-getting-started-guide.readthedocs.io/en/latest/linux-qemu.html)

### xv6-riscv

[xv6-riscv](https://github.com/mit-pdos/xv6-riscv) is the RISC-V port of [xv6](https://pdos.csail.mit.edu/6.828/2019/xv6.html) which is UNIX V6 rewritten by MIT for x86 in the current C language.

### Specifications

- [RISC-V ISA](https://riscv.org/specifications/)
- [Virtio Device](https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html)
- [UART](http://www.ti.com/lit/ug/sprugp1/sprugp1.pdf)
- [CLINT, PLIC (SiFive E31 Manual)](https://sifive.cdn.prismic.io/sifive%2Fc89f6e5a-cf9e-44c3-a3db-04420702dcc1_sifive+e31+manual+v19.08.pdf)
- [SiFive Interrupt Cookbook](https://sifive.cdn.prismic.io/sifive/0d163928-2128-42be-a75a-464df65e04e0_sifive-interrupt-cookbook.pdf)

# riscv-rust

riscv-rust is a [RISC-V](https://riscv.org/) processor emulator project written in Rust.

## Demo

[Online demo with Linux and xv6-riscv](https://takahirox.github.io/riscv-rust/index.html)

## Features

- Emulate RISC-V processor and peripheral devices
- Stable as [Linux](https://risc-v-getting-started-guide.readthedocs.io/en/latest/linux-qemu.html) and [xv6-riscv](https://github.com/mit-pdos/xv6-riscv) run on it
- Runnable locally
- Also runnable on browser with WebAssembly

## Screenshots

![animation](./screenshots/animation.gif)

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

## How to build riscv-rust and run Linux or xv6

### Standalone

```sh
$ git clone https://github.com/takahirox/riscv-rust.git
$ cd riscv-rust
# Run Linux
$ cargo run --release linux/bbl -f linux/busybear.bin -d linux/dtb.dtb
# Run xv6
$ cargo run --release xv6/kernel -f xv6/fs.img
```

### WebAssembly

Prerequirements
- Install [wasm-bindgen client](https://rustwasm.github.io/docs/wasm-bindgen/)

```sh
$ git clone https://github.com/takahirox/riscv-rust.git
$ cd riscv-rust
$ cargo build --release --lib --target wasm32-unknown-unknown
$ wasm-bindgen ./target/wasm32-unknown-unknown/release/riscv_rust.wasm --out-dir ./wasm/ --target web --no-typescript
# boot local server and access index.html
```

## How to build and run riscv-tests

Prerequirements
- Install [riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain)
- Install [riscv-tests](https://github.com/riscv/riscv-tests)

```sh
$ git clone https://github.com/takahirox/riscv-rust.git
$ cd riscv-rust
$ cargo run $path_to_riscv_tets/isa/rv32ui-p-add -n
```

## Links

### Linux RISC-V port

[Linux RISC-V port](https://risc-v-getting-started-guide.readthedocs.io/en/latest/linux-qemu.html)

### xv6-riscv

[xv6-riscv](https://github.com/mit-pdos/xv6-riscv) is the RISC-V port of [xv6](https://pdos.csail.mit.edu/6.828/2019/xv6.html) which is UNIX V6 rewritten by MIT for x86 in the current C language.

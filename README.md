# riscv-rust

riscv-rust is a [RISC-V](https://riscv.org/) processor emulator project written in Rust.

## Demo

[Online demo with xv6-riscv](https://takahirox.github.io/riscv-rust/index.html)

[xv6-riscv](https://github.com/mit-pdos/xv6-riscv) is the RISC-V port of [xv6](https://pdos.csail.mit.edu/6.828/2019/xv6.html) which is UNIX V6 rewritten by MIT for x86 in the current C language.

## Features

- Emulate RISC-V processor and peripheral devices
- Stable as [xv6-riscv](https://github.com/mit-pdos/xv6-riscv) runs on it
- Runnable locally
- Also runnable on browser with WebAssembly

## Screenshots

![animation](./screenshots/animation.gif)

## Instructions/Features support status

- [x] RV32/64I
- [x] RV32/64M
- [ ] RV32/64F
- [ ] RV32/64D
- [ ] RV32/64Q
- [x] RV32/64A (partially)
- [ ] RV32/64C
- [ ] RV32/64Zifencei
- [ ] RV32/64Zixsr
- [x] CSR
- [x] SV32
- [x] SV39
- [ ] SV48
- [x] Privileged instructions (partially)

etc...

## How to build riscv-rust and run xv6

### Standalone

```sh
$ git clone https://github.com/takahirox/riscv-rust.git
$ cd riscv-rust
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

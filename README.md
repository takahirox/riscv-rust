# riscv-rust

riscv-rust is a [RISC-V](https://riscv.org/) processor emulator project written in Rust

## Instruction support status

- [x] RV32/64I
- [x] RV32/64M
- [ ] RV32/64F
- [ ] RV32/64D
- [ ] RV32/64Q
- [ ] RV32/64A
- [ ] RV32/64C
- [ ] RV32/64Zifencei
- [ ] RV32/64Zixsr

etc...

## How to run test

Prerequirements
- Install [riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain)
- Install [riscv-tests](https://github.com/riscv/riscv-tests)

```sh
$ git clone https://github.com/takahirox/riscv-rust.git
$ cd riscv-rust
$ vi build_tests.sh # edit the paths to the installed toolchain and riscv-tests
$ bash build_test.sh
$ cargo run ./tests/rv32ui-p-add -x 32
$ cargo run ./tests/rv64ui-p-add -x 64
```

# riscv-rust

riscv-rust is a [RISC-V](https://riscv.org/) processor emulator project written in Rust

## Instruction support status

- [x] RV32I
- [ ] RV32M
- [ ] RV32F
- [ ] RV32D
- [ ] RV32Q
- [ ] RV32A
- [ ] RV32C
- [ ] RV32Zifencei
- [ ] RV32Zixsr
- [ ] RV64I
- [ ] RV64M
- [ ] RV64F
- [ ] RV64D
- [ ] RV64Q
- [ ] RV64A
- [ ] RV64C
- [ ] RV64Zifencei
- [ ] RV64Zixsr

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
$ cargo run ./tests/rv32ui-p-add
```

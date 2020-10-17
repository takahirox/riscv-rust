# How to build xv6 for riscv-rust

## Prerequirement

Install [riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain).

## Clone xv6-riscv

```sh
$ git clone https://github.com/mit-pdos/xv6-riscv.git
$ cd xv6-riscv
```

## Make kernel

```sh
$ vi Makefile
# Set higher optimization level.
# Replace this line
#   CFLAGS = -Wall -Werror -O -fno-omit-frame-pointer -ggdb
# with
#   CFLAGS = -Wall -Werror -O3 -fno-omit-frame-pointer -ggdb
$ make
```

## Make filesystem image

```sh
$ make fs.img
```

## Copy the files

```sh
$ cp kernel/kernel path_to_riscv-rust/resources/xv6/
$ cp fs.img path_to_riscv-rust/resources/xv6/
```

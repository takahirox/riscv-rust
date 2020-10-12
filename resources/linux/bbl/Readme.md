# How to build Linux + BBL for riscv-rust

## Prerequirement

Install [riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain).

## Make working directory

```sh
$ mkdir riscv64-linux
```

## Build Linux kernel

See https://github.com/takahirox/riscv-rust/tree/master/resources/linux/opensbi#build-linux-kernel

## Build BBL

```sh
$ cd riscv64-linux
$ git clone https://github.com/riscv/riscv-pk.git
$ cd riscv-pk
$ mkdir build
$ cd build
$ ../configure --enable-logo --host=riscv64-unknown-elf \
    --with-payload=../../linux/vmlinux
$ make -j $(nproc)
```

## Build busybox

See https://github.com/takahirox/riscv-rust/tree/master/resources/linux/opensbi#build-busybox

## Make root file system image

See https://github.com/takahirox/riscv-rust/tree/master/resources/linux/opensbi#make-root-file-system-image

## Copy the files

```sh
$ cd riscv64-linux
$ cp riscv-pk/build/bbl path_to_riscv-rust/resources/linux/bbl/
$ cp rootfs/rootfs.img path_to_riscv-rust/resources/linux/
```

## Appendix : Build QEMU

See https://github.com/takahirox/riscv-rust/tree/master/resources/linux/opensbi#appendix--build-qemu

```sh
$ cd riscv64-linux
$ qemu-system-riscv64 -nographic -machine virt \
    -kernel riscv-pk/build/bbl \
    -append "root=/dev/vda rw console=ttyS0" \
    -drive file=rootfs/rootfs.img,format=raw,id=hd0 \
    -device virtio-blk-device,drive=hd0
```

## References

- https://risc-v-getting-started-guide.readthedocs.io/en/latest/linux-qemu.html

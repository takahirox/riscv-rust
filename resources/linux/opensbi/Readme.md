# How to build Linux + OpenSBI for riscv-rust

## Prerequirement

Install [riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain).

## Make working directory

```sh
$ mkdir riscv64-linux
```

## Build Linux kernel

```sh
$ cd riscv64-linux
$ git clone https://github.com/torvalds/linux.git
$ cd linux
$ git checkout v5.4
$ make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- defconfig
$ make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- menuconfig
# Choose Platform type - Maximum Physical Memory (2GiB)
$ make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- -j $(nproc)
```

## Build OpenSBI

```sh
$ cd riscv64-linux
$ git clone https://github.com/riscv/opensbi.git
$ cd opensbi
$ git checkout v0.8
$ make CROSS_COMPILE=riscv64-unknown-elf- PLATFORM=generic \
    FW_PAYLOAD_PATH=../linux/arch/riscv/boot/Image
```

## Build Busybox

```sh
$ cd riscv64-linux
$ git clone https://github.com/mirror/busybox.git
$ cd busybox
$ git checkout 1_32_0
$ make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- defconfig
$ make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- menuconfig
# Check Settings - Build static binary (no shared libs)
$ make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- install
```

## Make root file system image

```sh
$ cd riscv64-linux
$ mkdir rootfs
$ cd rootfs
$ dd if=/dev/zero of=rootfs.img bs=1M count=50
$ mkfs.ext2 -L riscv-rootfs rootfs.img
$ sudo mkdir /mnt/rootfs
$ sudo mount rootfs.img /mnt/rootfs
$ sudo cp -ar ../busybox/_install/* /mnt/rootfs
$ sudo mkdir /mnt/rootfs/{dev,home,mnt,proc,sys,tmp,var}
$ sudo chown -R -h root:root /mnt/rootfs

# Check 
$ df /mnt/rootfs
Filesystem     1K-blocks  Used Available Use% Mounted on
/dev/loop5         49584  1704     45320   4% /mnt/rootfs
$ mount | grep rootfs
riscv64-linux/rootfs/rootfs.img on /mnt/rootfs type ext2 (rw,relatime)

# Clean up
$ sudo umount /mnt/rootfs
$ sudo rmdir /mnt/rootfs
```

## Copy the files

```sh
$ cd riscv64-linux
$ cp opensbi/build/platform/generic/firmware/fw_payload.elf path_to_riscv-rust/resources/linux/opensbi/
$ cp rootfs/rootfs.img path_to_riscv-rust/resources/linux/
```

## Appendix : Build QEMU

```sh
$ cd riscv64-linux
$ git clone https://github.com/qemu/qemu.git
$ cd qemu
$ git checkout v5.0.0
$ ./configure --target-list=riscv64-softmmu
$ make -j $(nproc)
$ sudo make install
```

## Appendix : Run with QEMU

```sh
$ cd riscv64-linux
$ qemu-system-riscv64 -nographic -machine virt \
    -kernel opensbi/build/platform/generic/firmware/fw_payload.elf \
    -append "root=/dev/vda rw console=ttyS0" \
    -drive file=rootfs/rootfs.img,format=raw,id=hd0 \
    -device virtio-blk-device,drive=hd0
```

## References

- https://risc-v-getting-started-guide.readthedocs.io/en/latest/linux-qemu.html
- https://github.com/UCanLinux/riscv64-sample#create-a-disk-with-50mb-capacity
- https://cstmize.hatenablog.jp/archive/2019/10/14 (in Japanese)

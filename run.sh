#!/bin/bash
set -xue

QEMU=qemu-system-riscv32
KERNEL=target/riscv32i-unknown-none-elf/release/kernel
USER=user/target/riscv32i-unknown-none-elf/release/user

echo "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In ut magna consequat, cursus velit aliquam, scelerisque odio. Ut lorem eros, feugiat quis bibendum vitae, malesuada ac orci. Praesent eget quam non nunc fringilla cursus imperdiet non tellus. Aenean dictum lobortis turpis, non interdum leo rhoncus sed. Cras in tellus auctor, faucibus tortor ut, maximus metus. Praesent placerat ut magna non tristique. Pellentesque at nunc quis dui tempor vulputate. Vestibulum vitae massa orci. Mauris et tellus quis risus sagittis placerat. Integer lorem leo, feugiat sed molestie non, viverra a tellus." > lorem.txt

(cd user && cargo build --release)
llvm-objcopy --set-section-flags .bss=alloc,contents -O binary $USER shell.bin
llvm-objcopy -Ibinary -Oelf32-littleriscv shell.bin shell.bin.o

cargo build --release

$QEMU -machine virt -bios default -nographic -serial mon:stdio --no-reboot \
    -d unimp,guest_errors,int,cpu_reset -D qemu.log \
    -drive id=drive0,file=lorem.txt,format=raw \
    -device virtio-blk-device,drive=drive0,bus=virtio-mmio-bus.0 \
    -kernel $KERNEL

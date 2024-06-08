#!/bin/bash
set -xue

QEMU=qemu-system-riscv32
KERNEL=target/riscv32i-unknown-none-elf/release/kernel
USER=user/target/riscv32i-unknown-none-elf/release/user

(cd user && cargo build --release)
llvm-objcopy --set-section-flags .bss=alloc,contents -O binary $USER shell.bin
llvm-objcopy -Ibinary -Oelf32-littleriscv shell.bin shell.bin.o

cargo build --release

$QEMU -machine virt -bios default -nographic -serial mon:stdio --no-reboot \
    -d unimp,guest_errors,int,cpu_reset -D qemu.log \
    -kernel $KERNEL

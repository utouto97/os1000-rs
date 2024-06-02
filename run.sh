#!/bin/bash
set -xue

QEMU=qemu-system-riscv32
KERNEL=target/riscv32i-unknown-none-elf/debug/kernel

cargo build

$QEMU -machine virt -bios default -nographic -serial mon:stdio --no-reboot \
    -d unimp,guest_errors,int,cpu_reset -D qemu.log \
    -kernel $KERNEL

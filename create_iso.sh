#!/bin/bash

# huggingOs ISO Creation Script

set -e

ISO_DIR="iso"
KERNEL_BIN="build/kernel.bin"
GRUB_CFG="boot/grub/grub.cfg"
ISO_NAME="huggingOs.iso"

echo "Creating ISO structure..."

# Create ISO directory structure
mkdir -p $ISO_DIR/boot/grub

# Copy kernel
if [ ! -f "$KERNEL_BIN" ]; then
    echo "Error: Kernel binary not found at $KERNEL_BIN"
    echo "Please build the kernel first: make all"
    exit 1
fi

cp $KERNEL_BIN $ISO_DIR/boot/kernel.bin

# Copy GRUB configuration
cp $GRUB_CFG $ISO_DIR/boot/grub/

# Create ISO
echo "Creating ISO image..."
grub-mkrescue -o $ISO_NAME $ISO_DIR

echo "ISO created: $ISO_NAME"






#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")"

echo "=========================================="
echo "  huggingOS Build and Run"
echo "=========================================="
echo ""

missing=0
for tool in nasm grub-mkrescue xorriso; do
    if command -v "$tool" >/dev/null 2>&1; then
        echo "  [OK] $tool found"
    else
        echo "  [MISSING] $tool"
        missing=1
    fi
done

if command -v i686-elf-gcc >/dev/null 2>&1; then
    echo "  [OK] i686-elf-gcc found"
elif command -v gcc >/dev/null 2>&1; then
    echo "  [OK] gcc found; Makefile will use -m32 fallback"
else
    echo "  [MISSING] gcc or i686-elf-gcc"
    missing=1
fi

if [ "$missing" -ne 0 ]; then
    echo ""
    echo "Install dependencies on Debian/Ubuntu/WSL with:"
    echo "  sudo apt update"
    echo "  sudo apt install -y build-essential gcc-multilib nasm grub-pc-bin grub-common xorriso qemu-system-x86"
    exit 1
fi

echo ""
echo "Building..."
make clean all iso

echo ""
if command -v qemu-system-i386 >/dev/null 2>&1; then
    echo "Starting QEMU..."
    qemu-system-i386 -cdrom huggingOs.iso -m 128M -vga std
else
    echo "Build complete: huggingOs.iso"
    echo "Install qemu-system-x86 or attach the ISO in VirtualBox."
fi

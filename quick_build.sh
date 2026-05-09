#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")"

echo "=== huggingOS Quick Build ==="
echo ""

if ! command -v nasm >/dev/null 2>&1 ||
   ! command -v grub-mkrescue >/dev/null 2>&1 ||
   ! command -v xorriso >/dev/null 2>&1; then
    echo "Missing build tools. On Debian/Ubuntu/WSL run:"
    echo "  sudo apt update"
    echo "  sudo apt install -y build-essential gcc-multilib nasm grub-pc-bin grub-common xorriso"
    exit 1
fi

if ! command -v i686-elf-gcc >/dev/null 2>&1 && ! command -v gcc >/dev/null 2>&1; then
    echo "Missing compiler. Install i686-elf-gcc or gcc with 32-bit support."
    exit 1
fi

make clean all iso

echo ""
echo "SUCCESS: huggingOs.iso created"
echo "Run with: make qemu"

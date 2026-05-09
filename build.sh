#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")"

echo "=========================================="
echo "  huggingOS Build Script"
echo "=========================================="
echo ""

check_tool() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "ERROR: $1 not found"
        return 1
    fi
    echo "  [OK] $1 found"
}

echo "Checking required tools..."
check_tool nasm
check_tool grub-mkrescue

if command -v i686-elf-gcc >/dev/null 2>&1; then
    echo "  [OK] i686-elf-gcc found"
else
    check_tool gcc
    echo "  [OK] using gcc -m32 fallback from Makefile"
fi

echo ""
echo "Building kernel and ISO..."
make clean all iso

echo ""
echo "Build complete: huggingOs.iso"
echo "Run with: make qemu"

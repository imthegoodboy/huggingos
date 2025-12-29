#!/bin/bash

# huggingOs Build Script

set -e

echo "=========================================="
echo "  huggingOs Build Script"
echo "=========================================="
echo ""

# Check for required tools
echo "Checking for required tools..."

check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "ERROR: $1 not found!"
        echo "Please install $1"
        exit 1
    fi
    echo "  ✓ $1 found"
}

check_tool "i686-elf-gcc"
check_tool "nasm"
check_tool "grub-mkrescue"

echo ""
echo "Building kernel..."
make clean
make all

echo ""
echo "Creating ISO..."
make iso

echo ""
echo "=========================================="
echo "  Build Complete!"
echo "=========================================="
echo ""
echo "ISO created: huggingOs.iso"
echo ""
echo "To test in VirtualBox:"
echo "  1. Open VirtualBox"
echo "  2. Create a new VM (Type: Other, Version: Other/Unknown)"
echo "  3. Set memory to 64MB or more"
echo "  4. Create a virtual hard disk (any size, won't be used)"
echo "  5. Go to Settings > Storage"
echo "  6. Add optical drive and select huggingOs.iso"
echo "  7. Start the VM"
echo ""







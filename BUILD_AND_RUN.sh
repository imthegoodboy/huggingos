#!/bin/bash

# huggingOs Build and Run Script
# This script will check dependencies, build the OS, and help you run it

echo "=========================================="
echo "  huggingOs Build and Run Script"
echo "=========================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Makefile" ]; then
    echo "Error: Makefile not found. Are you in the huggingOs directory?"
    exit 1
fi

# Check for required tools
echo "Checking for required tools..."
MISSING_TOOLS=0

if ! command -v nasm &> /dev/null; then
    echo "  [X] NASM not found"
    MISSING_TOOLS=1
else
    echo "  [✓] NASM found"
fi

if ! command -v i686-elf-gcc &> /dev/null; then
    echo "  [X] i686-elf-gcc not found (cross-compiler)"
    MISSING_TOOLS=1
else
    echo "  [✓] i686-elf-gcc found"
fi

if ! command -v grub-mkrescue &> /dev/null; then
    echo "  [X] grub-mkrescue not found"
    MISSING_TOOLS=1
else
    echo "  [✓] grub-mkrescue found"
fi

echo ""

if [ $MISSING_TOOLS -eq 1 ]; then
    echo "Some tools are missing. Installing..."
    echo ""
    
    sudo apt-get update
    sudo apt-get install -y build-essential nasm grub-pc-bin grub-common xorriso
    
    echo ""
    echo "For the cross-compiler (i686-elf-gcc), you need to build it."
    echo "See HOW_TO_RUN.md or README.md for instructions."
    echo ""
    echo "Or use this quick install script (takes 15-30 minutes):"
    echo "  wget https://raw.githubusercontent.com/lordmilko/i686-elf-tools/master/i686-elf-tools.sh"
    echo "  chmod +x i686-elf-tools.sh"
    echo "  sudo ./i686-elf-tools.sh"
    echo ""
    exit 1
fi

# Clean previous build
echo "Cleaning previous build..."
make clean

# Build kernel
echo ""
echo "Building kernel..."
make all

if [ $? -ne 0 ]; then
    echo ""
    echo "Build failed! Check the error messages above."
    exit 1
fi

# Create ISO
echo ""
echo "Creating ISO..."
make iso

if [ $? -ne 0 ]; then
    echo ""
    echo "ISO creation failed! Check the error messages above."
    exit 1
fi

echo ""
echo "=========================================="
echo "  Build Complete!"
echo "=========================================="
echo ""
echo "ISO created: huggingOs.iso"
echo ""
echo "To run in VirtualBox:"
echo "  1. Open VirtualBox"
echo "  2. Create new VM (32-bit, Other/Unknown)"
echo "  3. Attach huggingOs.iso in Storage settings"
echo "  4. Start the VM"
echo ""
echo "Or use QEMU:"
echo "  qemu-system-i386 -cdrom huggingOs.iso -m 128M"
echo ""



#!/bin/bash
# Quick build script - checks tools and builds the OS

cd /mnt/c/Users/parth/Desktop/myos 2>/dev/null || cd "$(dirname "$0")"

echo "=== huggingOs Quick Build ==="
echo ""

# Check for NASM
if ! command -v nasm &> /dev/null; then
    echo "Installing NASM..."
    sudo apt-get update && sudo apt-get install -y nasm
fi

# Check for GRUB
if ! command -v grub-mkrescue &> /dev/null; then
    echo "Installing GRUB..."
    sudo apt-get update && sudo apt-get install -y grub-pc-bin grub-common xorriso
fi

# Check for cross-compiler
if ! command -v i686-elf-gcc &> /dev/null; then
    echo ""
    echo "ERROR: i686-elf-gcc not found!"
    echo ""
    echo "You need to install the cross-compiler. Run:"
    echo "  ./INSTALL_TOOLS.sh"
    echo ""
    echo "Or install manually:"
    echo "  wget https://raw.githubusercontent.com/lordmilko/i686-elf-tools/master/i686-elf-tools.sh"
    echo "  chmod +x i686-elf-tools.sh"
    echo "  sudo ./i686-elf-tools.sh"
    exit 1
fi

# Build
echo "Building..."
make clean
make all && make iso

if [ $? -eq 0 ]; then
    echo ""
    echo "SUCCESS! ISO created: huggingOs.iso"
    echo ""
    echo "To run:"
    echo "  1. Open VirtualBox"
    echo "  2. Create VM (32-bit, Other/Unknown)"
    echo "  3. Attach huggingOs.iso"
    echo "  4. Start!"
else
    echo ""
    echo "Build failed! Check errors above."
    exit 1
fi



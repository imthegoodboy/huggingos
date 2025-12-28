#!/bin/bash

# Complete installation and build script for huggingOs

cd /mnt/c/Users/parth/Desktop/myos 2>/dev/null || cd "$(dirname "$0")"

echo "=========================================="
echo "  huggingOs - Install and Build"
echo "=========================================="
echo ""

# Step 1: Install basic tools
echo "Step 1: Installing basic build tools..."
sudo apt-get update
sudo apt-get install -y build-essential nasm grub-pc-bin grub-common xorriso

# Step 2: Check for cross-compiler
echo ""
echo "Step 2: Checking for cross-compiler..."
if command -v i686-elf-gcc &> /dev/null; then
    echo "  [✓] Cross-compiler found!"
else
    echo "  [X] Cross-compiler not found"
    echo ""
    echo "Installing cross-compiler (this takes 15-30 minutes)..."
    echo "Please wait..."
    
    # Try to download and install
    if [ ! -f /tmp/i686-elf-tools.sh ]; then
        wget -q https://raw.githubusercontent.com/lordmilko/i686-elf-tools/master/i686-elf-tools.sh -O /tmp/i686-elf-tools.sh
        chmod +x /tmp/i686-elf-tools.sh
    fi
    
    sudo /tmp/i686-elf-tools.sh
    
    # Add to PATH
    export PATH="/usr/local/i686-elf/bin:$PATH"
    echo 'export PATH="/usr/local/i686-elf/bin:$PATH"' >> ~/.bashrc
fi

# Step 3: Build
echo ""
echo "Step 3: Building the OS..."
make clean
make all

if [ $? -ne 0 ]; then
    echo ""
    echo "Build failed! Make sure cross-compiler is installed."
    exit 1
fi

# Step 4: Create ISO
echo ""
echo "Step 4: Creating ISO..."
make iso

if [ $? -ne 0 ]; then
    echo ""
    echo "ISO creation failed!"
    exit 1
fi

echo ""
echo "=========================================="
echo "  SUCCESS! OS Built!"
echo "=========================================="
echo ""
echo "ISO created: huggingOs.iso"
echo ""
echo "To run in VirtualBox:"
echo "  1. Open VirtualBox"
echo "  2. Create new VM:"
echo "     - Name: huggingOs"
echo "     - Type: Other"
echo "     - Version: Other/Unknown (32-bit)"
echo "     - Memory: 128 MB"
echo "  3. In Settings > Storage, attach huggingOs.iso"
echo "  4. Start the VM!"
echo ""



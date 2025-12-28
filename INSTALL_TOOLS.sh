#!/bin/bash

# Quick tool installation script for huggingOs

echo "Installing build tools for huggingOs..."
echo ""

sudo apt-get update
sudo apt-get install -y build-essential nasm grub-pc-bin grub-common xorriso qemu-system-x86

echo ""
echo "Tools installed!"
echo ""
echo "For the cross-compiler (i686-elf-gcc), you have two options:"
echo ""
echo "Option 1: Use pre-built script (recommended, faster):"
echo "  wget https://raw.githubusercontent.com/lordmilko/i686-elf-tools/master/i686-elf-tools.sh"
echo "  chmod +x i686-elf-tools.sh"
echo "  sudo ./i686-elf-tools.sh"
echo ""
echo "Option 2: Build from source (see HOW_TO_RUN.md)"
echo ""
echo "After installing the cross-compiler, run: ./BUILD_AND_RUN.sh"
echo ""



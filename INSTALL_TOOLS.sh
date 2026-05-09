#!/bin/bash

set -euo pipefail

echo "Installing build tools for huggingOS..."
echo ""

sudo apt-get update
sudo apt-get install -y build-essential gcc-multilib nasm grub-pc-bin grub-common xorriso qemu-system-x86

echo ""
echo "Tools installed."
echo "Build with: make clean all iso"
echo "Run with: make qemu"

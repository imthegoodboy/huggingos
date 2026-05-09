#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")"

echo "=========================================="
echo "  huggingOS Install and Build"
echo "=========================================="
echo ""

echo "Installing Debian/Ubuntu/WSL dependencies..."
sudo apt-get update
sudo apt-get install -y build-essential gcc-multilib nasm grub-pc-bin grub-common xorriso qemu-system-x86

echo ""
echo "Building huggingOS..."
make clean all iso

echo ""
echo "SUCCESS: huggingOs.iso created"
echo "Run with: make qemu"

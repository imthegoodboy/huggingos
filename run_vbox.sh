#!/bin/bash

# huggingOs VirtualBox Launcher Script

ISO_FILE="huggingOs.iso"
VM_NAME="huggingOs"
VM_MEMORY="128"

echo "huggingOs VirtualBox Launcher"
echo ""

# Check if ISO exists
if [ ! -f "$ISO_FILE" ]; then
    echo "Error: ISO file not found: $ISO_FILE"
    echo "Please build the ISO first: make iso"
    exit 1
fi

# Check if VirtualBox is installed
if ! command -v VBoxManage &> /dev/null; then
    echo "Error: VirtualBox not found!"
    echo "Please install VirtualBox to run the OS."
    exit 1
fi

# Check if VM exists
if VBoxManage showvminfo "$VM_NAME" &> /dev/null; then
    echo "VM '$VM_NAME' already exists."
    read -p "Delete and recreate? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        VBoxManage unregistervm "$VM_NAME" --delete 2>/dev/null || true
    else
        echo "Starting existing VM..."
        VBoxManage startvm "$VM_NAME"
        exit 0
    fi
fi

# Create VM
echo "Creating VirtualBox VM..."
VBoxManage createvm --name "$VM_NAME" --ostype "Other" --register

# Configure VM
VBoxManage modifyvm "$VM_NAME" --memory $VM_MEMORY
VBoxManage modifyvm "$VM_NAME" --vram 16
VBoxManage modifyvm "$VM_NAME" --graphicscontroller vboxsvga

# Create storage controller
VBoxManage storagectl "$VM_NAME" --name "IDE Controller" --add ide

# Attach ISO
VBoxManage storageattach "$VM_NAME" \
    --storagectl "IDE Controller" \
    --port 0 \
    --device 0 \
    --type dvddrive \
    --medium "$ISO_FILE"

echo ""
echo "VM created and configured!"
echo "Starting VM..."
VBoxManage startvm "$VM_NAME"



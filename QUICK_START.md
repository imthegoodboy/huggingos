# Quick Start Guide - huggingOs

This is a quick reference guide to get huggingOs up and running as fast as possible.

## Prerequisites Checklist

- [ ] Cross-compiler (i686-elf-gcc)
- [ ] NASM assembler
- [ ] GRUB tools (grub-mkrescue)
- [ ] VirtualBox (for testing)

## Quick Build & Test (Linux/macOS)

```bash
# 1. Build the kernel and create ISO
make clean
make iso

# 2. Run in VirtualBox (automated)
chmod +x run_vbox.sh
./run_vbox.sh
```

## Quick Build & Test (Windows with WSL)

```bash
# 1. Install tools in WSL
sudo apt-get update
sudo apt-get install build-essential nasm grub-pc-bin

# 2. Build and create ISO
make clean
make iso

# 3. The ISO will be in Windows file system
#    Copy huggingOs.iso to Windows and load in VirtualBox manually
```

## Manual VirtualBox Setup (3 minutes)

1. **Create VM:**
   - Type: Other / Other/Unknown (32-bit)
   - Memory: 128 MB
   - Create empty hard disk (any size)

2. **Attach ISO:**
   - Settings > Storage > Empty CD > Choose `huggingOs.iso`

3. **Start VM:**
   - Click Start
   - Wait for boot message

4. **Use the OS:**
   - Type commands in the terminal
   - Try: `help`, `info`, `echo Hello`

## Common Commands

Once the OS boots:
- `help` - Show commands
- `clear` - Clear screen  
- `info` - System info
- `version` - OS version
- `echo <text>` - Echo text

## Troubleshooting

**Build fails:**
- Install missing tools (see README.md)
- Check that i686-elf-gcc is in PATH

**ISO won't boot:**
- Rebuild: `make clean && make iso`
- Check VM settings (32-bit, enough memory)

**Keyboard not working:**
- Click inside VM window to focus
- Enable I/O APIC in VM settings

For detailed information, see README.md


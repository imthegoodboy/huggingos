# huggingOs - A Minimal Custom Operating System

Welcome to **huggingOs**, a minimal but fully functional custom operating system built from scratch. This OS demonstrates core operating system concepts including bootloading, kernel development, memory management, device drivers, and user interface.

## Features

- ✅ **Multiboot-compliant bootloader** (GRUB)
- ✅ **32-bit x86 kernel** with protected mode
- ✅ **Memory management** with heap allocator
- ✅ **VGA text mode graphics** with colorful terminal
- ✅ **Keyboard input** with PS/2 keyboard driver
- ✅ **Interactive shell** with 15+ commands
- ✅ **Clock and Calendar** utilities
- ✅ **Calculator** for basic math operations
- ✅ **Timer** functionality
- ✅ **Color customization** for terminal
- ✅ **ASCII art** banner display
- ✅ **Interrupt handling** (ISR/IRQ)
- ✅ **Global Descriptor Table** (GDT) setup
- ✅ **Extensible architecture** for future features

## System Requirements

### Build Requirements

To build huggingOs, you need the following tools installed:

1. **Cross-compiler toolchain** (i686-elf-gcc)
   - On Linux: Install from your package manager or build from source
   - On Windows: Use WSL, MSYS2, or Cygwin
   - On macOS: Install via Homebrew: `brew install i686-elf-gcc`

2. **NASM** (Netwide Assembler)
   - On Linux: `sudo apt-get install nasm` (Debian/Ubuntu) or `sudo yum install nasm` (RHEL/Fedora)
   - On Windows: Download from https://www.nasm.us/
   - On macOS: `brew install nasm`

3. **GRUB tools** (grub-mkrescue)
   - On Linux: `sudo apt-get install grub-pc-bin` or `sudo yum install grub2-tools`
   - On Windows: Install via WSL or use pre-built binaries
   - On macOS: `brew install grub`

4. **Make** build system
   - Usually pre-installed on Linux/macOS
   - On Windows: Use WSL or install via package manager

### Testing Requirements

- **VirtualBox** (recommended) - https://www.virtualbox.org/
- Alternative: QEMU - `sudo apt-get install qemu` or `brew install qemu`

## Building huggingOs

### Step 1: Install Build Tools

#### On Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install build-essential nasm grub-pc-bin grub-common
# Install cross-compiler
sudo apt-get install gcc-multilib g++-multilib
# Or build i686-elf-gcc from source (see below)
```

#### On macOS:
```bash
brew install nasm grub i686-elf-gcc
```

#### Building i686-elf-gcc (if not available via package manager):

1. Download and build Binutils:
```bash
wget https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.xz
tar xf binutils-2.40.tar.xz
cd binutils-2.40
./configure --target=i686-elf --prefix=/usr/local/i686-elf --disable-nls
make
sudo make install
```

2. Download and build GCC:
```bash
wget https://ftp.gnu.org/gnu/gcc/gcc-13.2.0/gcc-13.2.0.tar.xz
tar xf gcc-13.2.0.tar.xz
cd gcc-13.2.0
./configure --target=i686-elf --prefix=/usr/local/i686-elf --disable-nls \
    --enable-languages=c,c++ --without-headers
make all-gcc all-target-libgcc
sudo make install-gcc install-target-libgcc
```

Add to your PATH:
```bash
export PATH="/usr/local/i686-elf/bin:$PATH"
```

### Step 2: Build the Kernel

Using Make:
```bash
make clean
make all
```

Or use the build script:
```bash
chmod +x build.sh
./build.sh
```

This will compile all source files and create `build/kernel.bin`.

### Step 3: Create Bootable ISO

```bash
make iso
```

Or use the ISO script:
```bash
chmod +x create_iso.sh
./create_iso.sh
```

This creates `huggingOs.iso` in the current directory.

## Quick Start - Running huggingOs

### Option 1: Automated Build (Recommended)

1. **In Ubuntu/WSL, navigate to the project:**
   ```bash
   cd /mnt/c/Users/parth/Desktop/myos
   ```

2. **Run the automated build script:**
   ```bash
   ./INSTALL_AND_BUILD.sh
   ```
   
   This will:
   - Install all required tools (nasm, grub, etc.)
   - Install the cross-compiler (takes 15-30 minutes)
   - Build the OS
   - Create the bootable ISO

3. **Run in VirtualBox:**
   - Open VirtualBox
   - Click "New" to create a new VM
   - Name: `huggingOs`
   - Type: `Other`
   - Version: `Other/Unknown (32-bit)`
   - Memory: `128 MB` (or more)
   - Create a virtual hard disk (any size, won't be used)
   - Click "Settings" → "Storage"
   - Under "Controller: IDE", click on "Empty"
   - Click the CD icon 📀 and select "Choose a disk file"
   - Navigate to and select `huggingOs.iso`
   - Click "OK"
   - Click "Start" to boot the OS!

### Option 2: Manual Build

1. **Install build tools:**
   ```bash
   sudo apt-get update
   sudo apt-get install -y build-essential nasm grub-pc-bin grub-common xorriso
   ```

2. **Install cross-compiler:**
   ```bash
   wget https://raw.githubusercontent.com/lordmilko/i686-elf-tools/master/i686-elf-tools.sh
   chmod +x i686-elf-tools.sh
   sudo ./i686-elf-tools.sh
   ```
   (Takes 15-30 minutes)

3. **Build the OS:**
   ```bash
   make clean
   make all
   make iso
   ```

4. **Run in VirtualBox** (same as Option 1, step 3)

## Running huggingOs in VirtualBox

### Method 1: Using the Automated Script (Linux/macOS)

```bash
chmod +x run_vbox.sh
./run_vbox.sh
```

This script will:
- Check if the ISO exists
- Create a new VirtualBox VM named "huggingOs"
- Configure the VM with appropriate settings
- Attach the ISO and start the VM

### Method 2: Manual VirtualBox Setup

1. **Create a New Virtual Machine:**
   - Open VirtualBox
   - Click "New"
   - Name: `huggingOs`
   - Type: `Other`
   - Version: `Other/Unknown (32-bit)`
   - Click "Next"

2. **Configure Memory:**
   - Allocate at least **64 MB** of RAM (128 MB recommended)
   - Click "Next"

3. **Create Virtual Hard Disk:**
   - Select "Create a virtual hard disk now"
   - Click "Create"
   - Choose "VDI" format
   - Select "Dynamically allocated"
   - Size: Any size (we won't use it, but it's required)
   - Click "Create"

4. **Attach ISO:**
   - Select the VM and click "Settings"
   - Go to "Storage"
   - Under "Controller: IDE", click the empty CD icon
   - Click the CD icon on the right and select "Choose a disk file"
   - Navigate to and select `huggingOs.iso`
   - Click "OK"

5. **Configure Display:**
   - Go to "Display"
   - Video Memory: 16 MB
   - Graphics Controller: VBoxSVGA or VMSVGA
   - Click "OK"

6. **Start the VM:**
   - Select the VM and click "Start"
   - The OS should boot and display the welcome message

### Method 3: Using QEMU (Alternative)

```bash
qemu-system-i386 -cdrom huggingOs.iso -m 128M -vga std
```

## Using huggingOs

Once the OS boots, you'll see:

```
========================================
     Welcome to huggingOs v1.0.0
========================================

Initializing system...
  - Setting up GDT... [OK]
  - Setting up IDT... [OK]
  - Initializing memory... [OK]
  - Initializing keyboard... [OK]
  - Initializing graphics... [OK]
  - Starting shell... [OK]

System ready!

huggingOs>
```

### Available Commands

- `help` - Display list of available commands
- `clear` or `cls` - Clear the terminal screen
- `info` - Show system information
- `version` - Display OS version
- `echo <text>` - Echo text back to the terminal
- `reboot` - Reboot the system (currently halts)

### Example Session

```
huggingOs> help
huggingOs - Available Commands:
  help      - Show this help message
  clear     - Clear the screen
  info      - Show system information
  echo      - Echo text back
  version   - Show OS version
  reboot    - Reboot the system
  color     - Change terminal color
  calc      - Simple calculator
  banner    - Show ASCII art banner
  about     - About huggingOs
  clock     - Show current time
  calendar  - Show calendar
  date      - Show date and time
  timer     - Start a timer (seconds)

huggingOs> info
=== huggingOs System Information ===
OS Name: huggingOs
Version: 1.0.0
Architecture: x86 (32-bit)
Kernel: Monolithic
Memory: Available
Graphics: VGA Text Mode
Features: Terminal, Shell, Keyboard
Status: Operational

huggingOs> calc 25 * 4
Result: 100

huggingOs> color cyan
Color changed to cyan

huggingOs> banner
  _   _                   _         ___  ____
 | | | |_   _ _ __  _   _| |_ ___  / _ \/ ___|
...

huggingOs> timer 5
Timer set for 5 seconds
Counting down...
5... 4... 3... 2... 1...
Timer complete!
BEEP! BEEP! BEEP!
```

## Project Structure

```
huggingOs/
├── boot/
│   └── grub/
│       └── grub.cfg              # GRUB bootloader configuration
├── kernel/
│   ├── boot.asm                  # Kernel entry point (multiboot)
│   ├── kernel.c                  # Main kernel code
│   ├── kernel.h                  # Kernel headers and definitions
│   ├── gdt.asm/.c/.h             # Global Descriptor Table
│   ├── interrupts.asm/.c/.h      # Interrupt handlers
│   ├── linker.ld                 # Kernel linker script
│   ├── memory/
│   │   ├── heap.c                # Heap allocator
│   │   ├── paging.c              # Memory paging
│   │   └── memory.h
│   ├── drivers/
│   │   ├── keyboard.c            # PS/2 keyboard driver
│   │   ├── vga.c                 # VGA text mode driver
│   │   ├── vesa.c                # VESA framebuffer (framework)
│   │   └── drivers.h
│   ├── terminal/
│   │   ├── terminal.c            # Terminal emulator
│   │   ├── shell.c               # Command shell
│   │   └── terminal.h
│   └── lib/
│       ├── string.c              # String utilities
│       ├── stdio.c               # I/O functions
│       └── lib.h
├── build/                        # Compiled objects (generated)
├── iso/                          # ISO contents (generated)
├── Makefile                      # Build system
├── build.sh                      # Build script
├── create_iso.sh                 # ISO creation script
├── run_vbox.sh                   # VirtualBox launcher
└── README.md                     # This file
```

## Troubleshooting

### Build Issues

**Problem: "i686-elf-gcc: command not found"**
- Solution: Install the cross-compiler toolchain or add it to your PATH
- See "Building i686-elf-gcc" section above

**Problem: "nasm: command not found"**
- Solution: Install NASM from your package manager
- Linux: `sudo apt-get install nasm`
- macOS: `brew install nasm`

**Problem: "grub-mkrescue: command not found"**
- Solution: Install GRUB tools
- Linux: `sudo apt-get install grub-pc-bin`
- macOS: `brew install grub`

**Problem: Linker errors about undefined references**
- Solution: Ensure all source files are included in the Makefile
- Check that all function declarations are in header files

### Runtime Issues

**Problem: VM boots but shows only a black screen**
- Solution: Check that the kernel binary was created successfully
- Verify the ISO was created correctly: `file huggingOs.iso`
- Try increasing VM memory to 128 MB or more

**Problem: Keyboard input not working**
- Solution: Ensure keyboard interrupts are enabled
- Check VM settings: System > Motherboard > Enable I/O APIC
- Try clicking inside the VM window to focus it

**Problem: "Invalid multiboot magic number" error**
- Solution: The kernel wasn't loaded by GRUB correctly
- Rebuild the kernel and ISO
- Check that grub.cfg is correct

**Problem: System hangs during initialization**
- Solution: This may indicate an issue with interrupts or memory
- Check that all initialization functions return successfully
- Verify GDT and IDT are set up correctly

### VirtualBox Specific Issues

**Problem: VM crashes on startup**
- Solution: Try changing graphics controller (VBoxSVGA, VMSVGA, or VBoxVGA)
- Disable 3D acceleration
- Reduce allocated memory

**Problem: Cannot attach ISO**
- Solution: Ensure the ISO file exists and is not corrupted
- Try creating a new VM
- Check VirtualBox version (should be 6.0+)

## Extending huggingOs

The architecture is designed to be extensible. Here are some ideas for enhancements:

1. **File System Support**
   - Implement FAT12/FAT16/FAT32 driver
   - Add file I/O operations

2. **Multitasking**
   - Process scheduler
   - Task switching
   - System calls

3. **Enhanced Graphics**
   - Full VESA framebuffer implementation
   - Window manager
   - GUI framework

4. **Networking**
   - Ethernet driver
   - TCP/IP stack
   - Network utilities

5. **Device Drivers**
   - Mouse support
   - Sound card
   - USB support

6. **Advanced Features**
   - Virtual memory management
   - Memory protection
   - User mode / kernel mode separation

## Technical Details

- **Architecture**: x86 (32-bit)
- **Boot Method**: Multiboot (GRUB)
- **Memory Model**: Flat memory model with paging support
- **Graphics**: VGA text mode (80x25)
- **Input**: PS/2 keyboard
- **Interrupts**: PIC-based interrupt handling
- **Memory Management**: Simple heap allocator

## Color Scheme

huggingOs uses a vibrant color scheme:
- Background: Dark blue/black
- Primary Text: Light green/cyan
- Accents: Light blue, light green
- Errors: Red
- Success: Green

## Contributing

This is a learning project, but contributions are welcome! Areas for improvement:
- Code documentation
- Additional features
- Bug fixes
- Performance optimizations

## License

This project is provided as-is for educational purposes. Feel free to use, modify, and learn from it.

## Acknowledgments

Built with inspiration from:
- OSDev.org community
- Various OS development tutorials
- The Linux kernel (for structure reference)

## Resources

- [OSDev.org](https://wiki.osdev.org/) - Excellent OS development wiki
- [Multiboot Specification](https://www.gnu.org/software/grub/manual/multiboot/)
- [Intel IA-32 Architecture Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)

## Version History

- **v1.0.0** - Initial release
  - Basic kernel with bootloader
  - VGA text mode terminal
  - Keyboard input
  - Command shell
  - Memory management framework

---

**Happy coding! Enjoy exploring operating systems with huggingOs! 🚀**


# Complete VirtualBox Guide for huggingOs - Step by Step

This guide will walk you through everything from installing software to running huggingOs in VirtualBox.

## Part 1: Installing Required Software

### Step 1: Install VirtualBox

1. **Download VirtualBox:**
   - Go to https://www.virtualbox.org/wiki/Downloads
   - Download "Windows hosts" (since you're on Windows)
   - File will be named like `VirtualBox-7.x.x-xxxxx-Win.exe`

2. **Install VirtualBox:**
   - Run the installer
   - Click "Next" through all screens (default options are fine)
   - Click "Install"
   - When Windows asks for permission, click "Yes"
   - Wait for installation to complete
   - Click "Finish"

3. **Verify Installation:**
   - You should see VirtualBox Manager window open
   - If not, search for "VirtualBox" in Start menu

### Step 2: Install WSL (Windows Subsystem for Linux)

Since we need Linux tools to build the OS, we'll use WSL:

1. **Open PowerShell as Administrator:**
   - Press `Windows Key + X`
   - Select "Windows PowerShell (Admin)" or "Terminal (Admin)"
   - Click "Yes" when asked for permission

2. **Install WSL:**
   - Copy and paste this command:
   ```
   wsl --install
   ```
   - Press Enter
   - Wait for installation (this may take a few minutes)
   - You'll be asked to restart your computer - **Click "Y" and restart**

3. **After Restart:**
   - A terminal window will open asking you to create a username
   - Enter a username (like "yourname") and press Enter
   - Enter a password (you won't see it as you type) and press Enter
   - Confirm password and press Enter

4. **Update Ubuntu (inside WSL):**
   - In the Ubuntu terminal, type:
   ```bash
   sudo apt update && sudo apt upgrade -y
   ```
   - Press Enter
   - Enter your password if asked
   - Wait for updates to complete

### Step 3: Install Build Tools in WSL

1. **Open Ubuntu from Start Menu** (or type `wsl` in PowerShell)

2. **Install Required Tools:**
   Copy and paste this entire block:
   ```bash
   sudo apt-get update
   sudo apt-get install -y build-essential nasm grub-pc-bin grub-common xorriso qemu-system-x86
   ```

3. **Install Cross-Compiler:**
   We'll install a pre-built cross-compiler for easier setup:
   ```bash
   sudo apt-get install -y gcc-multilib g++-multilib
   ```
   
   If the above doesn't work, we'll build the cross-compiler (see Alternative Method below)

### Alternative: Building Cross-Compiler (if needed)

If the above doesn't work, we need to build the cross-compiler:

```bash
# Install prerequisites
sudo apt-get install -y build-essential bison flex libgmp3-dev libmpc-dev libmpfr-dev texinfo

# Create directory
mkdir -p ~/cross-compiler
cd ~/cross-compiler

# Download and build binutils
wget https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.xz
tar xf binutils-2.40.tar.xz
cd binutils-2.40
./configure --target=i686-elf --prefix=/usr/local/i686-elf --disable-nls --disable-werror
make -j$(nproc)
sudo make install
cd ..

# Download and build GCC
wget https://ftp.gnu.org/gnu/gcc/gcc-12.2.0/gcc-12.2.0.tar.xz
tar xf gcc-12.2.0.tar.xz
cd gcc-12.2.0
./configure --target=i686-elf --prefix=/usr/local/i686-elf --disable-nls --enable-languages=c,c++ --without-headers
make all-gcc all-target-libgcc -j$(nproc)
sudo make install-gcc install-target-libgcc

# Add to PATH
echo 'export PATH="/usr/local/i686-elf/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## Part 2: Building the OS

### Step 1: Navigate to Your Project

1. **In Ubuntu/WSL terminal, navigate to your project:**
   ```bash
   cd /mnt/c/Users/parth/Desktop/myos
   ```
   (This accesses your Windows Desktop from Linux)

2. **Verify files are there:**
   ```bash
   ls
   ```
   You should see folders like `kernel`, `boot`, `Makefile`, etc.

### Step 2: Build the Kernel

1. **Make the build script executable:**
   ```bash
   chmod +x build.sh
   chmod +x create_iso.sh
   ```

2. **Build the OS:**
   ```bash
   make clean
   make all
   ```

   If you see errors about `i686-elf-gcc` not found, see the cross-compiler installation above.

3. **Create the ISO:**
   ```bash
   make iso
   ```

   This should create a file called `huggingOs.iso` in your project directory.

4. **Verify the ISO was created:**
   ```bash
   ls -lh *.iso
   ```
   You should see `huggingOs.iso` (should be a few MB in size)

## Part 3: Setting Up VirtualBox

### Step 1: Create a New Virtual Machine

1. **Open VirtualBox** (from Start menu)

2. **Click the "New" button** (blue icon with star, or press Ctrl+N)

3. **Fill in the details:**
   - **Name**: `huggingOs`
   - **Machine Folder**: Leave default (usually `C:\Users\YourName\VirtualBox VMs`)
   - **Type**: Click dropdown, select **"Other"**
   - **Version**: Click dropdown, select **"Other/Unknown (32-bit)"**
   - Click **"Next"**

4. **Memory Size:**
   - Move the slider to **128 MB** (or type 128)
   - This is enough for our OS
   - Click **"Next"**

5. **Hard Disk:**
   - Select **"Create a virtual hard disk now"**
   - Click **"Create"**

6. **Hard Disk File Type:**
   - Select **"VDI (VirtualBox Disk Image)"**
   - Click **"Next"**

7. **Storage on Physical Hard Disk:**
   - Select **"Dynamically allocated"**
   - Click **"Next"**

8. **File Location and Size:**
   - Leave default location
   - Size: **1 GB** is fine (we won't actually use it)
   - Click **"Create"**

### Step 2: Configure the Virtual Machine

1. **Select your VM** (huggingOs) in the list

2. **Click "Settings"** (gear icon, or press Ctrl+S)

3. **System Settings:**
   - Go to **"System"** tab (should be open by default)
   - **Motherboard tab:**
     - Boot Order: Make sure **"Optical"** is first (if not, select it and click up arrow)
   - **Processor tab:**
     - Processors: **1** is fine
   - **Acceleration tab:**
     - Leave defaults

4. **Display Settings:**
   - Go to **"Display"** tab
   - **Screen tab:**
     - Video Memory: **16 MB** (use slider or type)
     - Graphics Controller: Select **"VBoxSVGA"** from dropdown
   - **Remote Desktop:** Leave disabled

5. **Storage Settings:**
   - Go to **"Storage"** tab
   - Under **"Controller: IDE"**, you'll see **"Empty"**
   - Click on **"Empty"**
   - On the right side, click the **CD icon** 📀
   - Select **"Choose a disk file..."**
   - Navigate to: `C:\Users\parth\Desktop\myos\huggingOs.iso`
   - Select `huggingOs.iso`
   - Click **"Open"**
   - You should now see `huggingOs.iso` listed

6. **Click "OK"** to save settings

### Step 3: Start the Virtual Machine

1. **Select huggingOs** in the VirtualBox list

2. **Click the green "Start" arrow** (or press Enter)

3. **First time warning:**
   - VirtualBox may warn about mouse/keyboard capture
   - Click **"OK"**

4. **The VM window should open** and you should see:
   - Black screen briefly
   - Then GRUB menu (or it may boot directly)
   - Then the huggingOs welcome screen!

5. **If you see the welcome screen**, you're successful! 🎉

## Part 4: Using the OS

### Basic Interaction

1. **The screen shows:**
   ```
   ========================================
        Welcome to huggingOs v1.0.0
   ========================================
   
   Initializing system...
     - Setting up GDT... [OK]
     - Setting up IDT... [OK]
     ...
   ```

2. **Wait for the prompt:**
   ```
   huggingOs>
   ```

3. **Try commands:**
   - Type `help` and press Enter
   - Type `info` and press Enter
   - Type `echo Hello World` and press Enter
   - Type `clear` and press Enter

### Important Notes

- **To release mouse/keyboard** from VM: Press **Right Ctrl** key
- **To get mouse/keyboard back**: Click inside the VM window
- **To close VM**: Click the X button, then choose "Power off the machine"

## Part 5: Troubleshooting

### Problem: "i686-elf-gcc: command not found"

**Solution:**
- Install cross-compiler (see Part 1, Step 3 - Alternative Method)
- Or use a pre-built cross-compiler:
  ```bash
  # Quick alternative: Use regular gcc with specific flags
  # We'll need to modify Makefile (see below)
  ```

### Problem: "grub-mkrescue: command not found"

**Solution:**
```bash
sudo apt-get install -y grub-pc-bin grub-common xorriso
```

### Problem: ISO created but VM shows black screen

**Solutions:**
1. **Check VM settings:**
   - Make sure it's 32-bit (not 64-bit)
   - Make sure ISO is attached in Storage settings

2. **Rebuild the ISO:**
   ```bash
   make clean
   make iso
   ```

3. **Check VM logs:**
   - In VirtualBox, go to Machine > Show Log
   - Look for error messages

### Problem: Keyboard doesn't work in VM

**Solutions:**
1. Click inside the VM window to focus it
2. In VM Settings > System > Motherboard, enable "Enable I/O APIC"
3. Try pressing Right Ctrl to release, then click back in

### Problem: Build errors

**Common fixes:**
```bash
# Make sure you're in the project directory
cd /mnt/c/Users/parth/Desktop/myos

# Clean and rebuild
make clean
make all

# If you see specific error messages, note them and we can fix them
```

## Part 6: Quick Reference

### Building Commands
```bash
cd /mnt/c/Users/parth/Desktop/myos  # Navigate to project
make clean                          # Clean previous builds
make all                            # Build kernel
make iso                            # Create ISO
```

### VirtualBox Shortcuts
- **Start VM**: Select VM, press Enter or click Start
- **Release Mouse**: Right Ctrl
- **Full Screen**: Right Ctrl + F
- **Close VM**: Right Ctrl + Q (or click X, Power off)

### OS Commands
- `help` - Show all commands
- `clear` - Clear screen
- `info` - System information
- `version` - OS version
- `echo <text>` - Echo text
- `reboot` - Reboot (currently halts)

## Need More Help?

If you encounter any errors:
1. Note the exact error message
2. Check which step you were on
3. Review the troubleshooting section above
4. Try the suggested solutions

The OS is designed to be simple but functional. Enjoy exploring operating systems! 🚀


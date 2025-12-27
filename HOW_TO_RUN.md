# How to Run huggingOs - Complete Beginner's Guide

Welcome! This guide will teach you **everything** from scratch to run huggingOs in VirtualBox.

## 🎯 What You'll Need

1. **A Windows computer** (you're already using one!)
2. **Internet connection** (to download software)
3. **About 30-45 minutes** (first time setup)
4. **Basic computer skills** (pointing, clicking, typing)

## 📋 Step-by-Step Instructions

### STEP 1: Install VirtualBox (15 minutes)

VirtualBox lets you run another "computer" inside your computer safely.

1. **Open your web browser** (Chrome, Edge, Firefox, etc.)

2. **Go to this website:**
   ```
   https://www.virtualbox.org/wiki/Downloads
   ```

3. **Find "Windows hosts"** section (should be near the top)

4. **Click the big blue download button** that says something like:
   ```
   VirtualBox 7.x.x platform packages
   Windows hosts
   ```

5. **Wait for download to finish** (file will be in your Downloads folder)

6. **Find the downloaded file:**
   - Press `Windows Key + E` to open File Explorer
   - Click "Downloads" on the left
   - Find file named `VirtualBox-7.x.x-xxxxx-Win.exe`

7. **Double-click the file** to start installation

8. **Click through the installer:**
   - Click "Next" on the welcome screen
   - Click "Next" (keep default installation location)
   - Click "Next" (keep default components)
   - Click "Yes" if Windows asks for permission
   - Click "Install"
   - Wait for installation (may take 5-10 minutes)
   - **IMPORTANT:** Uncheck "Start Oracle VM VirtualBox" if checked
   - Click "Finish"

9. **VirtualBox is now installed!** ✅

---

### STEP 2: Install WSL (Windows Subsystem for Linux) (10 minutes)

We need Linux tools to build the OS. WSL gives us Linux inside Windows.

1. **Open PowerShell as Administrator:**
   - Press `Windows Key` on your keyboard
   - Type: `powershell`
   - **Right-click** on "Windows PowerShell"
   - Select **"Run as administrator"**
   - Click "Yes" when asked for permission

2. **Copy and paste this command** (press `Ctrl + V` after copying):
   ```powershell
   wsl --install
   ```

3. **Press Enter**

4. **Wait for installation** (2-5 minutes)

5. **When it says "Press any key to restart"**:
   - **Save any open work**
   - Press any key
   - Your computer will restart

6. **After computer restarts:**
   - A black terminal window will open
   - It will ask you to create a username
   - **Type a username** (like: `john` or `yourname`) and press Enter
   - **Type a password** (you won't see it - this is normal!) and press Enter
   - **Type the password again** to confirm, press Enter

7. **WSL is now installed!** ✅

---

### STEP 3: Install Build Tools (5 minutes)

Now we install the programs needed to build the OS.

1. **Open Ubuntu** (the terminal that opened, or search "Ubuntu" in Start menu)

2. **Copy and paste this command** (do it ONE LINE at a time, press Enter after each):

   First command:
   ```bash
   sudo apt-get update
   ```
   Press Enter, wait for it to finish

   Second command:
   ```bash
   sudo apt-get install -y build-essential nasm grub-pc-bin grub-common xorriso
   ```
   Press Enter, type your password when asked, wait for installation

3. **Wait for installation to complete** (may take 5-10 minutes)

4. **Build tools are installed!** ✅

---

### STEP 4: Install Cross-Compiler (10-15 minutes)

This is a special compiler for building operating systems.

**Option A: Quick Method (if available)**

Try this first:
```bash
sudo apt-get install -y gcc-multilib g++-multilib
```

If that works, skip to Step 5!

**Option B: Build from Source (if Option A doesn't work)**

Copy and paste these commands one by one:

```bash
sudo apt-get install -y build-essential bison flex libgmp3-dev libmpc-dev libmpfr-dev texinfo
```

```bash
mkdir -p ~/cross-compiler && cd ~/cross-compiler
```

```bash
wget https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.xz
```

```bash
tar xf binutils-2.40.tar.xz && cd binutils-2.40
```

```bash
./configure --target=i686-elf --prefix=/usr/local/i686-elf --disable-nls --disable-werror
```

```bash
make -j$(nproc)
```

```bash
sudo make install
```

```bash
cd ..
```

```bash
wget https://ftp.gnu.org/gnu/gcc/gcc-12.2.0/gcc-12.2.0.tar.xz
```

```bash
tar xf gcc-12.2.0.tar.xz && cd gcc-12.2.0
```

```bash
./configure --target=i686-elf --prefix=/usr/local/i686-elf --disable-nls --enable-languages=c,c++ --without-headers
```

```bash
make all-gcc all-target-libgcc -j$(nproc)
```

```bash
sudo make install-gcc install-target-libgcc
```

```bash
echo 'export PATH="/usr/local/i686-elf/bin:$PATH"' >> ~/.bashrc
```

```bash
source ~/.bashrc
```

**Note:** The last two commands (make) will take 10-30 minutes. Be patient!

---

### STEP 5: Build the OS (2 minutes)

1. **Navigate to your project folder:**
   ```bash
   cd /mnt/c/Users/parth/Desktop/myos
   ```

2. **Make scripts executable:**
   ```bash
   chmod +x build.sh create_iso.sh
   ```

3. **Clean previous builds:**
   ```bash
   make clean
   ```

4. **Build the kernel:**
   ```bash
   make all
   ```

   **If you see an error about `i686-elf-gcc`**, go back to Step 4 and make sure the cross-compiler is installed and in your PATH.

5. **Create the ISO:**
   ```bash
   make iso
   ```

6. **Check if ISO was created:**
   ```bash
   ls -lh *.iso
   ```

   You should see `huggingOs.iso` with a size like "1.5M" or similar.

7. **OS is built!** ✅

---

### STEP 6: Set Up VirtualBox VM (5 minutes)

1. **Open VirtualBox** (search for it in Start menu)

2. **Click "New" button** (big blue star icon, or press `Ctrl + N`)

3. **Fill in the form:**
   - **Name:** Type `huggingOs`
   - **Type:** Click the dropdown, select **"Other"**
   - **Version:** Click the dropdown, select **"Other/Unknown (32-bit)"**
   - Click **"Next"**

4. **Memory:**
   - Move slider to **128 MB** (or type `128`)
   - Click **"Next"**

5. **Hard disk:**
   - Select **"Create a virtual hard disk now"**
   - Click **"Create"**

6. **Hard disk type:**
   - Select **"VDI"**
   - Click **"Next"**

7. **Storage:**
   - Select **"Dynamically allocated"**
   - Click **"Next"**

8. **Size:**
   - Leave default (1 GB is fine)
   - Click **"Create"**

9. **VM is created!** ✅

---

### STEP 7: Configure the VM (3 minutes)

1. **Click on "huggingOs"** in the list (left side of VirtualBox)

2. **Click "Settings"** (gear icon, or press `Ctrl + S`)

3. **Storage settings:**
   - Click **"Storage"** tab (left side)
   - Under "Controller: IDE", click on **"Empty"**
   - On the right side, click the **CD icon** 📀
   - Click **"Choose a disk file..."**
   - Navigate to: `C:\Users\parth\Desktop\myos\`
   - Select **`huggingOs.iso`**
   - Click **"Open"**

4. **Display settings:**
   - Click **"Display"** tab (left side)
   - Video Memory: Set to **16 MB** (use slider)
   - Graphics Controller: Select **"VBoxSVGA"** from dropdown
   - Click **"OK"**

5. **Configuration complete!** ✅

---

### STEP 8: Run the OS! (1 minute)

1. **Select "huggingOs"** in VirtualBox list

2. **Click the green "Start" arrow** (or press `Enter`)

3. **A new window will open** - this is your virtual machine!

4. **You should see:**
   - Black screen briefly
   - Then colorful text:
     ```
     ========================================
          Welcome to huggingOs v1.0.0
     ========================================
     
     Initializing system...
       - Setting up GDT... [OK]
       ...
     ```

5. **Wait for the prompt:**
   ```
   huggingOs>
   ```

6. **Try typing commands:**
   - Type `help` and press Enter
   - Type `info` and press Enter
   - Type `echo Hello World` and press Enter

7. **🎉 SUCCESS! Your OS is running!** 🎉

---

## 🎮 How to Use the OS

### Basic Commands

Once you see `huggingOs>`, you can type:

- `help` - Show all available commands
- `info` - Show system information
- `version` - Show OS version
- `clear` - Clear the screen
- `echo <text>` - Print text (example: `echo Hello`)
- `reboot` - Reboot the system

### Important Keyboard Shortcuts

- **Right Ctrl** - Release mouse/keyboard from VM (so you can use your computer normally)
- **Click inside VM window** - Get mouse/keyboard back to VM
- **Right Ctrl + F** - Make VM full screen
- **Right Ctrl + Q** - Close VM (or click X button)

---

## ❓ Troubleshooting

### "i686-elf-gcc: command not found"

**Solution:**
- Go back to Step 4 and install the cross-compiler
- Make sure you ran: `source ~/.bashrc` after installation
- Or restart Ubuntu terminal

### "grub-mkrescue: command not found"

**Solution:**
- Run: `sudo apt-get install -y grub-pc-bin grub-common xorriso`
- Then try building again

### Black screen in VM

**Check:**
1. Is the ISO file attached? (Settings > Storage)
2. Is VM set to 32-bit? (Settings > General > Version)
3. Try rebuilding: `make clean && make iso`

### Keyboard doesn't work

**Solutions:**
1. Click inside the VM window (to focus it)
2. Try pressing Right Ctrl, then click back in
3. In VM Settings > System > Motherboard, enable "Enable I/O APIC"

### Build errors

**Try:**
```bash
make clean
make all
```

If you see specific error messages, note them down and we can help fix them!

---

## 🎓 What's Next?

Now that your OS is running:

1. **Explore the commands** - Try them all!
2. **Read the code** - Look at `kernel/kernel.c` to see how it works
3. **Modify something** - Try changing colors or adding a command
4. **Learn more** - Check out OSDev.org for advanced topics

---

## 📞 Need Help?

If you get stuck:

1. **Check the error message** - It usually tells you what's wrong
2. **Read the troubleshooting section** above
3. **Make sure you followed all steps** in order
4. **Take a screenshot** of any errors you see

---

## 🎉 Congratulations!

You've just:
- ✅ Installed VirtualBox
- ✅ Set up a Linux environment (WSL)
- ✅ Installed development tools
- ✅ Built an operating system from source code
- ✅ Run it in a virtual machine

This is a real achievement! You're now running a custom OS that you built yourself!

Enjoy exploring huggingOs! 🚀


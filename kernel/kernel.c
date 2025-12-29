#include "kernel.h"
#include "gdt.h"
#include "interrupts.h"
#include "memory/memory.h"
#include "drivers/drivers.h"
#include "terminal/terminal.h"
#include "lib/lib.h"
#include "syscalls/syscalls.h"
#include "sys/logging.h"

// Multiboot information structure
typedef struct {
    uint32_t flags;
    uint32_t mem_lower;
    uint32_t mem_upper;
    uint32_t boot_device;
    uint32_t cmdline;
    uint32_t mods_count;
    uint32_t mods_addr;
    uint32_t num;
    uint32_t size;
    uint32_t addr;
    uint32_t shndx;
    uint32_t mmap_length;
    uint32_t mmap_addr;
    uint32_t drives_length;
    uint32_t drives_addr;
    uint32_t config_table;
    uint32_t boot_loader_name;
    uint32_t apm_table;
    uint32_t vbe_control_info;
    uint32_t vbe_mode_info;
    uint32_t vbe_mode;
    uint32_t vbe_interface_seg;
    uint32_t vbe_interface_off;
    uint32_t vbe_interface_len;
} multiboot_info_t;

static multiboot_info_t* mb_info = 0;

void kernel_panic(const char* message)
{
    terminal_setcolor(VGA_COLOR_WHITE, VGA_COLOR_RED);
    terminal_writestring("KERNEL PANIC: ");
    terminal_writeln(message);
    terminal_writeln("System halted.");
    
    asm volatile("cli");
    asm volatile("hlt");
}

void kernel_main_multiboot(uint32_t magic, multiboot_info_t* mbi)
{
    if (magic != MULTIBOOT_BOOTLOADER_MAGIC) {
        kernel_panic("Invalid multiboot magic number!");
        return;
    }
    
    mb_info = mbi;
    
    // Initialize VGA first (this clears the screen)
    vga_init();
    
    // Initialize terminal (wrapper layer)
    terminal_initialize();
    
    // Print enhanced ASCII art banner with colors
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writestring("\n\n");
    terminal_writeln("  ╔═══════════════════════════════════════════════════════╗");
    terminal_writeln("  ║                                                       ║");
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writeln("  ║     _   _                   _         ___  ____       ║");
    terminal_writeln("  ║    | | | |_   _ _ __  _   _| |_ ___  / _ \\/ ___|      ║");
    terminal_writeln("  ║    | |_| | | | | '_ \\| | | | __/ _ \\| | | \\___ \\      ║");
    terminal_writeln("  ║    |  _  | |_| | | | | |_| | || (_) | |_| |___) |    ║");
    terminal_writeln("  ║    |_| |_|\\__, |_| |_|\\__,_|\\__\\___/ \\___/|____/     ║");
    terminal_writeln("  ║            |___/                                       ║");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("  ║                                                       ║");
    terminal_writeln("  ╚═══════════════════════════════════════════════════════╝");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("        ═══ Version 1.0.0 - Production Ready ═══");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writeln("     ✨ A Modern Minimal Custom Operating System ✨");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("════════════════════════════════════════════════════════════");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writeln("");
    terminal_writeln("  >> Initializing system components...");
    terminal_writeln("");
    
    // Initialize GDT
    terminal_writestring("  [");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writestring("1/9");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writestring("] Setting up GDT...              ");
    gdt_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize IDT and interrupts
    terminal_writestring("  [");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writestring("2/10");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writestring("] Setting up IDT...               ");
    idt_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize system calls
    terminal_writestring("  [");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writestring("3/11");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writestring("] Initializing system calls...    ");
    syscalls_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize logging
    terminal_writestring("  [");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writestring("4/11");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writestring("] Initializing logging...          ");
    log_init();
    log_info("kernel", "System logging initialized");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize memory management
    uint32_t mem_size = 64 * 1024 * 1024; // 64MB default
    if (mb_info && (mb_info->flags & 0x01)) {
        mem_size = (mb_info->mem_upper + 1024) * 1024; // Convert to bytes
    }
    terminal_writestring("  [");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writestring("5/11");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writestring("] Initializing memory...          ");
    memory_init(mem_size);
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize keyboard
    terminal_writestring("  [");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writestring("6/11");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writestring("] Initializing keyboard...        ");
    keyboard_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize PIT (timer)
    terminal_writestring("  [");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writestring("7/11");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writestring("] Initializing timer...         ");
    extern void pit_init(void);
    pit_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize RTC (clock)
    terminal_writestring("  [");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writestring("8/11");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writestring("] Initializing RTC...             ");
    rtc_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // VGA already initialized at boot, just confirm
    terminal_writestring("  [");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writestring("9/11");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writestring("] Graphics initialized...         ");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize file system
    terminal_writestring("  [");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writestring("10/11");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writestring("] Initializing file system...     ");
    extern void ramfs_init(void);
    ramfs_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize shell
    terminal_writestring("  [");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writestring("11/11");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writestring("] Starting shell...               ");
    log_info("kernel", "Shell initialized");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("════════════════════════════════════════════════════════════");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("  [OK] System ready! All components initialized successfully.");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writeln("  >> Tip: Type 'help' to see all available commands");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("════════════════════════════════════════════════════════════");
    terminal_writeln("");
    
    // Enable interrupts
    asm volatile("sti");
    
    // Print shell prompt
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    shell_print_prompt();
    
    // Main kernel loop
    while (1) {
        // Check for keyboard input
        char c = keyboard_get_char();
        if (c != 0) {
            shell_process_input(c);
            
            // Check if shell requested exit
            if (shell_should_exit()) {
                terminal_writeln("");
                terminal_writeln("Shell exited with code: ");
                char code_str[16];
                itoa(shell_get_exit_code(), code_str, 10);
                terminal_writeln(code_str);
                terminal_writeln("");
                terminal_writeln("Restarting shell...");
                terminal_writeln("");
                shell_init();
                shell_print_prompt();
            }
        }
        
        // Small delay to prevent CPU spinning
        asm volatile("pause");
    }
}


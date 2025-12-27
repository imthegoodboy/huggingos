#include "kernel.h"
#include "gdt.h"
#include "interrupts.h"
#include "memory/memory.h"
#include "drivers/drivers.h"
#include "terminal/terminal.h"
#include "lib/lib.h"

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
    
    // Continue with initialization
    // Initialize terminal with colorful theme
    terminal_initialize();
    terminal_setcolor(VGA_COLOR_CYAN, VGA_COLOR_BLACK);
    
    // Print welcome banner
    terminal_writestring("\n");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("========================================");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("     Welcome to huggingOs v1.0.0");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("========================================");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    terminal_writeln("");
    terminal_writeln("Initializing system...");
    
    // Initialize GDT
    terminal_writestring("  - Setting up GDT... ");
    gdt_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize IDT and interrupts
    terminal_writestring("  - Setting up IDT... ");
    idt_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize memory management
    uint32_t mem_size = 64 * 1024 * 1024; // 64MB default
    if (mb_info && (mb_info->flags & 0x01)) {
        mem_size = (mb_info->mem_upper + 1024) * 1024; // Convert to bytes
    }
    terminal_writestring("  - Initializing memory... ");
    memory_init(mem_size);
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize keyboard
    terminal_writestring("  - Initializing keyboard... ");
    keyboard_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize graphics (VGA for now)
    terminal_writestring("  - Initializing graphics... ");
    vga_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    // Initialize shell
    terminal_writestring("  - Starting shell... ");
    shell_init();
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OK]");
    terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("System ready!");
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
        }
        
        // Small delay to prevent CPU spinning
        asm volatile("pause");
    }
}


// This is called from boot.asm with multiboot info
void kernel_main_multiboot(uint32_t magic, multiboot_info_t* mbi)
{
    if (magic != MULTIBOOT_BOOTLOADER_MAGIC) {
        kernel_panic("Invalid multiboot magic number!");
        return;
    }
    
    mb_info = mbi;
    kernel_main();
}


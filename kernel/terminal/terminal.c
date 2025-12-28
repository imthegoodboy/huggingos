#include "terminal.h"
#include "../kernel.h"
#include "../drivers/drivers.h"
#include "../lib/lib.h"

// Forward declarations
extern void vga_putchar(char c);
extern void vga_clear(void);
extern void vga_setcolor(uint8_t color);

void terminal_initialize(void)
{
    // VGA should be initialized before terminal
    // This is just a wrapper, VGA handles all the state
}

void terminal_setcolor(uint8_t fg, uint8_t bg)
{
    // Combine foreground and background colors and sync with VGA
    uint8_t color = fg | (bg << 4);
    vga_setcolor(color);
}

void terminal_putchar(char c)
{
    // Just pass through to VGA - it handles all cursor positioning
    vga_putchar(c);
}

void terminal_write(const char* data, size_t size)
{
    for (size_t i = 0; i < size; i++)
        terminal_putchar(data[i]);
}

void terminal_writestring(const char* data)
{
    terminal_write(data, strlen(data));
}

void terminal_writechar(char c)
{
    terminal_putchar(c);
}

void terminal_writeln(const char* data)
{
    terminal_writestring(data);
    terminal_newline();
}

void terminal_clear(void)
{
    // VGA handles cursor reset in vga_clear()
    vga_clear();
}

void terminal_scroll(void)
{
    // VGA driver handles scrolling
}

void terminal_newline(void)
{
    terminal_putchar('\n');
}

void terminal_backspace(void)
{
    terminal_putchar('\b');
}

void terminal_handle_input(char c)
{
    // This will be called from keyboard interrupt
    terminal_putchar(c);
}

void terminal_update(void)
{
    // Called periodically to update display
}

uint8_t terminal_getcolor(void)
{
    // Return default color - VGA manages its own color state
    return VGA_COLOR_LIGHT_GREEN | (VGA_COLOR_BLACK << 4);
}

void terminal_set_cursor(size_t x, size_t y)
{
    // VGA manages cursor internally, this is a placeholder
    // In a full implementation, we'd need to expose VGA cursor functions
    (void)x;
    (void)y;
}

size_t terminal_get_row(void)
{
    // VGA manages row internally
    return 0;
}

size_t terminal_get_column(void)
{
    // VGA manages column internally
    return 0;
}



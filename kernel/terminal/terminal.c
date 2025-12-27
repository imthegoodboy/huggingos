#include "terminal.h"
#include "../kernel.h"
#include "../drivers/drivers.h"
#include "../lib/lib.h"

// Forward declarations
extern void vga_putchar(char c);
extern void vga_clear(void);

#define VGA_WIDTH 80
#define VGA_HEIGHT 25

static size_t terminal_row = 0;
static size_t terminal_column = 0;
static uint8_t terminal_color = 0;
static size_t terminal_buffer_size = 0;

void terminal_initialize(void)
{
    terminal_row = 0;
    terminal_column = 0;
    terminal_color = VGA_COLOR_LIGHT_GREEN | (VGA_COLOR_BLACK << 4);
    vga_init();
    terminal_buffer_size = 0;
}

void terminal_setcolor(uint8_t fg, uint8_t bg)
{
    // Combine foreground and background colors
    terminal_color = fg | (bg << 4);
}

void terminal_putchar(char c)
{
    vga_putchar(c);
    
    if (c == '\n') {
        terminal_row++;
        terminal_column = 0;
        if (terminal_row >= VGA_HEIGHT) {
            terminal_row = VGA_HEIGHT - 1;
            vga_clear();
        }
    } else if (c == '\b') {
        if (terminal_column > 0) {
            terminal_column--;
        }
    } else {
        terminal_column++;
        if (terminal_column >= VGA_WIDTH) {
            terminal_column = 0;
            terminal_row++;
            if (terminal_row >= VGA_HEIGHT) {
                terminal_row = VGA_HEIGHT - 1;
                vga_clear();
            }
        }
    }
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
    vga_clear();
    terminal_row = 0;
    terminal_column = 0;
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
    return terminal_color;
}

void terminal_set_cursor(size_t x, size_t y)
{
    terminal_column = x;
    terminal_row = y;
}

size_t terminal_get_row(void)
{
    return terminal_row;
}

size_t terminal_get_column(void)
{
    return terminal_column;
}



#include "drivers.h"
#include "../kernel.h"
#include "../lib/lib.h"

extern size_t strlen(const char* str);
extern void outb(unsigned short port, unsigned char data);
extern unsigned char inb(unsigned short port);

static uint16_t* const VGA_MEMORY = (uint16_t*) 0xB8000;

#define VGA_DISPLAY_OFFSET 2

static size_t terminal_row = 0;
static size_t terminal_column = 0;
static uint8_t terminal_color = 0;
static uint16_t* terminal_buffer = (uint16_t*) VGA_MEMORY + VGA_DISPLAY_OFFSET;

static void vga_reset_start_address(void)
{
    outb(0x3D4, 0x0C);
    outb(0x3D5, (uint8_t)((VGA_DISPLAY_OFFSET >> 8) & 0xFF));
    outb(0x3D4, 0x0D);
    outb(0x3D5, (uint8_t)(VGA_DISPLAY_OFFSET & 0xFF));
    outb(0x3B4, 0x0C);
    outb(0x3B5, (uint8_t)((VGA_DISPLAY_OFFSET >> 8) & 0xFF));
    outb(0x3B4, 0x0D);
    outb(0x3B5, (uint8_t)(VGA_DISPLAY_OFFSET & 0xFF));
    inb(0x3DA);
    outb(0x3C0, 0x20 | 0x13);
    outb(0x3C0, 0x00);
}

static void vga_update_cursor(void)
{
    uint16_t pos = (uint16_t)(terminal_row * VGA_WIDTH + terminal_column + VGA_DISPLAY_OFFSET);
    outb(0x3D4, 0x0F);
    outb(0x3D5, (uint8_t)(pos & 0xFF));
    outb(0x3D4, 0x0E);
    outb(0x3D5, (uint8_t)((pos >> 8) & 0xFF));
}

uint8_t vga_entry_color(uint8_t fg, uint8_t bg)
{
    return fg | bg << 4;
}

uint16_t vga_entry(unsigned char uc, uint8_t color)
{
    return (uint16_t) uc | (uint16_t) color << 8;
}

void vga_init(void)
{
    vga_reset_start_address();
    terminal_row = 0;
    terminal_column = 0;
    terminal_color = vga_entry_color(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    vga_clear();
}

void vga_setcolor(uint8_t color)
{
    terminal_color = color;
}

void vga_putentryat(char c, uint8_t color, size_t x, size_t y)
{
    const size_t index = y * VGA_WIDTH + x;
    terminal_buffer[index] = vga_entry(c, color);
}

static void vga_scroll(void)
{
    // Move all lines up by one
    for (size_t y = 0; y < VGA_HEIGHT - 1; y++) {
        for (size_t x = 0; x < VGA_WIDTH; x++) {
            const size_t index = (y + 1) * VGA_WIDTH + x;
            terminal_buffer[y * VGA_WIDTH + x] = terminal_buffer[index];
        }
    }
    
    // Clear the last line
    for (size_t x = 0; x < VGA_WIDTH; x++) {
        terminal_buffer[(VGA_HEIGHT - 1) * VGA_WIDTH + x] = vga_entry(' ', terminal_color);
    }
}

void vga_putchar(char c)
{
    unsigned char uc = c;
    
    if (uc == '\n') {
        terminal_column = 0;
        terminal_row++;
        if (terminal_row >= VGA_HEIGHT) {
            vga_scroll();
            terminal_row = VGA_HEIGHT - 1;
        }
        vga_update_cursor();
        return;
    }
    
    if (uc == '\b') {
        if (terminal_column > 0) {
            terminal_column--;
        } else if (terminal_row > 0) {
            terminal_row--;
            terminal_column = VGA_WIDTH - 1;
        }
        vga_putentryat(' ', terminal_color, terminal_column, terminal_row);
        vga_update_cursor();
        return;
    }
    
    if (uc == '\t') {
        terminal_column = (terminal_column + 4) & ~(4 - 1);
        if (terminal_column >= VGA_WIDTH) {
            terminal_column = 0;
            terminal_row++;
            if (terminal_row >= VGA_HEIGHT) {
                vga_scroll();
                terminal_row = VGA_HEIGHT - 1;
            }
        }
        vga_update_cursor();
        return;
    }

    if (uc < 32) {
        return;
    }

    if (uc >= 127) {
        uc = '?';
    }
    
    vga_putentryat(uc, terminal_color, terminal_column, terminal_row);
    
    terminal_column++;
    if (terminal_column >= VGA_WIDTH) {
        terminal_column = 0;
        terminal_row++;
        if (terminal_row >= VGA_HEIGHT) {
            vga_scroll();
            terminal_row = VGA_HEIGHT - 1;
        }
    }
    vga_update_cursor();
}

void vga_write(const char* data, size_t size)
{
    for (size_t i = 0; i < size; i++)
        vga_putchar(data[i]);
}

void vga_writestring(const char* data)
{
    vga_write(data, strlen(data));
}

void vga_clear(void)
{
    vga_reset_start_address();
    for (size_t y = 0; y < VGA_HEIGHT; y++) {
        for (size_t x = 0; x < VGA_WIDTH; x++) {
            const size_t index = y * VGA_WIDTH + x;
            terminal_buffer[index] = vga_entry(' ', terminal_color);
        }
    }
    terminal_row = 0;
    terminal_column = 0;
    vga_update_cursor();
}

void vga_set_cursor(size_t x, size_t y)
{
    if (x >= VGA_WIDTH) {
        x = VGA_WIDTH - 1;
    }
    if (y >= VGA_HEIGHT) {
        y = VGA_HEIGHT - 1;
    }

    terminal_column = x;
    terminal_row = y;
    vga_update_cursor();
}

size_t vga_get_row(void)
{
    return terminal_row;
}

size_t vga_get_column(void)
{
    return terminal_column;
}


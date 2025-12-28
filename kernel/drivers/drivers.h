#ifndef DRIVERS_H
#define DRIVERS_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// VESA Framebuffer
typedef struct {
    uint32_t width;
    uint32_t height;
    uint32_t pitch;
    uint32_t bpp;
    uint32_t* framebuffer;
    bool initialized;
} vesa_info_t;

bool vesa_init(void);
void vesa_set_pixel(uint32_t x, uint32_t y, uint32_t color);
void vesa_fill_rect(uint32_t x, uint32_t y, uint32_t width, uint32_t height, uint32_t color);
void vesa_draw_char(uint32_t x, uint32_t y, char c, uint32_t fg, uint32_t bg);
void vesa_draw_string(uint32_t x, uint32_t y, const char* str, uint32_t fg, uint32_t bg);
void vesa_clear(uint32_t color);
vesa_info_t* vesa_get_info(void);

// VGA Text Mode
void vga_init(void);
void vga_setcolor(uint8_t color);
void vga_putentryat(char c, uint8_t color, size_t x, size_t y);
void vga_putchar(char c);
void vga_write(const char* data, size_t size);
void vga_writestring(const char* data);
void vga_clear(void);

// Keyboard
typedef struct {
    bool shift;
    bool ctrl;
    bool alt;
    char last_char;
    bool key_pressed;
} keyboard_state_t;

void keyboard_init(void);
void keyboard_handler(uint8_t scancode);
char keyboard_get_char(void);
bool keyboard_is_key_pressed(void);
keyboard_state_t* keyboard_get_state(void);

#endif



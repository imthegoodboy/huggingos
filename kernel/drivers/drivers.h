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

// RTC (Real-Time Clock)
typedef struct {
    uint8_t second;
    uint8_t minute;
    uint8_t hour;
    uint8_t day;
    uint8_t month;
    uint8_t year;
    bool initialized;
} rtc_time_t;

void rtc_init(void);
void rtc_get_time(rtc_time_t* time);
uint16_t rtc_get_full_year(void);
uint8_t rtc_get_month(void);
uint8_t rtc_get_day(void);
uint8_t rtc_get_hour(void);
uint8_t rtc_get_minute(void);
uint8_t rtc_get_second(void);

// PIT (Programmable Interval Timer)
void pit_init(void);
void pit_handler(void);
uint32_t pit_get_ticks(void);
uint32_t pit_get_milliseconds(void);
uint32_t pit_get_seconds(void);
void pit_delay_ms(uint32_t milliseconds);
bool pit_is_initialized(void);

#endif



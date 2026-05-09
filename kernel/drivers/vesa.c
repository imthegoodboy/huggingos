#include "drivers.h"
#include "../kernel.h"
#include "../lib/lib.h"

vesa_info_t vesa_info = {0};

bool vesa_init(void)
{
    vesa_info.width = 0;
    vesa_info.height = 0;
    vesa_info.pitch = 0;
    vesa_info.bpp = 0;
    vesa_info.framebuffer = NULL;
    vesa_info.initialized = false;
    return false;
}

vesa_info_t* vesa_get_info(void)
{
    return &vesa_info;
}

void vesa_set_pixel(uint32_t x, uint32_t y, uint32_t color)
{
    if (!vesa_info.initialized || !vesa_info.framebuffer)
        return;
    
    if (x >= vesa_info.width || y >= vesa_info.height)
        return;
    
    uint32_t offset = y * (vesa_info.pitch / 4) + x;
    vesa_info.framebuffer[offset] = color;
}

void vesa_fill_rect(uint32_t x, uint32_t y, uint32_t width, uint32_t height, uint32_t color)
{
    for (uint32_t py = y; py < y + height && py < vesa_info.height; py++) {
        for (uint32_t px = x; px < x + width && px < vesa_info.width; px++) {
            vesa_set_pixel(px, py, color);
        }
    }
}

void vesa_draw_char(uint32_t x, uint32_t y, char c, uint32_t fg, uint32_t bg)
{
    uint8_t char_val = (uint8_t)c;
    
    // Draw background
    vesa_fill_rect(x, y, 8, 8, bg);
    
    // Draw character outline (simplified)
    if (char_val >= 32 && char_val < 127) {
        for (int i = 0; i < 8 && i + y < vesa_info.height; i++) {
            for (int j = 0; j < 8 && j + x < vesa_info.width; j++) {
                // Simplified: draw a simple pattern
                if ((i == 0 || i == 7) || (j == 0 || j == 7)) {
                    vesa_set_pixel(x + j, y + i, fg);
                }
            }
        }
    }
}

void vesa_draw_string(uint32_t x, uint32_t y, const char* str, uint32_t fg, uint32_t bg)
{
    uint32_t cx = x;
    while (*str && cx + 8 < vesa_info.width) {
        vesa_draw_char(cx, y, *str, fg, bg);
        cx += 9; // Character width + spacing
        str++;
    }
}

void vesa_clear(uint32_t color)
{
    if (!vesa_info.initialized || !vesa_info.framebuffer)
        return;
    
    for (uint32_t y = 0; y < vesa_info.height; y++) {
        for (uint32_t x = 0; x < vesa_info.width; x++) {
            vesa_set_pixel(x, y, color);
        }
    }
}







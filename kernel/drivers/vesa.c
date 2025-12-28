#include "drivers.h"
#include "../kernel.h"
#include "../lib/lib.h"

vesa_info_t vesa_info = {0};

// Simple 8x8 font bitmap (simplified)
static const unsigned char font_8x8[128][8] = {
    // Basic ASCII characters
    [32] = {0,0,0,0,0,0,0,0}, // space
    [33] = {0x18,0x3C,0x3C,0x18,0x18,0,0x18,0}, // !
    // ... simplified font (would include full ASCII in real implementation)
};

static uint32_t rgb(uint8_t r, uint8_t g, uint8_t b)
{
    return (r << 16) | (g << 8) | b;
}

bool vesa_init(void)
{
    // In a real implementation, we'd use VESA BIOS Extensions
    // For now, use a simulated framebuffer at a fixed address
    // In VirtualBox/QEMU, we can use the multiboot framebuffer info
    
    // Default resolution for compatibility
    vesa_info.width = 1024;
    vesa_info.height = 768;
    vesa_info.pitch = 1024 * 4; // 32-bit color
    vesa_info.bpp = 32;
    
    // Try to use multiboot framebuffer if available
    // For now, use a simulated framebuffer
    vesa_info.framebuffer = (uint32_t*)0xFD000000; // Simulated address
    
    // Initialize with background color
    vesa_clear(COLOR_BG_DARK);
    
    vesa_info.initialized = true;
    return true;
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
    // Simplified character drawing
    // In a full implementation, we'd use a proper font bitmap
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






#ifndef KERNEL_H
#define KERNEL_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// Color definitions for huggingOs theme
#define COLOR_BG_DARK      0x1a1a2e
#define COLOR_BG_LIGHT     0x16213e
#define COLOR_PRIMARY      0x0f3460
#define COLOR_ACCENT       0xe94560
#define COLOR_TEXT         0xffffff
#define COLOR_SUCCESS      0x00ff00
#define COLOR_WARNING      0xffff00
#define COLOR_ERROR        0xff0000
#define COLOR_INFO         0x00bfff
#define COLOR_CYAN         0x00ffff
#define COLOR_MAGENTA      0xff00ff
#define COLOR_GREEN        0x00ff88
#define COLOR_BLUE         0x0088ff
#define COLOR_PURPLE       0x8844ff

// VGA Colors (for text mode)
#define VGA_COLOR_BLACK         0
#define VGA_COLOR_BLUE          1
#define VGA_COLOR_GREEN         2
#define VGA_COLOR_CYAN          3
#define VGA_COLOR_RED           4
#define VGA_COLOR_MAGENTA       5
#define VGA_COLOR_BROWN         6
#define VGA_COLOR_LIGHT_GREY    7
#define VGA_COLOR_DARK_GREY     8
#define VGA_COLOR_LIGHT_BLUE    9
#define VGA_COLOR_LIGHT_GREEN   10
#define VGA_COLOR_LIGHT_CYAN    11
#define VGA_COLOR_LIGHT_RED     12
#define VGA_COLOR_LIGHT_MAGENTA 13
#define VGA_COLOR_YELLOW        14
#define VGA_COLOR_WHITE         15

// VGA text mode dimensions
#define VGA_WIDTH  80
#define VGA_HEIGHT 25

// Multiboot header
#define MULTIBOOT_HEADER_MAGIC      0x1BADB002
#define MULTIBOOT_BOOTLOADER_MAGIC  0x2BADB002

// Function declarations
void kernel_main(void);
void kernel_panic(const char* message);

// Utility macros
#define UNUSED(x) (void)(x)

#endif






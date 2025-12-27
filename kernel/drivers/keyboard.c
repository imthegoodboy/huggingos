#include "drivers.h"
#include "../kernel.h"
#include "../interrupts.h"
#include "../lib/lib.h"

extern unsigned char inb(unsigned short port);
extern void outb(unsigned short port, unsigned char data);

keyboard_state_t keyboard_state = {0};

// US QWERTY keyboard scancode to ASCII mapping (set 1)
static const char scancode_to_ascii[128] = {
    0, 27, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\b',
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
    0, 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`', 0, '\\',
    'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', 0, '*', 0, ' ', 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '-', 0, 0, 0, '+', 0,
    0, 0, 0, 0, 0, 0, '<', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

static const char scancode_to_ascii_shift[128] = {
    0, 27, '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\b',
    '\t', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n',
    0, 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':', '"', '~', 0, '|',
    'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?', 0, '*', 0, ' ', 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '-', 0, 0, 0, '+', 0,
    0, 0, 0, 0, 0, 0, '<', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

void keyboard_init(void)
{
    keyboard_state.shift = false;
    keyboard_state.ctrl = false;
    keyboard_state.alt = false;
    keyboard_state.last_char = 0;
    keyboard_state.key_pressed = false;
    
    // Enable keyboard interrupt (IRQ1)
    outb(0x21, inb(0x21) & 0xFD);
}

void keyboard_handler(uint8_t scancode)
{
    if (scancode & 0x80) {
        // Key release
        uint8_t key = scancode & 0x7F;
        
        if (key == 0x2A || key == 0x36) {
            keyboard_state.shift = false;
        } else if (key == 0x1D) {
            keyboard_state.ctrl = false;
        } else if (key == 0x38) {
            keyboard_state.alt = false;
        }
    } else {
        // Key press
        if (scancode == 0x2A || scancode == 0x36) {
            keyboard_state.shift = true;
        } else if (scancode == 0x1D) {
            keyboard_state.ctrl = true;
        } else if (scancode == 0x38) {
            keyboard_state.alt = true;
        } else {
            char c = keyboard_state.shift ? 
                     scancode_to_ascii_shift[scancode] : 
                     scancode_to_ascii[scancode];
            
            if (c != 0) {
                keyboard_state.last_char = c;
                keyboard_state.key_pressed = true;
                
                // Send to terminal
                extern void terminal_handle_input(char);
                terminal_handle_input(c);
            }
        }
    }
}

char keyboard_get_char(void)
{
    if (keyboard_state.key_pressed) {
        keyboard_state.key_pressed = false;
        return keyboard_state.last_char;
    }
    return 0;
}

bool keyboard_is_key_pressed(void)
{
    return keyboard_state.key_pressed;
}

keyboard_state_t* keyboard_get_state(void)
{
    return &keyboard_state;
}


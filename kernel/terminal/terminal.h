#ifndef TERMINAL_H
#define TERMINAL_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define TERMINAL_MAX_LINE_LENGTH 256
#define TERMINAL_HISTORY_SIZE 32

typedef struct {
    char buffer[TERMINAL_MAX_LINE_LENGTH];
    size_t length;
    size_t cursor_pos;
} terminal_line_t;

void terminal_initialize(void);
void terminal_clear(void);
void terminal_setcolor(uint8_t fg, uint8_t bg);
void terminal_putchar(char c);
void terminal_write(const char* data, size_t size);
void terminal_writestring(const char* data);
void terminal_writechar(char c);
void terminal_writeln(const char* data);
void terminal_scroll(void);
void terminal_newline(void);
void terminal_backspace(void);
void terminal_handle_input(char c);
void terminal_update(void);
uint8_t terminal_getcolor(void);
void terminal_set_cursor(size_t x, size_t y);
size_t terminal_get_row(void);
size_t terminal_get_column(void);

// Shell functions
void shell_init(void);
void shell_process_input(char c);
void shell_print_prompt(void);
bool shell_should_exit(void);
int shell_get_exit_code(void);

#endif


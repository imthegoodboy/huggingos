#include "lib.h"
#include "../kernel.h"
#include "../terminal/terminal.h"

typedef __builtin_va_list va_list;
#define va_start(ap, last) __builtin_va_start(ap, last)
#define va_end(ap) __builtin_va_end(ap)
#define va_arg(ap, type) __builtin_va_arg(ap, type)

void putchar(char c)
{
    terminal_putchar(c);
}

void puts(const char* str)
{
    terminal_writestring(str);
    terminal_newline();
}

static void print_num(unsigned int num, int base)
{
    char digits[] = "0123456789ABCDEF";
    char buffer[32];
    int i = 0;
    
    if (num == 0) {
        putchar('0');
        return;
    }
    
    while (num > 0) {
        buffer[i++] = digits[num % base];
        num /= base;
    }
    
    while (--i >= 0) {
        putchar(buffer[i]);
    }
}

void printf(const char* format, ...)
{
    va_list args;
    va_start(args, format);
    
    const char* p = format;
    while (*p) {
        if (*p == '%' && *(p + 1)) {
            p++;
            switch (*p) {
                case 'd':
                case 'i': {
                    int val = va_arg(args, int);
                    if (val < 0) {
                        putchar('-');
                        val = -val;
                    }
                    print_num((unsigned int)val, 10);
                    break;
                }
                case 'u':
                    print_num(va_arg(args, unsigned int), 10);
                    break;
                case 'x':
                case 'X':
                    putchar('0');
                    putchar('x');
                    print_num(va_arg(args, unsigned int), 16);
                    break;
                case 's': {
                    const char* str = va_arg(args, const char*);
                    terminal_writestring(str);
                    break;
                }
                case 'c':
                    putchar((char)va_arg(args, int));
                    break;
                case '%':
                    putchar('%');
                    break;
                default:
                    putchar('%');
                    putchar(*p);
                    break;
            }
        } else {
            putchar(*p);
        }
        p++;
    }
    
    va_end(args);
}

int atoi(const char* str)
{
    int result = 0;
    int sign = 1;
    
    if (*str == '-') {
        sign = -1;
        str++;
    }
    
    while (*str >= '0' && *str <= '9') {
        result = result * 10 + (*str - '0');
        str++;
    }
    
    return result * sign;
}

char* itoa(int value, char* str, int base)
{
    char* ptr = str;
    char* ptr1 = str;
    char tmp_char;
    int tmp_value;
    
    if (base < 2 || base > 36) {
        *str = '\0';
        return str;
    }
    
    do {
        tmp_value = value;
        value /= base;
        *ptr++ = "zyxwvutsrqponmlkjihgfedcba9876543210123456789abcdefghijklmnopqrstuvwxyz" [35 + (tmp_value - value * base)];
    } while (value);
    
    if (tmp_value < 0)
        *ptr++ = '-';
    
    *ptr-- = '\0';
    while (ptr1 < ptr) {
        tmp_char = *ptr;
        *ptr-- = *ptr1;
        *ptr1++ = tmp_char;
    }
    
    return str;
}


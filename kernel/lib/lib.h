#ifndef LIB_H
#define LIB_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// String functions
size_t strlen(const char* str);
int strcmp(const char* s1, const char* s2);
int strncmp(const char* s1, const char* s2, size_t n);
char* strcpy(char* dest, const char* src);
char* strncpy(char* dest, const char* src, size_t n);
char* strcat(char* dest, const char* src);
char* strchr(const char* str, int character);
char* strrchr(const char* str, int character);
char* strstr(const char* haystack, const char* needle);
char* strtok(char* str, const char* delimiters);
int strcasecmp(const char* s1, const char* s2);
void* memset(void* ptr, int value, size_t num);
void* memcpy(void* dest, const void* src, size_t num);
void* memmove(void* dest, const void* src, size_t num);

// I/O functions
void putchar(char c);
void puts(const char* str);
void printf(const char* format, ...);
int atoi(const char* str);
char* itoa(int value, char* str, int base);

// Memory functions
void* kmalloc(size_t size);
void kfree(void* ptr);

#endif



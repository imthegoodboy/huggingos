#include "lib.h"

size_t strlen(const char* str)
{
    size_t len = 0;
    while (str[len])
        len++;
    return len;
}

int strcmp(const char* s1, const char* s2)
{
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(const unsigned char*)s1 - *(const unsigned char*)s2;
}

int strncmp(const char* s1, const char* s2, size_t n)
{
    while (n && *s1 && (*s1 == *s2)) {
        s1++;
        s2++;
        n--;
    }
    if (n == 0)
        return 0;
    return *(const unsigned char*)s1 - *(const unsigned char*)s2;
}

char* strcpy(char* dest, const char* src)
{
    char* ret = dest;
    while ((*dest++ = *src++))
        ;
    return ret;
}

char* strncpy(char* dest, const char* src, size_t n)
{
    char* ret = dest;
    size_t i;
    for (i = 0; i < n && src[i] != '\0'; i++)
        dest[i] = src[i];
    for (; i < n; i++)
        dest[i] = '\0';
    return ret;
}

char* strcat(char* dest, const char* src)
{
    char* ret = dest;
    while (*dest)
        dest++;
    while ((*dest++ = *src++))
        ;
    return ret;
}

char* strchr(const char* str, int character)
{
    while (*str) {
        if (*str == (char)character)
            return (char*)str;
        str++;
    }
    if (character == '\0')
        return (char*)str;
    return NULL;
}

char* strrchr(const char* str, int character)
{
    const char* last = NULL;
    while (*str) {
        if (*str == (char)character)
            last = str;
        str++;
    }
    if (character == '\0')
        return (char*)str;
    return (char*)last;
}

static char* strtok_save = NULL;

char* strtok(char* str, const char* delimiters)
{
    char* token_start;
    
    if (str != NULL) {
        strtok_save = str;
    } else if (strtok_save == NULL) {
        return NULL;
    }
    
    // Skip leading delimiters
    while (*strtok_save != '\0') {
        int is_delim = 0;
        for (const char* d = delimiters; *d != '\0'; d++) {
            if (*strtok_save == *d) {
                is_delim = 1;
                break;
            }
        }
        if (!is_delim) break;
        strtok_save++;
    }
    
    if (*strtok_save == '\0') {
        strtok_save = NULL;
        return NULL;
    }
    
    token_start = strtok_save;
    
    // Find end of token
    while (*strtok_save != '\0') {
        int is_delim = 0;
        for (const char* d = delimiters; *d != '\0'; d++) {
            if (*strtok_save == *d) {
                is_delim = 1;
                break;
            }
        }
        if (is_delim) {
            *strtok_save = '\0';
            strtok_save++;
            return token_start;
        }
        strtok_save++;
    }
    
    strtok_save = NULL;
    return token_start;
}

void* memset(void* ptr, int value, size_t num)
{
    unsigned char* p = (unsigned char*)ptr;
    while (num--)
        *p++ = (unsigned char)value;
    return ptr;
}

void* memcpy(void* dest, const void* src, size_t num)
{
    unsigned char* d = (unsigned char*)dest;
    const unsigned char* s = (const unsigned char*)src;
    while (num--)
        *d++ = *s++;
    return dest;
}

void* memmove(void* dest, const void* src, size_t num)
{
    unsigned char* d = (unsigned char*)dest;
    const unsigned char* s = (const unsigned char*)src;
    
    if (d < s) {
        while (num--)
            *d++ = *s++;
    } else {
        d += num;
        s += num;
        while (num--)
            *--d = *--s;
    }
    return dest;
}

char* strstr(const char* haystack, const char* needle)
{
    if (!needle || !*needle) return (char*)haystack;
    
    const char* h = haystack;
    while (*h) {
        const char* n = needle;
        const char* h2 = h;
        
        while (*h2 && *n && *h2 == *n) {
            h2++;
            n++;
        }
        
        if (!*n) return (char*)h;
        h++;
    }
    
    return NULL;
}

int strcasecmp(const char* s1, const char* s2)
{
    while (*s1 && *s2) {
        char c1 = *s1;
        char c2 = *s2;
        
        // Convert to lowercase
        if (c1 >= 'A' && c1 <= 'Z') c1 += 32;
        if (c2 >= 'A' && c2 <= 'Z') c2 += 32;
        
        if (c1 != c2) return c1 - c2;
        s1++;
        s2++;
    }
    
    char c1 = *s1;
    char c2 = *s2;
    if (c1 >= 'A' && c1 <= 'Z') c1 += 32;
    if (c2 >= 'A' && c2 <= 'Z') c2 += 32;
    
    return c1 - c2;
}



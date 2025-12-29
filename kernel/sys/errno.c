#include "errno.h"
#include "../lib/lib.h"

int errno = ENOERROR;

static const char* error_messages[] = {
    "Success",
    "Operation not permitted",
    "No such file or directory",
    "No such process",
    "Interrupted system call",
    "I/O error",
    "No such device or address",
    "Argument list too long",
    "Exec format error",
    "Bad file descriptor",
    "No child processes",
    "Try again",
    "Out of memory",
    "Permission denied",
    "Bad address",
    "Block device required",
    "Device or resource busy",
    "File exists",
    "Cross-device link",
    "No such device",
    "Not a directory",
    "Is a directory",
    "Invalid argument",
    "File table overflow",
    "Too many open files",
    "Not a typewriter",
    "Text file busy",
    "File too large",
    "No space left on device",
    "Illegal seek",
    "Read-only file system",
    "Too many links",
    "Broken pipe",
    "Math argument out of domain",
    "Math result not representable"
};

const char* strerror(int errnum)
{
    if (errnum >= 0 && errnum < 35) {
        return error_messages[errnum];
    }
    return "Unknown error";
}

void perror(const char* s)
{
    extern void terminal_writestring(const char*);
    extern void terminal_writeln(const char*);
    
    if (s && s[0] != '\0') {
        terminal_writestring(s);
        terminal_writestring(": ");
    }
    terminal_writeln(strerror(errno));
}



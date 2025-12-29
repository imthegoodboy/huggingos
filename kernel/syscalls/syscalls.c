#include "syscalls.h"
#include "../kernel.h"
#include "../lib/lib.h"
#include "../terminal/terminal.h"
#include "../fs/fs.h"
#include "../drivers/drivers.h"

static bool syscalls_initialized = false;
static int syscall_return_value = 0;

// Basic process ID tracking (simplified)
static uint32_t current_pid = 1;
static uint32_t next_pid = 2;

void syscalls_init(void)
{
    syscalls_initialized = true;
    current_pid = 1;
}

int sys_exit(int status)
{
    // In a full OS, this would terminate the process
    // For now, just return status
    return status;
}

int sys_write(int fd, const char* buf, size_t count)
{
    if (!buf || count == 0) return -1;
    
    if (fd == 1 || fd == 2) { // stdout/stderr
        for (size_t i = 0; i < count && buf[i] != '\0'; i++) {
            terminal_writechar(buf[i]);
        }
        return count;
    }
    
    return -1; // Unsupported file descriptor
}

int sys_read(int fd, char* buf, size_t count)
{
    // Simplified read - for keyboard input
    if (fd == 0 && buf && count > 0) { // stdin
        extern char keyboard_get_char(void);
        char c = keyboard_get_char();
        if (c != 0) {
            buf[0] = c;
            return 1;
        }
    }
    return 0;
}

int sys_getpid(void)
{
    return current_pid;
}

int sys_sleep(uint32_t seconds)
{
    extern uint32_t pit_get_milliseconds(void);
    uint32_t start = pit_get_milliseconds();
    uint32_t target = start + (seconds * 1000);
    
    while (pit_get_milliseconds() < target) {
        asm volatile("pause");
    }
    return 0;
}

int sys_getenv(const char* name, char* value, size_t max_len)
{
    if (!name || !value || max_len == 0) return -1;
    
    // Get from shell environment
    extern const char* shell_getenv(const char* name);
    const char* env_value = shell_getenv(name);
    if (env_value) {
        strncpy(value, env_value, max_len - 1);
        value[max_len - 1] = '\0';
        return 0;
    }
    return -1;
}

int sys_setenv(const char* name, const char* value)
{
    if (!name) return -1;
    
    // Set in shell environment  
    extern void shell_setenv(const char* name, const char* value);
    shell_setenv(name, value ? value : "");
    return 0;
}

void syscall_handler(uint32_t syscall_num, uint32_t arg1, uint32_t arg2, uint32_t arg3, uint32_t arg4)
{
    int result = -1;
    
    switch (syscall_num) {
        case SYS_EXIT:
            result = sys_exit((int)arg1);
            break;
        case SYS_WRITE:
            result = sys_write((int)arg1, (const char*)arg2, (size_t)arg3);
            break;
        case SYS_READ:
            result = sys_read((int)arg1, (char*)arg2, (size_t)arg3);
            break;
        case SYS_GETPID:
            result = sys_getpid();
            break;
        case SYS_SLEEP:
            result = sys_sleep(arg1);
            break;
        case SYS_GETENV:
            result = sys_getenv((const char*)arg1, (char*)arg2, (size_t)arg3);
            break;
        case SYS_SETENV:
            result = sys_setenv((const char*)arg1, (const char*)arg2);
            break;
        default:
            result = -1; // Unsupported syscall
            break;
    }
    
    syscall_return_value = result;
}

// Get the last syscall return value (for asm stub)
int syscall_get_return(void)
{
    return syscall_return_value;
}


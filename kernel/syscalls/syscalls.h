#ifndef SYSCALLS_H
#define SYSCALLS_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// System call numbers
#define SYS_EXIT        1
#define SYS_WRITE       2
#define SYS_READ        3
#define SYS_OPEN        4
#define SYS_CLOSE       5
#define SYS_FORK        6
#define SYS_EXEC        7
#define SYS_WAIT        8
#define SYS_GETPID      9
#define SYS_GETTIME     10
#define SYS_SLEEP       11
#define SYS_KILL        12
#define SYS_MALLOC      13
#define SYS_FREE        14
#define SYS_GETENV      15
#define SYS_SETENV      16

// System call handler
void syscall_handler(uint32_t syscall_num, uint32_t arg1, uint32_t arg2, uint32_t arg3, uint32_t arg4);
int syscall_get_return(void);
void syscalls_init(void);

// System call wrapper functions
int sys_exit(int status);
int sys_write(int fd, const char* buf, size_t count);
int sys_read(int fd, char* buf, size_t count);
int sys_getpid(void);
int sys_sleep(uint32_t seconds);
int sys_getenv(const char* name, char* value, size_t max_len);
int sys_setenv(const char* name, const char* value);

#endif


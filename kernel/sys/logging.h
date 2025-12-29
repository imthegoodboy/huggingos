#ifndef LOGGING_H
#define LOGGING_H

#include <stdint.h>
#include <stdbool.h>

// Log levels
#define LOG_DEBUG   0
#define LOG_INFO    1
#define LOG_WARNING 2
#define LOG_ERROR   3
#define LOG_CRITICAL 4

// Log functions
void log_init(void);
void log_message(int level, const char* component, const char* message);
void log_debug(const char* component, const char* message);
void log_info(const char* component, const char* message);
void log_warning(const char* component, const char* message);
void log_error(const char* component, const char* message);
void log_critical(const char* component, const char* message);

// Log buffer management
void log_print_all(void);
void log_clear(void);
uint32_t log_count(void);

#endif



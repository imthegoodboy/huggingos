#include "logging.h"
#include "../kernel.h"
#include "../lib/lib.h"
#include "../terminal/terminal.h"

#define MAX_LOG_ENTRIES 256
#define MAX_LOG_MESSAGE 128
#define MAX_LOG_COMPONENT 32

typedef struct {
    int level;
    char component[MAX_LOG_COMPONENT];
    char message[MAX_LOG_MESSAGE];
    uint32_t timestamp;
} log_entry_t;

static log_entry_t log_buffer[MAX_LOG_ENTRIES];
static uint32_t log_count_val = 0;
static uint32_t log_index = 0;
static bool logging_enabled = true;

static const char* level_names[] = {
    "DEBUG", "INFO", "WARN", "ERROR", "CRIT"
};

void log_init(void)
{
    memset(log_buffer, 0, sizeof(log_buffer));
    log_count_val = 0;
    log_index = 0;
    logging_enabled = true;
}

void log_message(int level, const char* component, const char* message)
{
    if (!logging_enabled || !message) return;
    
    log_entry_t* entry = &log_buffer[log_index];
    entry->level = level;
    
    if (component) {
        strncpy(entry->component, component, MAX_LOG_COMPONENT - 1);
        entry->component[MAX_LOG_COMPONENT - 1] = '\0';
    } else {
        entry->component[0] = '\0';
    }
    
    strncpy(entry->message, message, MAX_LOG_MESSAGE - 1);
    entry->message[MAX_LOG_MESSAGE - 1] = '\0';
    
    extern uint32_t pit_get_milliseconds(void);
    entry->timestamp = pit_get_milliseconds();
    
    log_index = (log_index + 1) % MAX_LOG_ENTRIES;
    if (log_count_val < MAX_LOG_ENTRIES) {
        log_count_val++;
    }
}

void log_debug(const char* component, const char* message)
{
    log_message(LOG_DEBUG, component, message);
}

void log_info(const char* component, const char* message)
{
    log_message(LOG_INFO, component, message);
}

void log_warning(const char* component, const char* message)
{
    log_message(LOG_WARNING, component, message);
}

void log_error(const char* component, const char* message)
{
    log_message(LOG_ERROR, component, message);
}

void log_critical(const char* component, const char* message)
{
    log_message(LOG_CRITICAL, component, message);
}

void log_print_all(void)
{
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("=== System Log ===");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    
    uint32_t start = (log_count_val >= MAX_LOG_ENTRIES) ? log_index : 0;
    uint32_t count = (log_count_val < MAX_LOG_ENTRIES) ? log_count_val : MAX_LOG_ENTRIES;
    
    for (uint32_t i = 0; i < count; i++) {
        uint32_t idx = (start + i) % MAX_LOG_ENTRIES;
        log_entry_t* entry = &log_buffer[idx];
        
        if (entry->message[0] != '\0') {
            // Set color based on level
            switch (entry->level) {
                case LOG_DEBUG:
                    terminal_setcolor(VGA_COLOR_DARK_GREY, VGA_COLOR_BLACK);
                    break;
                case LOG_INFO:
                    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
                    break;
                case LOG_WARNING:
                    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
                    break;
                case LOG_ERROR:
                case LOG_CRITICAL:
                    terminal_setcolor(VGA_COLOR_LIGHT_RED, VGA_COLOR_BLACK);
                    break;
            }
            
            terminal_writestring("[");
            terminal_writestring(level_names[entry->level]);
            terminal_writestring("] ");
            
            if (entry->component[0] != '\0') {
                terminal_writestring(entry->component);
                terminal_writestring(": ");
            }
            
            terminal_writeln(entry->message);
        }
    }
    
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
}

void log_clear(void)
{
    memset(log_buffer, 0, sizeof(log_buffer));
    log_count_val = 0;
    log_index = 0;
}

uint32_t log_count(void)
{
    return log_count_val;
}



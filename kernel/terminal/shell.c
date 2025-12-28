#include "terminal.h"
#include "../kernel.h"
#include "../lib/lib.h"
#include "../drivers/drivers.h"

#define SHELL_MAX_INPUT 256
#define SHELL_MAX_ARGS 16
#define SHELL_HISTORY_SIZE 20

static char shell_input[SHELL_MAX_INPUT];
static size_t shell_input_pos = 0;
static char shell_prompt[] = "huggingOS> ";
static char shell_history[SHELL_HISTORY_SIZE][SHELL_MAX_INPUT];
static int shell_history_count = 0;

static void shell_execute_command(const char* command);

void shell_init(void)
{
    shell_input_pos = 0;
    shell_history_count = 0;
    memset(shell_input, 0, SHELL_MAX_INPUT);
    memset(shell_history, 0, sizeof(shell_history));
}

void shell_process_input(char c)
{
    if (c == '\n' || c == '\r') {
        shell_input[shell_input_pos] = '\0';
        if (shell_input_pos > 0) {
            shell_execute_command(shell_input);
            shell_input_pos = 0;
            memset(shell_input, 0, SHELL_MAX_INPUT);
        }
        terminal_writestring(shell_prompt);
    } else if (c == '\b' || c == 127) {
        if (shell_input_pos > 0) {
            shell_input_pos--;
            shell_input[shell_input_pos] = '\0';
            terminal_backspace();
        }
    } else if (shell_input_pos < SHELL_MAX_INPUT - 1 && c >= 32 && c < 127) {
        shell_input[shell_input_pos++] = c;
        terminal_writechar(c);
    }
}

static void cmd_help(void)
{
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("huggingOS - Available Commands:");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("");
    terminal_writeln("  System Commands:");
    terminal_writeln("    help      - Show this help message");
    terminal_writeln("    clear     - Clear the screen");
    terminal_writeln("    info      - Show system information");
    terminal_writeln("    version   - Show OS version");
    terminal_writeln("    reboot    - Reboot the system");
    terminal_writeln("    shutdown  - Shutdown the system");
    terminal_writeln("    whoami    - Show current user");
    terminal_writeln("    pwd       - Print working directory");
    terminal_writeln("");
    terminal_writeln("  Date & Time:");
    terminal_writeln("    date      - Show date and time");
    terminal_writeln("    clock     - Show current time");
    terminal_writeln("    calendar  - Show calendar");
    terminal_writeln("    timer     - Start a timer (seconds)");
    terminal_writeln("    uptime    - Show system uptime");
    terminal_writeln("");
    terminal_writeln("  Utilities:");
    terminal_writeln("    echo      - Echo text back");
    terminal_writeln("    calc      - Simple calculator");
    terminal_writeln("    color     - Change terminal color");
    terminal_writeln("    banner    - Show ASCII art banner");
    terminal_writeln("    about     - About huggingOS");
    terminal_writeln("    history   - Show command history");
    terminal_writeln("    mem       - Show memory information");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
}

static void cmd_clear(void)
{
    terminal_clear();
    terminal_writestring(shell_prompt);
}

static void cmd_info(void)
{
    // Get uptime
    uint32_t uptime_seconds = pit_get_seconds();
    uint32_t uptime_hours = uptime_seconds / 3600;
    uint32_t uptime_minutes = (uptime_seconds % 3600) / 60;
    
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("=== huggingOs System Information ===");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("OS Name: huggingOs");
    terminal_writeln("Version: 1.0.0");
    terminal_writeln("Architecture: x86 (32-bit)");
    terminal_writeln("Kernel: Monolithic");
    terminal_writeln("Graphics: VGA Text Mode");
    terminal_writeln("Features: Terminal, Shell, Keyboard, RTC, PIT Timer");
    
    terminal_writestring("Uptime: ");
    char hour_str[16], minute_str[16];
    itoa(uptime_hours, hour_str, 10);
    itoa(uptime_minutes, minute_str, 10);
    terminal_writestring(hour_str);
    terminal_writestring("h ");
    terminal_writestring(minute_str);
    terminal_writeln("m");
    
    terminal_writeln("Status: Operational");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
}

static void cmd_version(void)
{
    terminal_writeln("huggingOs v1.0.0");
    terminal_writeln("Built with love and dedication");
}

static void cmd_echo(const char* args)
{
    if (strlen(args) > 0) {
        terminal_writeln(args);
    } else {
        terminal_writeln("");
    }
}

static void cmd_reboot(void)
{
    terminal_writeln("Rebooting...");
    // In a real implementation, we'd trigger a reboot
    // For now, just halt
    asm volatile("cli");
    asm volatile("hlt");
}

static void cmd_color(const char* args)
{
    if (strlen(args) == 0) {
        terminal_writeln("Usage: color <color_name>");
        terminal_writeln("Available colors: green, cyan, yellow, red, blue, magenta, white");
        return;
    }
    
    if (strcmp(args, "green") == 0) {
        terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
        terminal_writeln("Color changed to green");
    } else if (strcmp(args, "cyan") == 0) {
        terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
        terminal_writeln("Color changed to cyan");
    } else if (strcmp(args, "yellow") == 0) {
        terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
        terminal_writeln("Color changed to yellow");
    } else if (strcmp(args, "red") == 0) {
        terminal_setcolor(VGA_COLOR_LIGHT_RED, VGA_COLOR_BLACK);
        terminal_writeln("Color changed to red");
    } else if (strcmp(args, "blue") == 0) {
        terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
        terminal_writeln("Color changed to blue");
    } else if (strcmp(args, "magenta") == 0) {
        terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
        terminal_writeln("Color changed to magenta");
    } else if (strcmp(args, "white") == 0) {
        terminal_setcolor(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
        terminal_writeln("Color changed to white");
    } else {
        terminal_writeln("Unknown color. Type 'color' for available colors.");
    }
}

static void cmd_calc(const char* args)
{
    if (strlen(args) == 0) {
        terminal_writeln("Usage: calc <number1> <operator> <number2>");
        terminal_writeln("Operators: +, -, *, /");
        return;
    }
    
    int num1 = 0, num2 = 0;
    char op = '+';
    int result = 0;
    
    // Simple parsing: "5 + 3" format
    int parsed = 0;
    int i = 0;
    while (args[i] == ' ') i++; // Skip spaces
    
    // Parse first number
    while (args[i] >= '0' && args[i] <= '9') {
        num1 = num1 * 10 + (args[i] - '0');
        i++;
        parsed = 1;
    }
    
    while (args[i] == ' ') i++; // Skip spaces
    
    // Parse operator
    if (args[i] == '+' || args[i] == '-' || args[i] == '*' || args[i] == '/') {
        op = args[i];
        i++;
    } else {
        terminal_writeln("Invalid format. Example: calc 5 + 3");
        return;
    }
    
    while (args[i] == ' ') i++; // Skip spaces
    
    // Parse second number
    while (args[i] >= '0' && args[i] <= '9') {
        num2 = num2 * 10 + (args[i] - '0');
        i++;
    }
    
    // Calculate
    switch (op) {
        case '+':
            result = num1 + num2;
            break;
        case '-':
            result = num1 - num2;
            break;
        case '*':
            result = num1 * num2;
            break;
        case '/':
            if (num2 == 0) {
                terminal_writeln("Error: Division by zero!");
                return;
            }
            result = num1 / num2;
            break;
        default:
            terminal_writeln("Invalid operator");
            return;
    }
    
    // Display result
    char result_str[32];
    itoa(result, result_str, 10);
    terminal_writestring("Result: ");
    terminal_writeln(result_str);
}

static void cmd_banner(void)
{
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("  _   _                   _         ___  ____  ");
    terminal_writeln(" | | | |_   _ _ __  _   _| |_ ___  / _ \\/ ___| ");
    terminal_writeln(" | |_| | | | | '_ \\| | | | __/ _ \\| | | \\___ \\ ");
    terminal_writeln(" |  _  | |_| | | | | |_| | || (_) | |_| |___) |");
    terminal_writeln(" |_| |_|\\__, |_| |_|\\__,_|\\__\\___/ \\___/|____/ ");
    terminal_writeln("        |___/                                   ");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("            A Minimal Custom Operating System");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
}

static void cmd_about(void)
{
    terminal_writeln("=== About huggingOs ===");
    terminal_writeln("");
    terminal_writeln("huggingOs is a minimal custom operating system");
    terminal_writeln("built from scratch for educational purposes.");
    terminal_writeln("");
    terminal_writeln("Features:");
    terminal_writeln("  - Multiboot-compliant bootloader");
    terminal_writeln("  - 32-bit x86 kernel");
    terminal_writeln("  - Memory management");
    terminal_writeln("  - VGA text mode graphics");
    terminal_writeln("  - Keyboard input");
    terminal_writeln("  - Interactive shell");
    terminal_writeln("  - Multiple commands");
    terminal_writeln("");
    terminal_writeln("Version: 1.0.0");
    terminal_writeln("Built with dedication and passion!");
}

static void cmd_clock(void)
{
    // Get real time from RTC
    uint8_t hour = rtc_get_hour();
    uint8_t minute = rtc_get_minute();
    uint8_t second = rtc_get_second();
    
    // Format time display
    char hour_str[4];
    char minute_str[4];
    char second_str[4];
    
    itoa(hour, hour_str, 10);
    itoa(minute, minute_str, 10);
    itoa(second, second_str, 10);
    
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("  +-----------+");
    terminal_writestring("  |   ");
    if (hour < 10) terminal_writestring("0");
    terminal_writestring(hour_str);
    terminal_writestring(":");
    if (minute < 10) terminal_writestring("0");
    terminal_writestring(minute_str);
    terminal_writestring(":");
    if (second < 10) terminal_writestring("0");
    terminal_writestring(second_str);
    terminal_writeln("   |");
    terminal_writeln("  +-----------+");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
}

static void cmd_calendar(void)
{
    // Get current date from RTC
    uint16_t year = rtc_get_full_year();
    uint8_t month = rtc_get_month();
    uint8_t day = rtc_get_day();
    
    // Month names
    const char* month_names[] = {
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December"
    };
    
    // Days in each month
    int days_in_month[] = {31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31};
    
    // Check for leap year
    if (month == 2 && ((year % 4 == 0 && year % 100 != 0) || (year % 400 == 0))) {
        days_in_month[1] = 29;
    }
    
    // Calculate day of week for the first day of the month
    // Using Zeller's congruence
    int m = month;
    int y = year;
    if (m < 3) {
        m += 12;
        y--;
    }
    int k = y % 100;
    int j = y / 100;
    int day_of_week = (1 + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
    if (day_of_week < 0) day_of_week += 7;
    
    // Display calendar
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("=== Calendar ===");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("");
    
    char year_str[8];
    itoa(year, year_str, 10);
    terminal_writestring("     ");
    terminal_writestring(month_names[month - 1]);
    terminal_writestring(" ");
    terminal_writeln(year_str);
    
    terminal_writeln("Su Mo Tu We Th Fr Sa");
    
    // Print leading spaces
    for (int i = 0; i < day_of_week; i++) {
        terminal_writestring("   ");
    }
    
    // Print days
    for (int d = 1; d <= days_in_month[month - 1]; d++) {
        char day_str[4];
        itoa(d, day_str, 10);
        
        // Highlight current day
        if (d == day) {
            terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
        }
        
        if (d < 10) {
            terminal_writestring(" ");
        }
        terminal_writestring(day_str);
        
        if (d == day) {
            terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
        }
        
        terminal_writestring(" ");
        
        if ((day_of_week + d) % 7 == 0) {
            terminal_writeln("");
        }
    }
    terminal_writeln("");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
}

static void cmd_date(void)
{
    // Get real date and time from RTC
    rtc_time_t time;
    rtc_get_time(&time);
    uint16_t full_year = rtc_get_full_year();
    
    // Month names
    const char* month_names[] = {
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December"
    };
    
    const char* day_names[] = {
        "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"
    };
    
    // Calculate day of week
    int m = time.month;
    int y = full_year;
    int d = time.day;
    if (m < 3) {
        m += 12;
        y--;
    }
    int k = y % 100;
    int j = y / 100;
    int day_of_week = (d + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
    if (day_of_week < 0) day_of_week += 7;
    
    // Format time
    char hour_str[4], minute_str[4], second_str[4];
    char day_str[4], year_str[8];
    
    itoa(time.hour, hour_str, 10);
    itoa(time.minute, minute_str, 10);
    itoa(time.second, second_str, 10);
    itoa(time.day, day_str, 10);
    itoa(full_year, year_str, 10);
    
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("=== Current Date and Time ===");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("");
    
    terminal_writestring("Date: ");
    terminal_writestring(day_names[day_of_week]);
    terminal_writestring(", ");
    terminal_writestring(month_names[time.month - 1]);
    terminal_writestring(" ");
    terminal_writestring(day_str);
    terminal_writestring(", ");
    terminal_writeln(year_str);
    
    terminal_writestring("Time: ");
    if (time.hour < 10) terminal_writestring("0");
    terminal_writestring(hour_str);
    terminal_writestring(":");
    if (time.minute < 10) terminal_writestring("0");
    terminal_writestring(minute_str);
    terminal_writestring(":");
    if (time.second < 10) terminal_writestring("0");
    terminal_writeln(second_str);
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
}

static void cmd_timer(const char* args)
{
    if (strlen(args) == 0) {
        terminal_writeln("Usage: timer <seconds>");
        terminal_writeln("Example: timer 5");
        return;
    }
    
    int seconds = atoi(args);
    if (seconds <= 0) {
        terminal_writeln("Please provide a positive number of seconds");
        return;
    }
    
    terminal_writestring("Timer set for ");
    char sec_str[32];
    itoa(seconds, sec_str, 10);
    terminal_writestring(sec_str);
    terminal_writeln(" seconds");
    terminal_writeln("Counting down...");
    
    // Use PIT for accurate timing
    uint32_t start_ticks = pit_get_milliseconds();
    uint32_t target_ticks = start_ticks + (seconds * 1000);
    
    int last_second = seconds;
    while (pit_get_milliseconds() < target_ticks) {
        uint32_t elapsed_ms = pit_get_milliseconds() - start_ticks;
        int current_second = seconds - (elapsed_ms / 1000);
        
        if (current_second != last_second && current_second >= 0) {
            char num_str[32];
            itoa(current_second, num_str, 10);
            terminal_writestring(num_str);
            terminal_writestring("... ");
            last_second = current_second;
        }
        
        asm volatile("pause");
    }
    
    terminal_writeln("");
    terminal_writeln("Timer complete!");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writeln("BEEP! BEEP! BEEP!");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
}

static void cmd_uptime(void)
{
    uint32_t seconds = pit_get_seconds();
    uint32_t minutes = seconds / 60;
    uint32_t hours = minutes / 60;
    uint32_t days = hours / 24;
    
    seconds = seconds % 60;
    minutes = minutes % 60;
    hours = hours % 24;
    
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("=== System Uptime ===");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("");
    
    if (days > 0) {
        char day_str[16];
        itoa(days, day_str, 10);
        terminal_writestring("Uptime: ");
        terminal_writestring(day_str);
        terminal_writeln(days == 1 ? " day" : " days");
    }
    
    char hour_str[16], minute_str[16], second_str[16];
    itoa(hours, hour_str, 10);
    itoa(minutes, minute_str, 10);
    itoa(seconds, second_str, 10);
    
    if (days == 0) {
        terminal_writestring("Uptime: ");
    } else {
        terminal_writestring("         ");
    }
    
    terminal_writestring(hour_str);
    terminal_writestring(":");
    if (minutes < 10) terminal_writestring("0");
    terminal_writestring(minute_str);
    terminal_writestring(":");
    if (seconds < 10) terminal_writestring("0");
    terminal_writeln(second_str);
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
}

static void cmd_history(void)
{
    if (shell_history_count == 0) {
        terminal_writeln("No command history available.");
        return;
    }
    
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("=== Command History ===");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    
    int start = 0;
    if (shell_history_count > 20) {
        start = shell_history_count - 20;
    }
    
    for (int i = start; i < shell_history_count; i++) {
        char num_str[16];
        itoa(i + 1, num_str, 10);
        terminal_writestring(num_str);
        terminal_writestring(": ");
        terminal_writeln(shell_history[i]);
    }
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
}

static void cmd_shutdown(void)
{
    terminal_writeln("Shutting down...");
    terminal_writeln("Goodbye!");
    // In a real implementation, we'd trigger ACPI shutdown
    asm volatile("cli");
    asm volatile("hlt");
}

static void cmd_whoami(void)
{
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("Current User: root");
    terminal_writeln("User ID: 0");
    terminal_writeln("System: huggingOS");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
}

static void cmd_pwd(void)
{
    terminal_writeln("/");
    terminal_writeln("Note: File system not yet implemented");
}

static void cmd_mem(void)
{
    extern uint32_t get_total_memory(void);
    extern uint32_t get_free_memory(void);
    
    uint32_t total = get_total_memory();
    uint32_t free = get_free_memory();
    uint32_t used = total - free;
    
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("=== Memory Information ===");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("");
    
    char total_str[32], free_str[32], used_str[32];
    itoa(total / (1024 * 1024), total_str, 10);
    itoa(free / (1024 * 1024), free_str, 10);
    itoa(used / (1024 * 1024), used_str, 10);
    
    terminal_writestring("Total Memory: ");
    terminal_writestring(total_str);
    terminal_writeln(" MB");
    
    terminal_writestring("Used Memory:  ");
    terminal_writestring(used_str);
    terminal_writeln(" MB");
    
    terminal_writestring("Free Memory:  ");
    terminal_writestring(free_str);
    terminal_writeln(" MB");
    
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
}

static void shell_execute_command(const char* command)
{
    if (strlen(command) == 0)
        return;
    
    // Add to history (store a copy)
    if (strlen(command) > 0) {
        // Shift history if full
        if (shell_history_count >= SHELL_HISTORY_SIZE) {
            // Remove oldest entry
            for (int i = 0; i < SHELL_HISTORY_SIZE - 1; i++) {
                strcpy(shell_history[i], shell_history[i + 1]);
            }
            shell_history_count = SHELL_HISTORY_SIZE - 1;
        }
        // Add new entry
        strncpy(shell_history[shell_history_count], command, SHELL_MAX_INPUT - 1);
        shell_history[shell_history_count][SHELL_MAX_INPUT - 1] = '\0';
        shell_history_count++;
    }
    
    // Parse command and arguments
    char cmd[SHELL_MAX_INPUT];
    char args[SHELL_MAX_INPUT] = {0};
    
    int i = 0;
    while (command[i] && command[i] != ' ' && i < SHELL_MAX_INPUT - 1) {
        cmd[i] = command[i];
        i++;
    }
    cmd[i] = '\0';
    
    // Skip spaces
    while (command[i] == ' ')
        i++;
    
    // Get arguments
    if (command[i] != '\0') {
        strcpy(args, &command[i]);
    }
    
    // Execute command
    if (strcmp(cmd, "help") == 0) {
        cmd_help();
    } else if (strcmp(cmd, "clear") == 0 || strcmp(cmd, "cls") == 0) {
        cmd_clear();
    } else if (strcmp(cmd, "info") == 0) {
        cmd_info();
    } else if (strcmp(cmd, "version") == 0) {
        cmd_version();
    } else if (strcmp(cmd, "echo") == 0) {
        cmd_echo(args);
    } else if (strcmp(cmd, "reboot") == 0) {
        cmd_reboot();
    } else if (strcmp(cmd, "color") == 0) {
        cmd_color(args);
    } else if (strcmp(cmd, "calc") == 0) {
        cmd_calc(args);
    } else if (strcmp(cmd, "banner") == 0) {
        cmd_banner();
    } else if (strcmp(cmd, "about") == 0) {
        cmd_about();
    } else if (strcmp(cmd, "clock") == 0) {
        cmd_clock();
    } else if (strcmp(cmd, "calendar") == 0 || strcmp(cmd, "cal") == 0) {
        cmd_calendar();
    } else if (strcmp(cmd, "date") == 0) {
        cmd_date();
    } else if (strcmp(cmd, "timer") == 0) {
        cmd_timer(args);
    } else if (strcmp(cmd, "uptime") == 0) {
        cmd_uptime();
    } else if (strcmp(cmd, "history") == 0 || strcmp(cmd, "hist") == 0) {
        cmd_history();
    } else if (strcmp(cmd, "shutdown") == 0 || strcmp(cmd, "halt") == 0) {
        cmd_shutdown();
    } else if (strcmp(cmd, "whoami") == 0) {
        cmd_whoami();
    } else if (strcmp(cmd, "pwd") == 0) {
        cmd_pwd();
    } else if (strcmp(cmd, "mem") == 0 || strcmp(cmd, "memory") == 0) {
        cmd_mem();
    } else {
        terminal_writestring("Unknown command: ");
        terminal_writeln(cmd);
        terminal_writeln("Type 'help' for a list of commands.");
    }
}

void shell_print_prompt(void)
{
    terminal_writestring(shell_prompt);
}


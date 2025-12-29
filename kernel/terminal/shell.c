#include "terminal.h"
#include "../kernel.h"
#include "../lib/lib.h"
#include "../drivers/drivers.h"
#include "../fs/fs.h"
#include "../sys/logging.h"

#define SHELL_MAX_INPUT 256
#define SHELL_MAX_ARGS 16
#define SHELL_HISTORY_SIZE 20
#define MAX_ENV_VARS 32
#define MAX_ALIASES 16
#define MAX_ENV_NAME 64
#define MAX_ENV_VALUE 128

static char shell_input[SHELL_MAX_INPUT];
static size_t shell_input_pos = 0;
static char shell_prompt[128];
static char shell_history[SHELL_HISTORY_SIZE][SHELL_MAX_INPUT];
static int shell_history_count = 0;

// Environment variables
typedef struct {
    char name[MAX_ENV_NAME];
    char value[MAX_ENV_VALUE];
} env_var_t;
static env_var_t env_vars[MAX_ENV_VARS];
static int env_count = 0;

// Aliases
typedef struct {
    char name[32];
    char command[128];
} alias_t;
static alias_t aliases[MAX_ALIASES];
static int alias_count = 0;

static int shell_exit_code = 0;
static bool shell_exit_flag = false;

static void shell_execute_command(const char* command);
static void shell_update_prompt(void);
void shell_setenv(const char* name, const char* value);

void shell_init(void)
{
    shell_input_pos = 0;
    shell_history_count = 0;
    env_count = 0;
    alias_count = 0;
    shell_exit_code = 0;
    shell_exit_flag = false;
    memset(shell_input, 0, SHELL_MAX_INPUT);
    memset(shell_history, 0, sizeof(shell_history));
    memset(env_vars, 0, sizeof(env_vars));
    memset(aliases, 0, sizeof(aliases));
    
    // Set default environment variables
    shell_setenv("USER", "root");
    shell_setenv("HOME", "/");
    shell_setenv("SHELL", "/bin/sh");
    shell_setenv("PATH", "/bin:/usr/bin");
    shell_setenv("PWD", "/");
    
    shell_update_prompt();
}

void shell_setenv(const char* name, const char* value)
{
    // Check if exists
    for (int i = 0; i < env_count; i++) {
        if (strcmp(env_vars[i].name, name) == 0) {
            strncpy(env_vars[i].value, value, MAX_ENV_VALUE - 1);
            env_vars[i].value[MAX_ENV_VALUE - 1] = '\0';
            return;
        }
    }
    
    // Add new
    if (env_count < MAX_ENV_VARS) {
        strncpy(env_vars[env_count].name, name, MAX_ENV_NAME - 1);
        env_vars[env_count].name[MAX_ENV_NAME - 1] = '\0';
        strncpy(env_vars[env_count].value, value, MAX_ENV_VALUE - 1);
        env_vars[env_count].value[MAX_ENV_VALUE - 1] = '\0';
        env_count++;
    }
}

const char* shell_getenv(const char* name)
{
    for (int i = 0; i < env_count; i++) {
        if (strcmp(env_vars[i].name, name) == 0) {
            return env_vars[i].value;
        }
    }
    return NULL;
}

static void shell_update_prompt(void)
{
    extern void ramfs_get_full_path(uint32_t entry_id, char* path, uint32_t max_len);
    extern uint32_t ramfs_get_current_dir(void);
    
    char path[128];
    uint32_t current = ramfs_get_current_dir();
    ramfs_get_full_path(current, path, 128);
    
    const char* user = shell_getenv("USER");
    if (!user) user = "root";
    
    // Create a colorful, modern prompt (store as plain text, colors applied when printing)
    strcpy(shell_prompt, user);
    strcat(shell_prompt, "@huggingOS:");
    strcat(shell_prompt, path);
    strcat(shell_prompt, "$ ");
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
        shell_update_prompt();
        shell_print_prompt();
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
    terminal_writeln("╔════════════════════════════════════════════════════════════╗");
    terminal_writeln("║  huggingOS v1.0.0 - Production Ready Edition            ║");
    terminal_writeln("╚════════════════════════════════════════════════════════════╝");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writeln("  [*] System Commands:");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("    help      - Show this help message");
    terminal_writeln("    clear     - Clear the screen");
    terminal_writeln("    info      - Show system information");
    terminal_writeln("    version   - Show OS version");
    terminal_writeln("    reboot    - Reboot the system");
    terminal_writeln("    shutdown  - Shutdown the system");
    terminal_writeln("    whoami    - Show current user");
    terminal_writeln("    uname     - Show system information");
    terminal_writeln("    exit      - Exit shell");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writeln("  [*] Date & Time:");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("    date      - Show date and time");
    terminal_writeln("    clock     - Show current time");
    terminal_writeln("    calendar  - Show calendar");
    terminal_writeln("    timer     - Start a timer (seconds)");
    terminal_writeln("    uptime    - Show system uptime");
    terminal_writeln("    sleep     - Sleep for N seconds");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writeln("  [*] File System:");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("    ls        - List directory contents");
    terminal_writeln("    mkdir     - Create directory");
    terminal_writeln("    cd        - Change directory");
    terminal_writeln("    pwd       - Print working directory");
    terminal_writeln("    cat       - Display file contents");
    terminal_writeln("    touch     - Create empty file");
    terminal_writeln("    rm        - Remove file or directory");
    terminal_writeln("    mv        - Move/rename file");
    terminal_writeln("    cp        - Copy file");
    terminal_writeln("    find      - Find files");
    terminal_writeln("    df        - Show filesystem usage");
    terminal_writeln("    du        - Show directory usage");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writeln("  [*] Text Processing:");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("    grep      - Search text in files");
    terminal_writeln("    wc        - Word count");
    terminal_writeln("    head      - Show first N lines");
    terminal_writeln("    tail      - Show last N lines");
    terminal_writeln("    sort      - Sort file lines");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writeln("  [*] Utilities:");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("    echo      - Echo text back (use > for file)");
    terminal_writeln("    calc      - Simple calculator");
    terminal_writeln("    color     - Change terminal color");
    terminal_writeln("    banner    - Show ASCII art banner");
    terminal_writeln("    about     - About huggingOS");
    terminal_writeln("    history   - Show command history");
    terminal_writeln("    mem       - Show memory information");
    terminal_writeln("    env       - Show environment variables");
    terminal_writeln("    export    - Set environment variable");
    terminal_writeln("    alias     - Create command alias");
    terminal_writeln("    unalias   - Remove command alias");
    terminal_writeln("    test      - Test condition");
    terminal_writeln("    true      - Return success");
    terminal_writeln("    false     - Return failure");
    terminal_writeln("    basename  - Get filename from path");
    terminal_writeln("    dirname   - Get directory from path");
    terminal_writeln("    which     - Find command location");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writeln("  [*] Fun Commands:");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("    moti      - Show motivational quote");
    terminal_writeln("    joke      - Tell a random joke");
    terminal_writeln("    fortune   - Show fortune cookie");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("════════════════════════════════════════════════════════════");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writeln("  >> Tip: Type 'help <command>' for detailed usage");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("════════════════════════════════════════════════════════════");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
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
    uint32_t uptime_days = uptime_hours / 24;
    uptime_hours = uptime_hours % 24;
    
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("╔════════════════════════════════════════════════════════════╗");
    terminal_writeln("║         huggingOS System Information                     ║");
    terminal_writeln("╚════════════════════════════════════════════════════════════╝");
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writestring("  OS Name:        ");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("huggingOS");
    
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writestring("  Version:        ");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("1.0.0 (Production Ready)");
    
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writestring("  Architecture:   ");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("x86 (32-bit)");
    
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writestring("  Kernel:         ");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("Monolithic");
    
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writestring("  Graphics:       ");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("VGA Text Mode");
    
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writestring("  Features:       ");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("Terminal, Shell, Keyboard, RTC, PIT Timer");
    
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writestring("  Uptime:         ");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    char hour_str[16], minute_str[16], day_str[16];
    itoa(uptime_days, day_str, 10);
    itoa(uptime_hours, hour_str, 10);
    itoa(uptime_minutes, minute_str, 10);
    if (uptime_days > 0) {
        terminal_writestring(day_str);
        terminal_writestring("d ");
    }
    terminal_writestring(hour_str);
    terminal_writestring("h ");
    terminal_writestring(minute_str);
    terminal_writeln("m");
    
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writestring("  Status:         ");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("[OPERATIONAL]");
    
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
    terminal_writeln("");
    terminal_writeln("  ╔═══════════════════════════════════════════════════════╗");
    terminal_writeln("  ║                                                       ║");
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writeln("  ║     _   _                   _         ___  ____       ║");
    terminal_writeln("  ║    | | | |_   _ _ __  _   _| |_ ___  / _ \\/ ___|      ║");
    terminal_writeln("  ║    | |_| | | | | '_ \\| | | | __/ _ \\| | | \\___ \\      ║");
    terminal_writeln("  ║    |  _  | |_| | | | | |_| | || (_) | |_| |___) |    ║");
    terminal_writeln("  ║    |_| |_|\\__, |_| |_|\\__,_|\\__\\___/ \\___/|____/     ║");
    terminal_writeln("  ║            |___/                                       ║");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("  ║                                                       ║");
    terminal_writeln("  ╚═══════════════════════════════════════════════════════╝");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writeln("");
    terminal_writeln("            A Minimal Custom Operating System");
    terminal_writeln("              Version 1.0.0 - Production Ready");
    terminal_writeln("");
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
    extern void ramfs_get_full_path(uint32_t entry_id, char* path, uint32_t max_len);
    extern uint32_t ramfs_get_current_dir(void);
    
    char path[256];
    uint32_t current = ramfs_get_current_dir();
    ramfs_get_full_path(current, path, 256);
    terminal_writeln(path);
}

// Essential File System Commands
static void cmd_ls(const char* args)
{
    uint32_t dir_id = ramfs_get_current_dir();
    if (args && strlen(args) > 0) {
        uint32_t target = ramfs_find_path(args);
        if (target != 256 && ramfs_entry_is_directory(target)) {
            dir_id = target;
        } else {
            terminal_writeln("Directory not found");
            return;
        }
    }
    
    uint32_t entries[16];
    uint32_t count = ramfs_list_directory(dir_id, entries, 16);
    
    if (count == 0) {
        terminal_writeln("Directory is empty");
        return;
    }
    
    for (uint32_t i = 0; i < count; i++) {
        uint32_t entry_id = entries[i];
        if (ramfs_entry_is_directory(entry_id)) {
            terminal_setcolor(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
            terminal_writestring(ramfs_entry_get_name(entry_id));
            terminal_writeln("/");
        } else {
            terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
            terminal_writestring(ramfs_entry_get_name(entry_id));
            char size_str[32];
            itoa(ramfs_entry_get_size(entry_id), size_str, 10);
            terminal_writestring(" (");
            terminal_writestring(size_str);
            terminal_writeln(" bytes)");
        }
    }
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
}

static void cmd_mkdir(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: mkdir <directory_name>");
        return;
    }
    
    uint32_t dir_id = ramfs_create_directory(args);
    if (dir_id == 256) {
        terminal_writeln("Error: Could not create directory");
    } else {
        terminal_writestring("Directory created: ");
        terminal_writeln(args);
    }
}

static void cmd_cd(const char* args)
{
    if (!args || strlen(args) == 0 || strcmp(args, "~") == 0) {
        ramfs_change_directory(0); // Go to root
        shell_setenv("PWD", "/");
        shell_update_prompt();
        return;
    }
    
    if (strcmp(args, "..") == 0) {
        uint32_t current = ramfs_get_current_dir();
        uint32_t parent = ramfs_entry_get_parent(current);
        if (parent != current && parent != 256) {
            ramfs_change_directory(parent);
            char path[128];
            ramfs_get_full_path(parent, path, 128);
            shell_setenv("PWD", path);
            shell_update_prompt();
        }
        return;
    }
    
    uint32_t dir_id = ramfs_find_path(args);
    if (dir_id == 256) {
        terminal_writeln("Directory not found");
    } else {
        if (!ramfs_entry_is_directory(dir_id)) {
            terminal_writeln("Not a directory");
        } else {
            ramfs_change_directory(dir_id);
            // Update PWD environment variable
            char path[128];
            ramfs_get_full_path(dir_id, path, 128);
            shell_setenv("PWD", path);
            shell_update_prompt();
        }
    }
}

static void cmd_cat(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: cat <filename>");
        return;
    }
    
    uint32_t file_id = ramfs_find_path(args);
    if (file_id == 256) {
        terminal_writeln("File not found");
        return;
    }
    
    if (ramfs_entry_is_directory(file_id)) {
        terminal_writeln("Cannot cat a directory");
        return;
    }
    
    if (ramfs_entry_get_size(file_id) == 0) {
        terminal_writeln("File is empty");
        return;
    }
    
    uint8_t buffer[1024];
    uint32_t read = ramfs_read_file(file_id, buffer, 1024);
    if (read > 0) {
        for (uint32_t i = 0; i < read && buffer[i] != '\0'; i++) {
            terminal_writechar(buffer[i]);
        }
        terminal_writeln("");
    }
}

static void cmd_touch(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: touch <filename>");
        return;
    }
    
    uint32_t file_id = ramfs_create_file(args);
    if (file_id == 256) {
        terminal_writeln("Error: Could not create file");
    } else {
        terminal_writestring("File created: ");
        terminal_writeln(args);
    }
}

static void cmd_rm(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: rm <filename_or_directory>");
        return;
    }
    
    uint32_t entry_id = ramfs_find_path(args);
    if (entry_id == 256) {
        terminal_writeln("File or directory not found");
        return;
    }
    
    if (ramfs_delete_entry(entry_id)) {
        terminal_writestring("Deleted: ");
        terminal_writeln(args);
    } else {
        terminal_writeln("Error: Could not delete");
    }
}

static void cmd_mv(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: mv <source> <destination>");
        return;
    }
    
    // Parse source and destination
    char src[256], dst[256];
    int i = 0, j = 0;
    
    // Get source
    while (args[i] && args[i] != ' ' && j < 255) {
        src[j++] = args[i++];
    }
    src[j] = '\0';
    
    // Skip spaces
    while (args[i] == ' ') i++;
    
    // Get destination
    j = 0;
    while (args[i] && j < 255) {
        dst[j++] = args[i++];
    }
    dst[j] = '\0';
    
    if (strlen(dst) == 0) {
        terminal_writeln("Usage: mv <source> <destination>");
        return;
    }
    
    uint32_t src_id = ramfs_find_path(src);
    if (src_id == 256) {
        terminal_writeln("Source not found");
        return;
    }
    
    if (src_id != 256) {
        // Simple rename - just change the name
        ramfs_entry_set_name(src_id, dst);
        terminal_writestring("Renamed: ");
        terminal_writestring(src);
        terminal_writestring(" -> ");
        terminal_writeln(dst);
    }
}

static void cmd_cp(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: cp <source> <destination>");
        return;
    }
    
    // Parse arguments (simplified)
    char src[256], dst[256];
    int i = 0, j = 0;
    while (args[i] && args[i] != ' ' && j < 255) {
        src[j++] = args[i++];
    }
    src[j] = '\0';
    while (args[i] == ' ') i++;
    j = 0;
    while (args[i] && j < 255) {
        dst[j++] = args[i++];
    }
    dst[j] = '\0';
    
    if (strlen(dst) == 0) {
        terminal_writeln("Usage: cp <source> <destination>");
        return;
    }
    
    uint32_t src_id = ramfs_find_path(src);
    if (src_id == 256) {
        terminal_writeln("Source not found");
        return;
    }
    
    if (ramfs_entry_is_directory(src_id)) {
        terminal_writeln("Cannot copy directory (use mkdir)");
        return;
    }
    
    uint32_t dst_id = ramfs_create_file(dst);
    if (dst_id == 256) {
        terminal_writeln("Could not create destination file");
        return;
    }
    
    uint32_t src_size = ramfs_entry_get_size(src_id);
    uint8_t* src_data = ramfs_entry_get_data(src_id);
    if (src_size > 0 && src_data) {
        ramfs_write_file(dst_id, src_data, src_size);
    }
    
    terminal_writestring("Copied: ");
    terminal_writestring(src);
    terminal_writestring(" -> ");
    terminal_writeln(dst);
}

// Enhanced echo with file redirection
static void cmd_echo_file(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: echo \"text\" > filename");
        return;
    }
    
    // Look for >
    char* redirect = strchr(args, '>');
    if (!redirect) {
        // No redirection, just echo
        cmd_echo(args);
        return;
    }
    
    // Parse text and filename
    *redirect = '\0';
    redirect++;
    while (*redirect == ' ') redirect++;
    
    // Remove quotes from text if present
    char text_buffer[256];
    strcpy(text_buffer, args);
    char* text = text_buffer;
    if (text[0] == '"' && text[strlen(text)-1] == '"') {
        text[strlen(text)-1] = '\0';
        text++;
    }
    
    uint32_t file_id = ramfs_create_file(redirect);
    if (file_id == 256) {
        terminal_writeln("Error: Could not create file");
        return;
    }
    
    ramfs_write_file(file_id, (uint8_t*)text, strlen(text));
    terminal_writestring("Written to: ");
    terminal_writeln(redirect);
}

// Fun Commands
static void cmd_moti(void)
{
    const char* quotes[] = {
        "The only way to do great work is to love what you do. - Steve Jobs",
        "Innovation distinguishes between a leader and a follower. - Steve Jobs",
        "Life is what happens to you while you're busy making other plans. - John Lennon",
        "The future belongs to those who believe in the beauty of their dreams. - Eleanor Roosevelt",
        "It is during our darkest moments that we must focus to see the light. - Aristotle",
        "Success is not final, failure is not fatal: it is the courage to continue that counts. - Winston Churchill",
        "The only impossible journey is the one you never begin. - Tony Robbins",
        "In the middle of difficulty lies opportunity. - Albert Einstein",
        "Believe you can and you're halfway there. - Theodore Roosevelt",
        "It does not matter how slowly you go as long as you do not stop. - Confucius"
    };
    
    // Simple "random" selection using current time
    uint8_t second = rtc_get_second();
    uint32_t index = second % 10;
    
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writeln("=== Daily Motivation ===");
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("");
    terminal_writeln(quotes[index]);
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
}

static void cmd_joke(void)
{
    const char* jokes[] = {
        "Why don't programmers like nature? It has too many bugs!",
        "How do you comfort a JavaScript bug? You console it!",
        "Why do Java developers wear glasses? Because they can't C#!",
        "A SQL query walks into a bar, walks up to two tables and asks: 'Can I join you?'",
        "Why did the developer go broke? Because he used up all his cache!",
        "What's a programmer's favorite hangout place? Foo Bar!",
        "Why do Python programmers prefer dark mode? Because light attracts bugs!",
        "How many programmers does it take to change a light bulb? None, that's a hardware problem!",
        "Why did the programmer quit his job? He didn't get arrays!",
        "What do you call a programmer from Finland? Nerdic!"
    };
    
    uint8_t second = rtc_get_second();
    uint32_t index = second % 10;
    
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writeln("=== Random Joke ===");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writeln("");
    terminal_writeln(jokes[index]);
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
}

static void cmd_fortune(void)
{
    const char* fortunes[] = {
        "A journey of a thousand miles begins with a single step.",
        "Today is a good day to learn something new.",
        "Your hard work will pay off soon.",
        "Opportunity knocks but once.",
        "The best time to plant a tree was 20 years ago. The second best time is now.",
        "You are capable of amazing things.",
        "Success is the sum of small efforts repeated day in and day out.",
        "The only way to do great work is to love what you do.",
        "Your limitation—it's only your imagination.",
        "Great things never come from comfort zones."
    };
    
    uint8_t minute = rtc_get_minute();
    uint32_t index = minute % 10;
    
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("=== Fortune Cookie ===");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writeln("");
    terminal_writeln(fortunes[index]);
    terminal_writeln("");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
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

// Production Commands
static void cmd_grep(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: grep <pattern> <file>");
        return;
    }
    
    char pattern[128], filename[128];
    int i = 0, j = 0;
    
    // Parse pattern
    while (args[i] && args[i] != ' ' && j < 127) {
        pattern[j++] = args[i++];
    }
    pattern[j] = '\0';
    
    // Skip spaces
    while (args[i] == ' ') i++;
    
    // Parse filename
    j = 0;
    while (args[i] && j < 127) {
        filename[j++] = args[i++];
    }
    filename[j] = '\0';
    
    if (strlen(filename) == 0) {
        terminal_writeln("Usage: grep <pattern> <file>");
        return;
    }
    
    uint32_t file_id = ramfs_find_path(filename);
    if (file_id == 256) {
        terminal_writeln("File not found");
        return;
    }
    
    if (ramfs_entry_is_directory(file_id)) {
        terminal_writeln("Cannot grep a directory");
        return;
    }
    
    if (ramfs_entry_get_size(file_id) == 0) {
        return; // Empty file
    }
    
    uint8_t buffer[2048];
    uint32_t read = ramfs_read_file(file_id, buffer, 2048);
    if (read > 0) {
        buffer[read] = '\0';
        char* line_start = (char*)buffer;
        char* line_end;
        int line_num = 1;
        
        while ((line_end = strchr(line_start, '\n')) != NULL || (line_start < (char*)buffer + read)) {
            if (line_end) {
                *line_end = '\0';
            }
            
            if (strstr(line_start, pattern) != NULL) {
                char num_str[16];
                itoa(line_num, num_str, 10);
                terminal_writestring(filename);
                terminal_writestring(":");
                terminal_writestring(num_str);
                terminal_writestring(":");
                terminal_writeln(line_start);
            }
            
            if (line_end) {
                line_start = line_end + 1;
            } else {
                break;
            }
            line_num++;
        }
    }
}

static void cmd_find(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: find <directory> -name <pattern>");
        terminal_writeln("       find <name> (searches current directory)");
        return;
    }
    
    char pattern[128] = {0};
    
    // Simple find: just search for name in current directory
    if (strstr(args, "-name") == NULL) {
        // Simple search
        strncpy(pattern, args, 127);
        pattern[127] = '\0';
    } else {
        // Parse -name pattern
        char* name_pos = strstr(args, "-name");
        if (name_pos) {
            name_pos += 5; // Skip "-name"
            while (*name_pos == ' ') name_pos++;
            strncpy(pattern, name_pos, 127);
            pattern[127] = '\0';
        }
    }
    
    uint32_t dir_id = ramfs_get_current_dir();
    uint32_t entries[16];
    uint32_t count = ramfs_list_directory(dir_id, entries, 16);
    
    bool found = false;
    for (uint32_t i = 0; i < count; i++) {
        uint32_t entry_id = entries[i];
        const char* entry_name = ramfs_entry_get_name(entry_id);
        if (entry_name && strstr(entry_name, pattern) != NULL) {
            char path[256];
            ramfs_get_full_path(entries[i], path, 256);
            terminal_writeln(path);
            found = true;
        }
    }
    
    if (!found) {
        terminal_writeln("No matches found");
    }
}

static void cmd_wc(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: wc <file>");
        return;
    }
    
    uint32_t file_id = ramfs_find_path(args);
    if (file_id == 256) {
        terminal_writeln("File not found");
        return;
    }
    
    if (ramfs_entry_is_directory(file_id)) {
        terminal_writeln("Cannot count a directory");
        return;
    }
    
    uint8_t buffer[2048];
    uint32_t read = ramfs_read_file(file_id, buffer, 2048);
    
    int lines = 0, words = 0, chars = read;
    bool in_word = false;
    
    for (uint32_t i = 0; i < read; i++) {
        if (buffer[i] == '\n') lines++;
        if (buffer[i] == ' ' || buffer[i] == '\t' || buffer[i] == '\n') {
            if (in_word) {
                words++;
                in_word = false;
            }
        } else {
            in_word = true;
        }
    }
    if (in_word) words++;
    
    char lines_str[32], words_str[32], chars_str[32];
    itoa(lines, lines_str, 10);
    itoa(words, words_str, 10);
    itoa(chars, chars_str, 10);
    
    terminal_writestring(lines_str);
    terminal_writestring(" ");
    terminal_writestring(words_str);
    terminal_writestring(" ");
    terminal_writestring(chars_str);
    terminal_writestring(" ");
    terminal_writeln(args);
}

static void cmd_head(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: head <file> [lines]");
        return;
    }
    
    int num_lines = 10;
    char filename[128];
    
    // Parse arguments
    char args_copy[256];
    strcpy(args_copy, args);
    char* token = strtok(args_copy, " ");
    strcpy(filename, token);
    
    token = strtok(NULL, " ");
    if (token) {
        num_lines = atoi(token);
        if (num_lines <= 0) num_lines = 10;
    }
    
    uint32_t file_id = ramfs_find_path(filename);
    if (file_id == 256) {
        terminal_writeln("File not found");
        return;
    }
    
    if (ramfs_entry_is_directory(file_id)) {
        terminal_writeln("Cannot head a directory");
        return;
    }
    
    uint8_t buffer[2048];
    uint32_t read = ramfs_read_file(file_id, buffer, 2048);
    if (read > 0) {
        buffer[read] = '\0';
        int lines = 0;
        for (uint32_t i = 0; i < read && lines < num_lines; i++) {
            terminal_writechar(buffer[i]);
            if (buffer[i] == '\n') lines++;
        }
    }
}

static void cmd_tail(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: tail <file> [lines]");
        return;
    }
    
    int num_lines = 10;
    char filename[128];
    
    char args_copy[256];
    strcpy(args_copy, args);
    char* token = strtok(args_copy, " ");
    strcpy(filename, token);
    
    token = strtok(NULL, " ");
    if (token) {
        num_lines = atoi(token);
        if (num_lines <= 0) num_lines = 10;
    }
    
    uint32_t file_id = ramfs_find_path(filename);
    if (file_id == 256) {
        terminal_writeln("File not found");
        return;
    }
    
    if (ramfs_entry_is_directory(file_id)) {
        terminal_writeln("Cannot tail a directory");
        return;
    }
    
    uint8_t buffer[2048];
    uint32_t read = ramfs_read_file(file_id, buffer, 2048);
    if (read > 0) {
        buffer[read] = '\0';
        
        // Count lines
        int total_lines = 0;
        for (uint32_t i = 0; i < read; i++) {
            if (buffer[i] == '\n') total_lines++;
        }
        
        // Find start position
        int lines_to_skip = total_lines - num_lines;
        if (lines_to_skip < 0) lines_to_skip = 0;
        
        int current_line = 0;
        uint32_t start_pos = 0;
        for (uint32_t i = 0; i < read; i++) {
            if (buffer[i] == '\n') {
                current_line++;
                if (current_line > lines_to_skip) {
                    start_pos = i + 1;
                    break;
                }
            }
        }
        
        // Print from start_pos
        for (uint32_t i = start_pos; i < read; i++) {
            terminal_writechar(buffer[i]);
        }
    }
}

static void cmd_sort(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: sort <file>");
        return;
    }
    
    uint32_t file_id = ramfs_find_path(args);
    if (file_id == 256) {
        terminal_writeln("File not found");
        return;
    }
    
    if (ramfs_entry_is_directory(file_id)) {
        terminal_writeln("Cannot sort a directory");
        return;
    }
    
    uint8_t buffer[2048];
    uint32_t read = ramfs_read_file(file_id, buffer, 2048);
    if (read > 0) {
        buffer[read] = '\0';
        
        // Simple bubble sort of lines
        char* lines[256];
        int line_count = 0;
        char* line_start = (char*)buffer;
        
        lines[line_count++] = line_start;
        for (uint32_t i = 0; i < read; i++) {
            if (buffer[i] == '\n') {
                buffer[i] = '\0';
                if (i + 1 < read) {
                    lines[line_count++] = (char*)buffer + i + 1;
                }
            }
        }
        
        // Sort lines
        for (int i = 0; i < line_count - 1; i++) {
            for (int j = 0; j < line_count - i - 1; j++) {
                if (strcmp(lines[j], lines[j + 1]) > 0) {
                    char* temp = lines[j];
                    lines[j] = lines[j + 1];
                    lines[j + 1] = temp;
                }
            }
        }
        
        // Print sorted lines
        for (int i = 0; i < line_count; i++) {
            terminal_writeln(lines[i]);
        }
    }
}

static void cmd_uname(const char* args)
{
    bool show_all = false;
    if (args && strstr(args, "-a") != NULL) {
        show_all = true;
    }
    
    if (show_all || !args || strlen(args) == 0) {
        terminal_writestring("huggingOS ");
    }
    if (show_all || (args && strstr(args, "-n") != NULL)) {
        terminal_writeln("huggingOS");
    }
    if (show_all || (args && strstr(args, "-r") != NULL)) {
        terminal_writeln("1.0.0");
    }
    if (show_all || (args && strstr(args, "-m") != NULL)) {
        terminal_writeln("i686");
    }
    if (show_all) {
        terminal_writeln("huggingOS huggingOS 1.0.0 i686");
    }
}

static void cmd_sleep(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: sleep <seconds>");
        return;
    }
    
    int seconds = atoi(args);
    if (seconds <= 0) {
        terminal_writeln("Please provide a positive number of seconds");
        return;
    }
    
    uint32_t start_ticks = pit_get_milliseconds();
    uint32_t target_ticks = start_ticks + (seconds * 1000);
    
    while (pit_get_milliseconds() < target_ticks) {
        asm volatile("pause");
    }
}

static void cmd_exit(const char* args)
{
    int exit_code = 0;
    if (args && strlen(args) > 0) {
        exit_code = atoi(args);
    }
    shell_exit_code = exit_code;
    shell_exit_flag = true;
    terminal_writeln("Exiting shell...");
}

static void cmd_env(void)
{
    for (int i = 0; i < env_count; i++) {
        terminal_writestring(env_vars[i].name);
        terminal_writestring("=");
        terminal_writeln(env_vars[i].value);
    }
}

static void cmd_export(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: export <VAR>=<value>");
        return;
    }
    
    char* eq_pos = strchr(args, '=');
    if (!eq_pos) {
        terminal_writeln("Usage: export <VAR>=<value>");
        return;
    }
    
    char name[64];
    int name_len = eq_pos - args;
    if (name_len >= 64) name_len = 63;
    strncpy(name, args, name_len);
    name[name_len] = '\0';
    
    const char* value = eq_pos + 1;
    shell_setenv(name, value);
    
    terminal_writestring("Exported: ");
    terminal_writestring(name);
    terminal_writestring("=");
    terminal_writeln(value);
}

static void cmd_alias(const char* args)
{
    if (!args || strlen(args) == 0) {
        // List all aliases
        if (alias_count == 0) {
            terminal_writeln("No aliases defined");
        } else {
            for (int i = 0; i < alias_count; i++) {
                terminal_writestring(aliases[i].name);
                terminal_writestring("='");
                terminal_writestring(aliases[i].command);
                terminal_writeln("'");
            }
        }
        return;
    }
    
    char* eq_pos = strchr(args, '=');
    if (!eq_pos) {
        terminal_writeln("Usage: alias <name>='<command>'");
        return;
    }
    
    char name[32];
    int name_len = eq_pos - args;
    if (name_len >= 32) name_len = 31;
    strncpy(name, args, name_len);
    name[name_len] = '\0';
    
    // Skip = and quotes
    const char* cmd_start = eq_pos + 1;
    if (*cmd_start == '\'' || *cmd_start == '"') cmd_start++;
    
    char command[128];
    strncpy(command, cmd_start, 127);
    command[127] = '\0';
    
    // Remove trailing quote
    int cmd_len = strlen(command);
    if (cmd_len > 0 && (command[cmd_len - 1] == '\'' || command[cmd_len - 1] == '"')) {
        command[cmd_len - 1] = '\0';
    }
    
    // Check if alias exists
    for (int i = 0; i < alias_count; i++) {
        if (strcmp(aliases[i].name, name) == 0) {
            strncpy(aliases[i].command, command, 127);
            aliases[i].command[127] = '\0';
            return;
        }
    }
    
    // Add new alias
    if (alias_count < MAX_ALIASES) {
        strncpy(aliases[alias_count].name, name, 31);
        aliases[alias_count].name[31] = '\0';
        strncpy(aliases[alias_count].command, command, 127);
        aliases[alias_count].command[127] = '\0';
        alias_count++;
    }
}

static void cmd_unalias(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: unalias <alias_name>");
        return;
    }
    
    for (int i = 0; i < alias_count; i++) {
        if (strcmp(aliases[i].name, args) == 0) {
            // Shift remaining aliases
            for (int j = i; j < alias_count - 1; j++) {
                strcpy(aliases[j].name, aliases[j + 1].name);
                strcpy(aliases[j].command, aliases[j + 1].command);
            }
            alias_count--;
            terminal_writestring("Removed alias: ");
            terminal_writeln(args);
            return;
        }
    }
    
    terminal_writestring("Alias not found: ");
    terminal_writeln(args);
}

static void cmd_df(void)
{
    extern uint32_t get_total_memory(void);
    extern uint32_t get_free_memory(void);
    
    uint32_t total = get_total_memory();
    uint32_t free = get_free_memory();
    uint32_t used = total - free;
    
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("Filesystem      Size      Used      Avail     Use%");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    
    char total_str[32], used_str[32], free_str[32];
    itoa(total / (1024 * 1024), total_str, 10);
    itoa(used / (1024 * 1024), used_str, 10);
    itoa(free / (1024 * 1024), free_str, 10);
    
    int use_percent = (used * 100) / total;
    char percent_str[16];
    itoa(use_percent, percent_str, 10);
    
    terminal_writestring("ramfs           ");
    terminal_writestring(total_str);
    terminal_writestring("M       ");
    terminal_writestring(used_str);
    terminal_writestring("M       ");
    terminal_writestring(free_str);
    terminal_writestring("M       ");
    terminal_writestring(percent_str);
    terminal_writeln("%");
}

static void cmd_du(const char* args)
{
    uint32_t dir_id = ramfs_get_current_dir();
    if (args && strlen(args) > 0) {
        uint32_t target = ramfs_find_path(args);
        if (target != 256 && ramfs_entry_is_directory(target)) {
            dir_id = target;
        }
    }
    
    // Calculate directory size (simplified)
    uint32_t entries[16];
    uint32_t count = ramfs_list_directory(dir_id, entries, 16);
    
    uint32_t total_size = 0;
    for (uint32_t i = 0; i < count; i++) {
        uint32_t entry_id = entries[i];
        total_size += ramfs_entry_get_size(entry_id);
    }
    
    char size_str[32];
    itoa(total_size / 1024, size_str, 10);
    
    char path[256];
    ramfs_get_full_path(dir_id, path, 256);
    
    terminal_writestring(size_str);
    terminal_writestring("K    ");
    terminal_writeln(path);
}

static void cmd_test(const char* args)
{
    if (!args || strlen(args) == 0) {
        shell_exit_code = 1;
        return;
    }
    
    // Simple test: check if file exists
    uint32_t file_id = ramfs_find_path(args);
    if (file_id != 256) {
        shell_exit_code = 0;
    } else {
        shell_exit_code = 1;
    }
}

static void cmd_true(void)
{
    shell_exit_code = 0;
}

static void cmd_false(void)
{
    shell_exit_code = 1;
}

static void cmd_basename(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: basename <path>");
        return;
    }
    
    char* last_slash = strrchr(args, '/');
    if (last_slash) {
        terminal_writeln(last_slash + 1);
    } else {
        terminal_writeln(args);
    }
}

static void cmd_dirname(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: dirname <path>");
        return;
    }
    
    char path_copy[256];
    strcpy(path_copy, args);
    char* last_slash = strrchr(path_copy, '/');
    if (last_slash) {
        *last_slash = '\0';
        if (strlen(path_copy) == 0) {
            terminal_writeln("/");
        } else {
            terminal_writeln(path_copy);
        }
    } else {
        terminal_writeln(".");
    }
}

static void cmd_dmesg(const char* args)
{
    extern void log_print_all(void);
    extern void log_clear(void);
    
    if (args && (strcmp(args, "-c") == 0 || strcmp(args, "--clear") == 0)) {
        log_clear();
        terminal_writeln("System log cleared");
        return;
    }
    
    log_print_all();
}

static void cmd_which(const char* args)
{
    if (!args || strlen(args) == 0) {
        terminal_writeln("Usage: which <command>");
        return;
    }
    
    // Check if it's a builtin
    const char* builtins[] = {
        "help", "clear", "info", "version", "echo", "reboot", "color", "calc",
        "banner", "about", "clock", "calendar", "date", "timer", "uptime",
        "history", "shutdown", "whoami", "pwd", "mem", "ls", "mkdir", "cd",
        "cat", "touch", "rm", "mv", "cp", "moti", "joke", "fortune", "grep",
        "find", "wc", "head", "tail", "sort", "uname", "sleep", "exit", "env",
        "export", "alias", "unalias", "df", "du", "test", "true", "false",
        "basename", "dirname", "which", NULL
    };
    
    for (int i = 0; builtins[i]; i++) {
        if (strcmp(builtins[i], args) == 0) {
            terminal_writestring(args);
            terminal_writeln(": shell builtin");
            return;
        }
    }
    
    terminal_writestring(args);
    terminal_writeln(": not found");
}

static const char* shell_resolve_alias(const char* cmd)
{
    for (int i = 0; i < alias_count; i++) {
        if (strcmp(aliases[i].name, cmd) == 0) {
            return aliases[i].command;
        }
    }
    return NULL;
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
    
    // Check for alias
    const char* alias_cmd = shell_resolve_alias(cmd);
    if (alias_cmd) {
        // Execute alias command
        char full_cmd[256];
        strcpy(full_cmd, alias_cmd);
        if (strlen(args) > 0) {
            strcat(full_cmd, " ");
            strcat(full_cmd, args);
        }
        shell_execute_command(full_cmd);
        return;
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
    } else if (strcmp(cmd, "ls") == 0) {
        cmd_ls(args);
    } else if (strcmp(cmd, "mkdir") == 0) {
        cmd_mkdir(args);
    } else if (strcmp(cmd, "cd") == 0) {
        cmd_cd(args);
        shell_update_prompt();
    } else if (strcmp(cmd, "cat") == 0) {
        cmd_cat(args);
    } else if (strcmp(cmd, "touch") == 0) {
        cmd_touch(args);
    } else if (strcmp(cmd, "rm") == 0) {
        cmd_rm(args);
    } else if (strcmp(cmd, "mv") == 0) {
        cmd_mv(args);
    } else if (strcmp(cmd, "cp") == 0) {
        cmd_cp(args);
    } else if (strcmp(cmd, "moti") == 0 || strcmp(cmd, "motivation") == 0) {
        cmd_moti();
    } else if (strcmp(cmd, "joke") == 0) {
        cmd_joke();
    } else if (strcmp(cmd, "fortune") == 0) {
        cmd_fortune();
    } else if (strcmp(cmd, "grep") == 0) {
        cmd_grep(args);
    } else if (strcmp(cmd, "find") == 0) {
        cmd_find(args);
    } else if (strcmp(cmd, "wc") == 0) {
        cmd_wc(args);
    } else if (strcmp(cmd, "head") == 0) {
        cmd_head(args);
    } else if (strcmp(cmd, "tail") == 0) {
        cmd_tail(args);
    } else if (strcmp(cmd, "sort") == 0) {
        cmd_sort(args);
    } else if (strcmp(cmd, "uname") == 0) {
        cmd_uname(args);
    } else if (strcmp(cmd, "sleep") == 0) {
        cmd_sleep(args);
    } else if (strcmp(cmd, "exit") == 0) {
        cmd_exit(args);
    } else if (strcmp(cmd, "env") == 0) {
        cmd_env();
    } else if (strcmp(cmd, "export") == 0) {
        cmd_export(args);
    } else if (strcmp(cmd, "alias") == 0) {
        cmd_alias(args);
    } else if (strcmp(cmd, "unalias") == 0) {
        cmd_unalias(args);
    } else if (strcmp(cmd, "df") == 0) {
        cmd_df();
    } else if (strcmp(cmd, "du") == 0) {
        cmd_du(args);
    } else if (strcmp(cmd, "test") == 0 || strcmp(cmd, "[") == 0) {
        cmd_test(args);
    } else if (strcmp(cmd, "true") == 0) {
        cmd_true();
    } else if (strcmp(cmd, "false") == 0) {
        cmd_false();
    } else if (strcmp(cmd, "basename") == 0) {
        cmd_basename(args);
    } else if (strcmp(cmd, "dirname") == 0) {
        cmd_dirname(args);
    } else if (strcmp(cmd, "which") == 0) {
        cmd_which(args);
    } else if (strcmp(cmd, "dmesg") == 0 || strcmp(cmd, "log") == 0) {
        cmd_dmesg(args);
    } else {
        // Check if echo has file redirection
        if (strcmp(cmd, "echo") == 0 && strchr(args, '>') != NULL) {
            cmd_echo_file(args);
        } else {
            terminal_writestring("Unknown command: ");
            terminal_writeln(cmd);
            terminal_writeln("Type 'help' for a list of commands.");
        }
    }
}

void shell_print_prompt(void)
{
    // Print prompt with colors
    const char* user = shell_getenv("USER");
    if (!user) user = "root";
    
    terminal_setcolor(VGA_COLOR_LIGHT_MAGENTA, VGA_COLOR_BLACK);
    terminal_writestring(user);
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writestring("@");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    terminal_writestring("huggingOS");
    terminal_setcolor(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
    terminal_writestring(":");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    
    extern void ramfs_get_full_path(uint32_t entry_id, char* path, uint32_t max_len);
    extern uint32_t ramfs_get_current_dir(void);
    char path[128];
    uint32_t current = ramfs_get_current_dir();
    ramfs_get_full_path(current, path, 128);
    terminal_writestring(path);
    
    terminal_setcolor(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
    terminal_writestring("$ ");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
}

bool shell_should_exit(void)
{
    return shell_exit_flag;
}

int shell_get_exit_code(void)
{
    return shell_exit_code;
}


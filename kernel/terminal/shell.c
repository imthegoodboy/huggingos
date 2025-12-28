#include "terminal.h"
#include "../kernel.h"
#include "../lib/lib.h"

#define SHELL_MAX_INPUT 256
#define SHELL_MAX_ARGS 16

static char shell_input[SHELL_MAX_INPUT];
static size_t shell_input_pos = 0;
static char shell_prompt[] = "huggingOs> ";

static void shell_execute_command(const char* command);

void shell_init(void)
{
    shell_input_pos = 0;
    memset(shell_input, 0, SHELL_MAX_INPUT);
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
    terminal_writeln("huggingOs - Available Commands:");
    terminal_writeln("  help      - Show this help message");
    terminal_writeln("  clear     - Clear the screen");
    terminal_writeln("  info      - Show system information");
    terminal_writeln("  echo      - Echo text back");
    terminal_writeln("  version   - Show OS version");
    terminal_writeln("  reboot    - Reboot the system");
    terminal_writeln("  color     - Change terminal color");
    terminal_writeln("  calc      - Simple calculator");
    terminal_writeln("  banner    - Show ASCII art banner");
    terminal_writeln("  about     - About huggingOs");
    terminal_writeln("  clock     - Show current time");
    terminal_writeln("  calendar  - Show calendar");
    terminal_writeln("  date      - Show date and time");
    terminal_writeln("  timer     - Start a timer (seconds)");
}

static void cmd_clear(void)
{
    terminal_clear();
    terminal_writestring(shell_prompt);
}

static void cmd_info(void)
{
    terminal_writeln("=== huggingOs System Information ===");
    terminal_writeln("OS Name: huggingOs");
    terminal_writeln("Version: 1.0.0");
    terminal_writeln("Architecture: x86 (32-bit)");
    terminal_writeln("Kernel: Monolithic");
    terminal_writeln("Memory: Available");
    terminal_writeln("Graphics: VGA Text Mode");
    terminal_writeln("Features: Terminal, Shell, Keyboard");
    terminal_writeln("Status: Operational");
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
    // Simple clock display (simulated since we don't have RTC access)
    terminal_writeln("Current Time: [System Clock]");
    terminal_writeln("Note: Real-time clock requires RTC driver");
    terminal_writeln("This is a demonstration clock feature.");
    
    // Show a simple clock display
    terminal_setcolor(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
    terminal_writeln("  +-------+");
    terminal_writeln("  | 12:00 |");
    terminal_writeln("  +-------+");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
}

static void cmd_calendar(void)
{
    // Simple calendar display
    terminal_writeln("=== Calendar ===");
    terminal_writeln("");
    terminal_writeln("    December 2024");
    terminal_writeln("Su Mo Tu We Th Fr Sa");
    terminal_writeln(" 1  2  3  4  5  6  7");
    terminal_writeln(" 8  9 10 11 12 13 14");
    terminal_writeln("15 16 17 18 19 20 21");
    terminal_writeln("22 23 24 25 26 27 28");
    terminal_writeln("29 30 31");
    terminal_writeln("");
    terminal_writeln("Note: Full calendar requires date/time support");
}

static void cmd_date(void)
{
    terminal_writeln("Current Date and Time:");
    terminal_writeln("Date: December 27, 2024");
    terminal_writeln("Time: [System Time]");
    terminal_writeln("");
    terminal_writeln("Note: Real date/time requires RTC driver");
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
    
    // Simple countdown (in a real OS, this would use proper timing)
    for (int i = seconds; i > 0; i--) {
        char num_str[32];
        itoa(i, num_str, 10);
        terminal_writestring(num_str);
        terminal_writestring("... ");
        
        // Simple delay loop (not accurate, but demonstrates the feature)
        volatile int delay = 0;
        for (volatile int j = 0; j < 1000000; j++) {
            delay++;
        }
    }
    
    terminal_writeln("");
    terminal_writeln("Timer complete!");
    terminal_setcolor(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    terminal_writeln("BEEP! BEEP! BEEP!");
    terminal_setcolor(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
}

static void shell_execute_command(const char* command)
{
    if (strlen(command) == 0)
        return;
    
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


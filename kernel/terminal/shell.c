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
    terminal_write("Result: ");
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
    } else {
        terminal_write("Unknown command: ");
        terminal_writeln(cmd);
        terminal_writeln("Type 'help' for a list of commands.");
    }
}

void shell_print_prompt(void)
{
    terminal_writestring(shell_prompt);
}


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
    terminal_writeln("  help    - Show this help message");
    terminal_writeln("  clear   - Clear the screen");
    terminal_writeln("  info    - Show system information");
    terminal_writeln("  echo    - Echo text back");
    terminal_writeln("  version - Show OS version");
    terminal_writeln("  reboot  - Reboot the system");
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
    terminal_writeln("Graphics: VGA/VESA");
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


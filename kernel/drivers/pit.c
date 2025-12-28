#include "drivers.h"
#include "../interrupts.h"
#include "../lib/lib.h"

// PIT I/O ports
#define PIT_CHANNEL0 0x40
#define PIT_CHANNEL1 0x41
#define PIT_CHANNEL2 0x42
#define PIT_COMMAND  0x43

// PIT command byte format
#define PIT_CHANNEL0_SEL 0x00
#define PIT_ACCESS_LOHI  0x30
#define PIT_MODE_3       0x06  // Square wave generator

// PIT frequency: 1193182 Hz
#define PIT_BASE_FREQ 1193182
#define PIT_TARGET_FREQ 1000  // 1000 Hz = 1ms per tick

static volatile uint32_t pit_ticks = 0;
static bool pit_initialized = false;

void pit_init(void)
{
    // Calculate divisor for ~1ms ticks (1000 Hz)
    uint16_t divisor = PIT_BASE_FREQ / PIT_TARGET_FREQ;
    
    // Send command byte: channel 0, access mode lobyte/hibyte, mode 3
    outb(PIT_COMMAND, PIT_CHANNEL0_SEL | PIT_ACCESS_LOHI | PIT_MODE_3);
    
    // Send divisor (low byte, then high byte)
    outb(PIT_CHANNEL0, divisor & 0xFF);
    outb(PIT_CHANNEL0, (divisor >> 8) & 0xFF);
    
    pit_ticks = 0;
    pit_initialized = true;
}

void pit_handler(void)
{
    pit_ticks++;
}

uint32_t pit_get_ticks(void)
{
    return pit_ticks;
}

// Get milliseconds since boot
uint32_t pit_get_milliseconds(void)
{
    return pit_ticks;
}

// Get seconds since boot
uint32_t pit_get_seconds(void)
{
    return pit_ticks / 1000;
}

// Simple delay function (approximate, in milliseconds)
void pit_delay_ms(uint32_t milliseconds)
{
    uint32_t start = pit_get_ticks();
    while ((pit_get_ticks() - start) < milliseconds) {
        asm volatile("pause");
    }
}

bool pit_is_initialized(void)
{
    return pit_initialized;
}





#include "kernel.h"
#include "interrupts.h"

struct idt_entry idt[256];
struct idt_ptr idtp;

void idt_set_gate(unsigned char num, unsigned long base, unsigned short sel, unsigned char flags)
{
    idt[num].base_lo = (base & 0xFFFF);
    idt[num].base_hi = (base >> 16) & 0xFFFF;
    idt[num].sel = sel;
    idt[num].always0 = 0;
    idt[num].flags = flags;
}

extern void idt_flush();

void idt_init()
{
    idtp.limit = (sizeof(struct idt_entry) * 256) - 1;
    idtp.base = (unsigned int)&idt;

    memset(&idt, 0, sizeof(struct idt_entry) * 256);

    // Remap PIC
    outb(0x20, 0x11);
    outb(0xA0, 0x11);
    outb(0x21, 0x20);
    outb(0xA1, 0x28);
    outb(0x21, 0x04);
    outb(0xA1, 0x02);
    outb(0x21, 0x01);
    outb(0xA1, 0x01);
    outb(0x21, 0x0);
    outb(0xA1, 0x0);

    idt_flush();
}

void isr_handler(registers_t regs)
{
    if (regs.int_no < 32) {
        // Handle exception
        kernel_panic("Exception occurred");
    }
}

void irq_handler(registers_t regs)
{
    // Handle keyboard interrupt
    if (regs.int_no >= 32) {
        unsigned char irq = regs.int_no - 32;
        if (irq == 1) {
            // Keyboard interrupt
            unsigned char scancode = inb(0x60);
            extern void keyboard_handler(uint8_t);
            keyboard_handler(scancode);
        }
        
        // Send EOI to PIC
        if (regs.int_no >= 40) {
            outb(0xA0, 0x20);
        }
        outb(0x20, 0x20);
    }
}

unsigned char inb(unsigned short port)
{
    unsigned char ret;
    asm volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

void outb(unsigned short port, unsigned char data)
{
    asm volatile("outb %0, %1" : : "a"(data), "Nd"(port));
}

void remap_pic()
{
    // Already done in idt_init
}


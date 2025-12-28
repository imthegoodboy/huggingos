#ifndef INTERRUPTS_H
#define INTERRUPTS_H

#include <stdint.h>

struct idt_entry
{
    unsigned short base_lo;
    unsigned short sel;
    unsigned char always0;
    unsigned char flags;
    unsigned short base_hi;
} __attribute__((packed));

struct idt_ptr
{
    unsigned short limit;
    unsigned int base;
} __attribute__((packed));

typedef struct {
    uint32_t ds;
    uint32_t edi, esi, ebp, esp, ebx, edx, ecx, eax;
    uint32_t int_no, err_code;
    uint32_t eip, cs, eflags, useresp, ss;
} registers_t;

void idt_init();
void isr_handler(registers_t regs);
void irq_handler(registers_t regs);
unsigned char inb(unsigned short port);
void outb(unsigned short port, unsigned char data);
void remap_pic();

#endif



#include "memory.h"
#include "../lib/lib.h"
#include "../kernel.h"

page_directory_t* current_directory = 0;
uint32_t mem_size = 0x4000000; // 64MB default

void paging_init()
{
    // Paging is intentionally disabled for this flat 32-bit kernel.
    // The heap uses physical addresses below the reported memory limit.
    current_directory = 0;
}

page_directory_t* paging_get_directory()
{
    return current_directory;
}

void paging_map_page(void* virtual_address, void* physical_address)
{
    UNUSED(virtual_address);
    UNUSED(physical_address);
    // Simplified - full implementation would map pages
}

void memory_init(uint32_t size)
{
    mem_size = size;
    paging_init();
    heap_init(size);
}

uint32_t get_total_memory(void)
{
    return mem_size;
}

uint32_t get_free_memory(void)
{
    extern uint32_t heap_get_used(void);
    uint32_t reserved = 0x100000 + heap_get_used();
    if (reserved >= mem_size) {
        return 0;
    }
    return mem_size - reserved;
}


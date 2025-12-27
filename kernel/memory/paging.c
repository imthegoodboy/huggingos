#include "memory.h"
#include "../lib/lib.h"

page_directory_t* current_directory = 0;
uint32_t mem_size = 0x4000000; // 64MB default

void paging_init()
{
    // Simplified paging initialization
    // In a full implementation, we'd set up page tables
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
}

uint32_t get_total_memory(void)
{
    return mem_size;
}

uint32_t get_free_memory(void)
{
    return mem_size - (heap_place_static - KHEAP_START);
}


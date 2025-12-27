#include "memory.h"
#include "../lib/lib.h"
#include "../kernel.h"

static uint32_t heap_place = KHEAP_START;
static uint32_t heap_max = KHEAP_START + KHEAP_INITIAL_SIZE;

typedef struct ordered_array {
    void** array;
    uint32_t size;
    uint32_t max_size;
    int (*predicate)(void*, void*);
} ordered_array_t;

int standard_lessthan_predicate(void* a, void* b)
{
    return (uint32_t)a < (uint32_t)b;
}

ordered_array_t heap_index;

void heap_init()
{
    uint32_t end = (uint32_t)heap_place + KHEAP_INITIAL_SIZE;
    heap_max = end;
    
    // Simple placeholder - full implementation would use ordered_array
    // For now, use a simple allocation strategy
}

uint32_t find_smallest_hole(uint32_t size, uint8_t page_align)
{
    // Simplified implementation
    // In a full OS, this would search the heap index
    return heap_place;
}

void expand(uint32_t new_size)
{
    if (new_size <= heap_max - heap_place)
        return;
    
    // For now, just extend heap_max
    heap_max = heap_place + new_size;
}

uint32_t contract(uint32_t new_size)
{
    if (new_size >= heap_max - heap_place)
        return;
    
    heap_max = heap_place + new_size;
    return new_size;
}

void* kmalloc_int(uint32_t size, int align, uint32_t* phys)
{
    if (heap_place == 0) {
        heap_init();
    }
    
    uint32_t new_location = heap_place;
    
    if (align == 1 && (heap_place & 0xFFFFF000)) {
        new_location &= 0xFFFFF000;
        new_location += 0x1000;
    }
    
    if (phys) {
        *phys = new_location;
    }
    
    uint32_t old_location = heap_place;
    heap_place = new_location + size;
    
    if (heap_place > heap_max) {
        expand(heap_max + size);
    }
    
    return (void*)new_location;
}

void* kmalloc(uint32_t size)
{
    return kmalloc_int(size, 0, 0);
}

void* kmalloc_a(uint32_t size)
{
    return kmalloc_int(size, 1, 0);
}

void* kmalloc_p(uint32_t size, uint32_t* phys)
{
    return kmalloc_int(size, 0, phys);
}

void* kmalloc_ap(uint32_t size, uint32_t* phys)
{
    return kmalloc_int(size, 1, phys);
}

void kfree(void* p)
{
    // Simplified - in a full implementation, we'd mark the block as free
    // For now, we'll just leave it (memory leak for simplicity)
    UNUSED(p);
}


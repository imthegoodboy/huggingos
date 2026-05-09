#include "memory.h"
#include "../lib/lib.h"
#include "../kernel.h"

#define HEAP_BLOCK_MAGIC 0xC0DEF00D
#define HEAP_ALIGN       8

typedef struct heap_block {
    uint32_t magic;
    uint32_t size;
    uint8_t free;
    struct heap_block* next;
    struct heap_block* prev;
} heap_block_t;

extern uint32_t kernel_end;

static heap_block_t* heap_head = NULL;
static uint32_t heap_start = 0;
static uint32_t heap_limit = 0;
static uint32_t heap_total = 0;
static uint32_t heap_used = 0;
static bool heap_ready = false;

static uint32_t align_up(uint32_t value, uint32_t alignment)
{
    return (value + alignment - 1) & ~(alignment - 1);
}

static uint32_t payload_address(heap_block_t* block)
{
    return (uint32_t)block + sizeof(heap_block_t);
}

static void split_block(heap_block_t* block, uint32_t wanted_size)
{
    uint32_t min_split_size = sizeof(heap_block_t) + HEAP_ALIGN;

    if (block->size <= wanted_size + min_split_size) {
        return;
    }

    heap_block_t* next = (heap_block_t*)(payload_address(block) + wanted_size);
    next->magic = HEAP_BLOCK_MAGIC;
    next->size = block->size - wanted_size - sizeof(heap_block_t);
    next->free = true;
    next->next = block->next;
    next->prev = block;

    if (next->next) {
        next->next->prev = next;
    }

    block->size = wanted_size;
    block->next = next;
}

static void merge_with_next(heap_block_t* block)
{
    heap_block_t* next = block->next;
    if (!next || !next->free || next->magic != HEAP_BLOCK_MAGIC) {
        return;
    }

    block->size += sizeof(heap_block_t) + next->size;
    block->next = next->next;
    if (block->next) {
        block->next->prev = block;
    }
}

static heap_block_t* align_block_for_payload(heap_block_t* block)
{
    uint32_t block_start = (uint32_t)block;
    uint32_t block_end = payload_address(block) + block->size;
    uint32_t aligned_payload = align_up(payload_address(block), PAGE_SIZE);
    uint32_t aligned_header = aligned_payload - sizeof(heap_block_t);
    uint32_t prefix_size = aligned_header - block_start;

    if (prefix_size > 0 && prefix_size < sizeof(heap_block_t) + HEAP_ALIGN) {
        aligned_payload += PAGE_SIZE;
        aligned_header = aligned_payload - sizeof(heap_block_t);
        prefix_size = aligned_header - block_start;
    }

    if (aligned_payload >= block_end || prefix_size >= block_end - block_start) {
        return NULL;
    }

    if (prefix_size == 0) {
        return block;
    }

    if (prefix_size < sizeof(heap_block_t) + HEAP_ALIGN) {
        return NULL;
    }

    heap_block_t* next = block->next;
    heap_block_t* aligned = (heap_block_t*)aligned_header;

    block->size = prefix_size - sizeof(heap_block_t);
    block->next = aligned;

    aligned->magic = HEAP_BLOCK_MAGIC;
    aligned->size = block_end - aligned_payload;
    aligned->free = true;
    aligned->prev = block;
    aligned->next = next;

    if (next) {
        next->prev = aligned;
    }

    return aligned;
}

static heap_block_t* find_free_block(uint32_t size)
{
    heap_block_t* block = heap_head;
    while (block) {
        if (block->magic == HEAP_BLOCK_MAGIC && block->free && block->size >= size) {
            return block;
        }
        block = block->next;
    }
    return NULL;
}

static heap_block_t* find_free_aligned_block(uint32_t size)
{
    heap_block_t* block = heap_head;
    while (block) {
        if (block->magic == HEAP_BLOCK_MAGIC && block->free) {
            heap_block_t* aligned = align_block_for_payload(block);
            if (aligned && aligned->size >= size) {
                return aligned;
            }
        }
        block = block->next;
    }
    return NULL;
}

void heap_init(uint32_t mem_size)
{
    uint32_t start = align_up((uint32_t)&kernel_end, HEAP_ALIGN);

    if (start < KHEAP_START) {
        start = KHEAP_START;
    }

    if (mem_size <= start + sizeof(heap_block_t) + HEAP_ALIGN) {
        heap_ready = false;
        heap_head = NULL;
        heap_start = start;
        heap_limit = start;
        heap_total = 0;
        heap_used = 0;
        return;
    }

    heap_start = start;
    heap_limit = mem_size & ~(HEAP_ALIGN - 1);
    heap_total = heap_limit - heap_start;
    heap_used = 0;

    heap_head = (heap_block_t*)heap_start;
    heap_head->magic = HEAP_BLOCK_MAGIC;
    heap_head->size = heap_total - sizeof(heap_block_t);
    heap_head->free = true;
    heap_head->next = NULL;
    heap_head->prev = NULL;
    heap_ready = true;
}

void* kmalloc_int(uint32_t size, int align, uint32_t* phys)
{
    if (size == 0) {
        return NULL;
    }

    if (!heap_ready) {
        heap_init(0x4000000);
    }

    uint32_t wanted_size = align_up(size, HEAP_ALIGN);

    heap_block_t* block = align == 1 ?
        find_free_aligned_block(wanted_size) :
        find_free_block(wanted_size);
    if (!block) {
        return NULL;
    }

    split_block(block, wanted_size);
    block->free = false;
    heap_used += block->size;

    void* payload = (void*)payload_address(block);

    if (phys) {
        *phys = (uint32_t)payload;
    }

    return payload;
}

void* kmalloc(uint32_t size)
{
    return kmalloc_int(size, 0, NULL);
}

void* kmalloc_a(uint32_t size)
{
    return kmalloc_int(size, 1, NULL);
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
    if (!p || !heap_ready) {
        return;
    }

    heap_block_t* block = (heap_block_t*)((uint32_t)p - sizeof(heap_block_t));
    if (block->magic != HEAP_BLOCK_MAGIC || block->free) {
        return;
    }

    block->free = true;
    if (heap_used >= block->size) {
        heap_used -= block->size;
    } else {
        heap_used = 0;
    }

    merge_with_next(block);
    if (block->prev && block->prev->free) {
        merge_with_next(block->prev);
    }
}

uint32_t heap_get_used(void)
{
    return heap_used;
}

uint32_t heap_get_total(void)
{
    return heap_total;
}

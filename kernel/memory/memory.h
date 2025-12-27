#ifndef MEMORY_H
#define MEMORY_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define PAGE_SIZE 4096
#define KHEAP_START 0xC0000000
#define KHEAP_INITIAL_SIZE 0x100000
#define HEAP_INDEX_SIZE 0x20000
#define HEAP_MAGIC 0x123890AB
#define HEAP_MIN_SIZE 0x70000

typedef struct {
    uint32_t magic;
    uint8_t is_hole;
    uint32_t size;
} heap_header_t;

typedef struct {
    uint32_t magic;
    heap_header_t* header;
} heap_footer_t;

typedef struct {
    uint32_t start_address;
    uint32_t end_address;
    uint32_t max_address;
    uint8_t supervisor;
    uint8_t readonly;
} page_t;

typedef struct {
    uint32_t total_frames;
    uint32_t* frames;
    page_t* pages;
} page_directory_t;

void memory_init(uint32_t mem_size);
void* kmalloc(size_t size);
void kfree(void* ptr);
void* krealloc(void* ptr, size_t size);
uint32_t get_total_memory(void);
uint32_t get_free_memory(void);

// Page management
page_directory_t* paging_get_directory(void);
void paging_init(void);
void paging_map_page(void* virtual_address, void* physical_address);

#endif


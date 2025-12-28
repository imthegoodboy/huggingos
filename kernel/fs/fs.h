#ifndef FS_H
#define FS_H

#include <stdint.h>
#include <stdbool.h>

// RAMFS file system functions
void ramfs_init(void);
uint32_t ramfs_create_file(const char* path);
uint32_t ramfs_create_directory(const char* path);
uint32_t ramfs_find_path(const char* path);
bool ramfs_write_file(uint32_t file_id, const uint8_t* data, uint32_t size);
uint32_t ramfs_read_file(uint32_t file_id, uint8_t* buffer, uint32_t max_size);
bool ramfs_delete_entry(uint32_t entry_id);
bool ramfs_change_directory(uint32_t dir_id);
uint32_t ramfs_get_current_dir(void);
uint32_t ramfs_list_directory(uint32_t dir_id, uint32_t* buffer, uint32_t max_count);
void ramfs_get_full_path(uint32_t entry_id, char* path, uint32_t max_len);

// File system entry structure (forward declaration)
typedef struct ramfs_entry ramfs_entry_t;

ramfs_entry_t* ramfs_get_entry(uint32_t entry_id);

// Accessor functions for entry fields
bool ramfs_entry_is_directory(uint32_t entry_id);
uint32_t ramfs_entry_get_size(uint32_t entry_id);
uint32_t ramfs_entry_get_parent(uint32_t entry_id);
const char* ramfs_entry_get_name(uint32_t entry_id);
uint8_t* ramfs_entry_get_data(uint32_t entry_id);
bool ramfs_entry_set_name(uint32_t entry_id, const char* name);

#endif


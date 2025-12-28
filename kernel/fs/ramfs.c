#include "fs.h"
#include "../lib/lib.h"
#include "../memory/memory.h"

#define MAX_FILES 256
#define MAX_DIRS 64
#define MAX_FILENAME 64
#define MAX_PATH 256

// File system entry structure (full definition)
typedef struct ramfs_entry {
    char name[MAX_FILENAME];
    uint8_t* data;
    uint32_t size;
    uint32_t capacity;
    bool is_directory;
    uint32_t parent_dir;
    uint32_t num_children;
    uint32_t children[16]; // Max 16 entries per directory
} ramfs_entry_t;

static ramfs_entry_t filesystem[MAX_FILES];
static uint32_t num_entries = 0;
static uint32_t current_dir = 0;
static bool fs_initialized = false;

void ramfs_init(void)
{
    if (fs_initialized) return;
    
    // Initialize root directory
    ramfs_entry_t* root = &filesystem[0];
    memset(root->name, 0, MAX_FILENAME);
    strcpy(root->name, "/");
    root->data = NULL;
    root->size = 0;
    root->capacity = 0;
    root->is_directory = true;
    root->parent_dir = 0; // Root is its own parent
    root->num_children = 0;
    memset(root->children, 0, sizeof(root->children));
    
    num_entries = 1;
    current_dir = 0;
    fs_initialized = true;
}

static uint32_t ramfs_find_entry_in_dir(uint32_t dir_id, const char* name)
{
    ramfs_entry_t* dir = &filesystem[dir_id];
    
    for (uint32_t i = 0; i < dir->num_children; i++) {
        uint32_t child_id = dir->children[i];
        if (strcmp(filesystem[child_id].name, name) == 0) {
            return child_id;
        }
    }
    
    return MAX_FILES; // Not found
}

static uint32_t ramfs_get_free_entry(void)
{
    for (uint32_t i = 0; i < MAX_FILES; i++) {
        if (filesystem[i].name[0] == '\0') {
            return i;
        }
    }
    return MAX_FILES; // No free entry
}

uint32_t ramfs_create_file(const char* path)
{
    if (!fs_initialized) ramfs_init();
    
    // Simple implementation: handle only current directory files for now
    char path_copy[MAX_PATH];
    strcpy(path_copy, path);
    
    // Remove leading slashes
    while (*path_copy == '/') {
        memmove(path_copy, path_copy + 1, strlen(path_copy));
    }
    
    // Simple: create in current directory
    uint32_t parent_dir = current_dir;
    const char* filename = path_copy;
    
    // Handle empty filename
    if (strlen(filename) == 0) {
        return MAX_FILES;
    }
    
    // Check if file already exists
    uint32_t existing = ramfs_find_entry_in_dir(parent_dir, filename);
    if (existing != MAX_FILES) {
        return existing; // File exists, return it
    }
    
    // Create new file
    uint32_t new_id = ramfs_get_free_entry();
    if (new_id == MAX_FILES) return MAX_FILES;
    
    ramfs_entry_t* new_file = &filesystem[new_id];
    memset(new_file->name, 0, MAX_FILENAME);
    strncpy(new_file->name, filename, MAX_FILENAME - 1);
    new_file->data = NULL;
    new_file->size = 0;
    new_file->capacity = 0;
    new_file->is_directory = false;
    new_file->parent_dir = parent_dir;
    new_file->num_children = 0;
    
    // Add to parent directory
    ramfs_entry_t* parent = &filesystem[parent_dir];
    if (parent->num_children < 16) {
        parent->children[parent->num_children++] = new_id;
    }
    
    num_entries++;
    return new_id;
}

uint32_t ramfs_create_directory(const char* path)
{
    if (!fs_initialized) ramfs_init();
    
    // Similar to create_file but mark as directory
    uint32_t dir_id = ramfs_create_file(path);
    if (dir_id != MAX_FILES) {
        filesystem[dir_id].is_directory = true;
        filesystem[dir_id].num_children = 0;
    }
    return dir_id;
}

uint32_t ramfs_find_path(const char* path)
{
    if (!fs_initialized) ramfs_init();
    
    char path_copy[MAX_PATH];
    strcpy(path_copy, path);
    
    uint32_t dir = current_dir;
    
    // Handle absolute path
    if (path_copy[0] == '/') {
        dir = 0;
        if (path_copy[1] == '\0') return 0; // Root directory
    }
    
    // Parse path components
    char* token = strtok(path_copy + (path_copy[0] == '/' ? 1 : 0), "/");
    
    while (token) {
        uint32_t next = ramfs_find_entry_in_dir(dir, token);
        if (next == MAX_FILES) return MAX_FILES;
        dir = next;
        token = strtok(NULL, "/");
    }
    
    return dir;
}

bool ramfs_write_file(uint32_t file_id, const uint8_t* data, uint32_t size)
{
    if (file_id >= MAX_FILES || filesystem[file_id].is_directory) return false;
    
    ramfs_entry_t* file = &filesystem[file_id];
    
    // Allocate or reallocate buffer
    if (file->capacity < size) {
        if (file->data) {
            // Reallocate (simple version - just allocate new)
            file->data = (uint8_t*)kmalloc(size + 1024);
        } else {
            file->data = (uint8_t*)kmalloc(size + 1024);
        }
        file->capacity = size + 1024;
    }
    
    memcpy(file->data, data, size);
    file->size = size;
    return true;
}

uint32_t ramfs_read_file(uint32_t file_id, uint8_t* buffer, uint32_t max_size)
{
    if (file_id >= MAX_FILES || filesystem[file_id].is_directory) return 0;
    
    ramfs_entry_t* file = &filesystem[file_id];
    uint32_t to_read = (max_size < file->size) ? max_size : file->size;
    
    if (file->data && to_read > 0) {
        memcpy(buffer, file->data, to_read);
    }
    
    return to_read;
}

bool ramfs_delete_entry(uint32_t entry_id)
{
    if (entry_id == 0 || entry_id >= MAX_FILES) return false; // Can't delete root
    
    ramfs_entry_t* entry = &filesystem[entry_id];
    
    // Delete all children if it's a directory
    if (entry->is_directory) {
        for (uint32_t i = 0; i < entry->num_children; i++) {
            ramfs_delete_entry(entry->children[i]);
        }
    }
    
    // Free data
    if (entry->data) {
        kfree(entry->data);
    }
    
    // Remove from parent
    if (entry->parent_dir < MAX_FILES) {
        ramfs_entry_t* parent = &filesystem[entry->parent_dir];
        for (uint32_t i = 0; i < parent->num_children; i++) {
            if (parent->children[i] == entry_id) {
                // Shift remaining children
                for (uint32_t j = i; j < parent->num_children - 1; j++) {
                    parent->children[j] = parent->children[j + 1];
                }
                parent->num_children--;
                break;
            }
        }
    }
    
    // Clear entry
    memset(entry, 0, sizeof(ramfs_entry_t));
    
    // Update current directory if we deleted it
    if (current_dir == entry_id) {
        current_dir = 0;
    }
    
    num_entries--;
    return true;
}

bool ramfs_change_directory(uint32_t dir_id)
{
    if (dir_id >= MAX_FILES || !filesystem[dir_id].is_directory) return false;
    current_dir = dir_id;
    return true;
}

uint32_t ramfs_get_current_dir(void)
{
    return current_dir;
}

ramfs_entry_t* ramfs_get_entry(uint32_t entry_id)
{
    if (entry_id >= MAX_FILES) return NULL;
    return &filesystem[entry_id];
}

uint32_t ramfs_list_directory(uint32_t dir_id, uint32_t* buffer, uint32_t max_count)
{
    if (dir_id >= MAX_FILES || !filesystem[dir_id].is_directory) return 0;
    
    ramfs_entry_t* dir = &filesystem[dir_id];
    uint32_t count = (dir->num_children < max_count) ? dir->num_children : max_count;
    
    for (uint32_t i = 0; i < count; i++) {
        buffer[i] = dir->children[i];
    }
    
    return count;
}

void ramfs_get_full_path(uint32_t entry_id, char* path, uint32_t max_len)
{
    if (entry_id == 0) {
        strcpy(path, "/");
        return;
    }
    
    char components[16][MAX_FILENAME];
    uint32_t depth = 0;
    uint32_t current = entry_id;
    
    // Build path backwards
    while (current != 0 && depth < 16) {
        strcpy(components[depth], filesystem[current].name);
        current = filesystem[current].parent_dir;
        depth++;
    }
    
    // Build full path
    path[0] = '\0';
    strcat(path, "/");
    
    for (int32_t i = depth - 1; i >= 0; i--) {
        if (strlen(path) + strlen(components[i]) + 2 < max_len) {
            if (path[strlen(path) - 1] != '/') strcat(path, "/");
            strcat(path, components[i]);
        }
    }
}

bool ramfs_entry_is_directory(uint32_t entry_id)
{
    if (entry_id >= MAX_FILES) return false;
    return filesystem[entry_id].is_directory;
}

uint32_t ramfs_entry_get_size(uint32_t entry_id)
{
    if (entry_id >= MAX_FILES) return 0;
    return filesystem[entry_id].size;
}

uint32_t ramfs_entry_get_parent(uint32_t entry_id)
{
    if (entry_id >= MAX_FILES) return MAX_FILES;
    return filesystem[entry_id].parent_dir;
}

const char* ramfs_entry_get_name(uint32_t entry_id)
{
    if (entry_id >= MAX_FILES) return NULL;
    return filesystem[entry_id].name;
}

uint8_t* ramfs_entry_get_data(uint32_t entry_id)
{
    if (entry_id >= MAX_FILES) return NULL;
    return filesystem[entry_id].data;
}

bool ramfs_entry_set_name(uint32_t entry_id, const char* name)
{
    if (entry_id >= MAX_FILES) return false;
    strncpy(filesystem[entry_id].name, name, MAX_FILENAME - 1);
    filesystem[entry_id].name[MAX_FILENAME - 1] = '\0';
    return true;
}


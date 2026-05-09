#include "fs.h"
#include "../lib/lib.h"
#include "../memory/memory.h"

#define MAX_FILES 256
#define MAX_DIRS 64
#define MAX_FILENAME 64
#define MAX_PATH 256
#define MAX_CHILDREN 64
#define RAMFS_NOT_FOUND MAX_FILES

// File system entry structure (full definition)
typedef struct ramfs_entry {
    char name[MAX_FILENAME];
    uint8_t* data;
    uint32_t size;
    uint32_t capacity;
    bool is_directory;
    uint32_t parent_dir;
    uint32_t num_children;
    uint32_t children[MAX_CHILDREN];
} ramfs_entry_t;

static ramfs_entry_t filesystem[MAX_FILES];
static uint32_t num_entries = 0;
static uint32_t current_dir = 0;
static bool fs_initialized = false;

uint32_t ramfs_find_path(const char* path);

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
    if (dir_id >= MAX_FILES || !filesystem[dir_id].is_directory || !name) {
        return RAMFS_NOT_FOUND;
    }

    ramfs_entry_t* dir = &filesystem[dir_id];
    
    for (uint32_t i = 0; i < dir->num_children; i++) {
        uint32_t child_id = dir->children[i];
        if (strcmp(filesystem[child_id].name, name) == 0) {
            return child_id;
        }
    }
    
    return RAMFS_NOT_FOUND;
}

static uint32_t ramfs_get_free_entry(void)
{
    for (uint32_t i = 0; i < MAX_FILES; i++) {
        if (filesystem[i].name[0] == '\0') {
            return i;
        }
    }
    return RAMFS_NOT_FOUND;
}

static bool ramfs_valid_name(const char* name)
{
    if (!name || strlen(name) == 0 || strlen(name) >= MAX_FILENAME) {
        return false;
    }

    if (strcmp(name, ".") == 0 || strcmp(name, "..") == 0) {
        return false;
    }

    return strchr(name, '/') == NULL;
}

static bool ramfs_add_child(uint32_t parent_dir, uint32_t child_id)
{
    if (parent_dir >= MAX_FILES || child_id >= MAX_FILES) {
        return false;
    }

    ramfs_entry_t* parent = &filesystem[parent_dir];
    if (!parent->is_directory || parent->num_children >= MAX_CHILDREN) {
        return false;
    }

    parent->children[parent->num_children++] = child_id;
    return true;
}

static bool ramfs_split_parent(const char* path, uint32_t* parent_out, char* name_out)
{
    if (!path || !parent_out || !name_out) {
        return false;
    }

    char path_copy[MAX_PATH];
    strncpy(path_copy, path, MAX_PATH - 1);
    path_copy[MAX_PATH - 1] = '\0';

    uint32_t len = strlen(path_copy);
    while (len > 1 && path_copy[len - 1] == '/') {
        path_copy[len - 1] = '\0';
        len--;
    }

    if (len == 0 || strcmp(path_copy, "/") == 0) {
        return false;
    }

    char* last_slash = strrchr(path_copy, '/');
    uint32_t parent = current_dir;
    const char* name = path_copy;

    if (last_slash) {
        name = last_slash + 1;
        if (last_slash == path_copy) {
            parent = 0;
        } else {
            *last_slash = '\0';
            parent = ramfs_find_path(path_copy);
        }
    }

    if (parent == RAMFS_NOT_FOUND || !filesystem[parent].is_directory || !ramfs_valid_name(name)) {
        return false;
    }

    *parent_out = parent;
    strncpy(name_out, name, MAX_FILENAME - 1);
    name_out[MAX_FILENAME - 1] = '\0';
    return true;
}

static uint32_t ramfs_create_entry(const char* path, bool is_directory)
{
    if (!fs_initialized) ramfs_init();

    uint32_t parent_dir;
    char name[MAX_FILENAME];

    if (!ramfs_split_parent(path, &parent_dir, name)) {
        return RAMFS_NOT_FOUND;
    }

    uint32_t existing = ramfs_find_entry_in_dir(parent_dir, name);
    if (existing != RAMFS_NOT_FOUND) {
        if (filesystem[existing].is_directory == is_directory) {
            return existing;
        }
        return RAMFS_NOT_FOUND;
    }

    uint32_t new_id = ramfs_get_free_entry();
    if (new_id == RAMFS_NOT_FOUND) {
        return RAMFS_NOT_FOUND;
    }

    ramfs_entry_t* entry = &filesystem[new_id];
    memset(entry, 0, sizeof(ramfs_entry_t));
    strncpy(entry->name, name, MAX_FILENAME - 1);
    entry->name[MAX_FILENAME - 1] = '\0';
    entry->data = NULL;
    entry->size = 0;
    entry->capacity = 0;
    entry->is_directory = is_directory;
    entry->parent_dir = parent_dir;
    entry->num_children = 0;

    if (!ramfs_add_child(parent_dir, new_id)) {
        memset(entry, 0, sizeof(ramfs_entry_t));
        return RAMFS_NOT_FOUND;
    }

    num_entries++;
    return new_id;
}

uint32_t ramfs_create_file(const char* path)
{
    return ramfs_create_entry(path, false);
}

uint32_t ramfs_create_directory(const char* path)
{
    return ramfs_create_entry(path, true);
}

uint32_t ramfs_find_path(const char* path)
{
    if (!fs_initialized) ramfs_init();

    if (!path || path[0] == '\0') {
        return current_dir;
    }
    
    char path_copy[MAX_PATH];
    strncpy(path_copy, path, MAX_PATH - 1);
    path_copy[MAX_PATH - 1] = '\0';
    
    uint32_t dir = current_dir;
    
    // Handle absolute path
    if (path_copy[0] == '/') {
        dir = 0;
        if (path_copy[1] == '\0') return 0; // Root directory
    }
    
    // Parse path components
    char* token = strtok(path_copy + (path_copy[0] == '/' ? 1 : 0), "/");
    
    while (token) {
        if (strcmp(token, ".") == 0) {
            token = strtok(NULL, "/");
            continue;
        }

        if (strcmp(token, "..") == 0) {
            dir = filesystem[dir].parent_dir;
            token = strtok(NULL, "/");
            continue;
        }

        uint32_t next = ramfs_find_entry_in_dir(dir, token);
        if (next == RAMFS_NOT_FOUND) return RAMFS_NOT_FOUND;
        dir = next;
        token = strtok(NULL, "/");
        if (token && !filesystem[dir].is_directory) {
            return RAMFS_NOT_FOUND;
        }
    }
    
    return dir;
}

bool ramfs_write_file(uint32_t file_id, const uint8_t* data, uint32_t size)
{
    if (file_id >= MAX_FILES || filesystem[file_id].name[0] == '\0' || filesystem[file_id].is_directory) return false;
    
    ramfs_entry_t* file = &filesystem[file_id];

    if (size == 0) {
        file->size = 0;
        return true;
    }

    if (!data) {
        return false;
    }
    
    // Allocate or reallocate buffer
    if (file->capacity < size) {
        uint8_t* new_data = (uint8_t*)kmalloc(size + 1);
        if (!new_data) {
            return false; // Allocation failed
        }

        if (file->data) {
            kfree(file->data);
        }

        file->data = new_data;
        file->capacity = size + 1;
    }
    
    memcpy(file->data, data, size);
    file->data[size] = '\0';
    file->size = size;
    return true;
}

bool ramfs_append_file(uint32_t file_id, const uint8_t* data, uint32_t size)
{
    if (file_id >= MAX_FILES || filesystem[file_id].name[0] == '\0' || filesystem[file_id].is_directory) return false;
    if (size == 0) return true;
    if (!data) return false;

    ramfs_entry_t* file = &filesystem[file_id];
    uint32_t new_size = file->size + size;

    if (file->capacity < new_size + 1) {
        uint8_t* new_data = (uint8_t*)kmalloc(new_size + 1);
        if (!new_data) {
            return false;
        }

        if (file->data && file->size > 0) {
            memcpy(new_data, file->data, file->size);
            kfree(file->data);
        }

        file->data = new_data;
        file->capacity = new_size + 1;
    }

    memcpy(file->data + file->size, data, size);
    file->size = new_size;
    file->data[file->size] = '\0';
    return true;
}

uint32_t ramfs_read_file(uint32_t file_id, uint8_t* buffer, uint32_t max_size)
{
    if (file_id >= MAX_FILES || !buffer || filesystem[file_id].name[0] == '\0' || filesystem[file_id].is_directory) return 0;
    
    ramfs_entry_t* file = &filesystem[file_id];
    uint32_t to_read = (max_size < file->size) ? max_size : file->size;
    
    if (file->data && to_read > 0) {
        memcpy(buffer, file->data, to_read);
    }
    
    return to_read;
}

bool ramfs_delete_entry(uint32_t entry_id)
{
    if (entry_id == 0 || entry_id >= MAX_FILES || filesystem[entry_id].name[0] == '\0') return false; // Can't delete root
    
    ramfs_entry_t* entry = &filesystem[entry_id];
    
    // Delete all children if it's a directory
    if (entry->is_directory) {
        while (entry->num_children > 0) {
            ramfs_delete_entry(entry->children[0]);
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
    if (dir_id >= MAX_FILES || filesystem[dir_id].name[0] == '\0' || !filesystem[dir_id].is_directory) return false;
    current_dir = dir_id;
    return true;
}

uint32_t ramfs_get_current_dir(void)
{
    return current_dir;
}

ramfs_entry_t* ramfs_get_entry(uint32_t entry_id)
{
    if (entry_id >= MAX_FILES || filesystem[entry_id].name[0] == '\0') return NULL;
    return &filesystem[entry_id];
}

uint32_t ramfs_list_directory(uint32_t dir_id, uint32_t* buffer, uint32_t max_count)
{
    if (dir_id >= MAX_FILES || !buffer || !filesystem[dir_id].is_directory) return 0;
    
    ramfs_entry_t* dir = &filesystem[dir_id];
    uint32_t count = (dir->num_children < max_count) ? dir->num_children : max_count;
    
    for (uint32_t i = 0; i < count; i++) {
        buffer[i] = dir->children[i];
    }
    
    return count;
}

void ramfs_get_full_path(uint32_t entry_id, char* path, uint32_t max_len)
{
    if (!path || max_len == 0) {
        return;
    }

    if (entry_id >= MAX_FILES || filesystem[entry_id].name[0] == '\0') {
        strncpy(path, "/", max_len - 1);
        path[max_len - 1] = '\0';
        return;
    }

    if (entry_id == 0) {
        strncpy(path, "/", max_len - 1);
        path[max_len - 1] = '\0';
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
    if (entry_id >= MAX_FILES || filesystem[entry_id].name[0] == '\0') return false;
    return filesystem[entry_id].is_directory;
}

uint32_t ramfs_entry_get_size(uint32_t entry_id)
{
    if (entry_id >= MAX_FILES || filesystem[entry_id].name[0] == '\0') return 0;
    return filesystem[entry_id].size;
}

uint32_t ramfs_entry_get_parent(uint32_t entry_id)
{
    if (entry_id >= MAX_FILES || filesystem[entry_id].name[0] == '\0') return RAMFS_NOT_FOUND;
    return filesystem[entry_id].parent_dir;
}

const char* ramfs_entry_get_name(uint32_t entry_id)
{
    if (entry_id >= MAX_FILES || filesystem[entry_id].name[0] == '\0') return NULL;
    return filesystem[entry_id].name;
}

uint8_t* ramfs_entry_get_data(uint32_t entry_id)
{
    if (entry_id >= MAX_FILES || filesystem[entry_id].name[0] == '\0') return NULL;
    return filesystem[entry_id].data;
}

bool ramfs_entry_set_name(uint32_t entry_id, const char* name)
{
    if (entry_id == 0 || entry_id >= MAX_FILES || filesystem[entry_id].name[0] == '\0' || !ramfs_valid_name(name)) return false;
    uint32_t parent = filesystem[entry_id].parent_dir;
    uint32_t existing = ramfs_find_entry_in_dir(parent, name);
    if (existing != RAMFS_NOT_FOUND && existing != entry_id) return false;
    strncpy(filesystem[entry_id].name, name, MAX_FILENAME - 1);
    filesystem[entry_id].name[MAX_FILENAME - 1] = '\0';
    return true;
}


# huggingOs Makefile

# Directories
KERNEL_DIR = kernel
BOOT_DIR = boot
BUILD_DIR = build
ISO_DIR = iso

# Tools
ASM = nasm
# Try to use cross-compiler, fallback to system gcc with -m32
ifneq ($(shell which i686-elf-gcc 2>/dev/null),)
	CC = i686-elf-gcc
	LD = i686-elf-ld
else
	CC = gcc
	LD = ld
	OBJCOPY = objcopy
endif
GRUB_MKRESCUE = grub-mkrescue

# Flags
ASMFLAGS = -f elf32
CFLAGS = -m32 -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -g
LDFLAGS = -m elf_i386 -T $(KERNEL_DIR)/linker.ld

# Source files
ASM_SOURCES = $(wildcard $(KERNEL_DIR)/*.asm)
C_SOURCES = $(wildcard $(KERNEL_DIR)/*.c) \
            $(wildcard $(KERNEL_DIR)/**/*.c) \
            $(wildcard $(KERNEL_DIR)/**/**/*.c)

# Object files
ASM_OBJECTS = $(ASM_SOURCES:$(KERNEL_DIR)/%.asm=$(BUILD_DIR)/%.o)
C_OBJECTS = $(C_SOURCES:$(KERNEL_DIR)/%.c=$(BUILD_DIR)/%.o)
OBJECTS = $(ASM_OBJECTS) $(C_OBJECTS)

# Kernel binary
KERNEL_BIN = $(BUILD_DIR)/kernel.bin

# ISO
ISO = huggingOs.iso

.PHONY: all clean iso run qemu help product-status product-doctor product-capabilities product-run-status product-agent-ai-status product-agent-ai-plan product-agent-ai-run product-agent-secrets product-agent-smoke product-smoke

all: $(KERNEL_BIN)

$(KERNEL_BIN): $(OBJECTS)
	@echo "Linking kernel..."
	@mkdir -p $(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ $^
	@echo "Kernel built: $@"

$(BUILD_DIR)/%.o: $(KERNEL_DIR)/%.asm
	@echo "Assembling $<..."
	@mkdir -p $(dir $@)
	$(ASM) $(ASMFLAGS) -o $@ $<

$(BUILD_DIR)/%.o: $(KERNEL_DIR)/%.c
	@echo "Compiling $<..."
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -I$(KERNEL_DIR) -c -o $@ $<

iso: $(ISO)

$(ISO): $(KERNEL_BIN)
	@echo "Creating ISO..."
	@mkdir -p $(ISO_DIR)/boot/grub
	@cp $(KERNEL_BIN) $(ISO_DIR)/boot/kernel.bin
	@cp $(BOOT_DIR)/grub/grub.cfg $(ISO_DIR)/boot/grub/
	$(GRUB_MKRESCUE) -o $(ISO) $(ISO_DIR)
	@echo "ISO created: $(ISO)"

clean:
	@echo "Cleaning..."
	@rm -rf $(BUILD_DIR)
	@rm -rf $(ISO_DIR)
	@rm -f $(ISO)
	@echo "Clean complete."

run: $(ISO)
	@echo "Starting VirtualBox..."
	@echo "Note: Make sure VirtualBox is installed and configured."
	@echo "You can run the ISO manually in VirtualBox or use: VBoxManage startvm huggingOs"

qemu: $(ISO)
	@echo "Starting QEMU..."
	qemu-system-i386 -cdrom $(ISO) -m 128M -vga std

product-status:
	python3 product/cli/huggingos.py status

product-doctor:
	python3 product/cli/huggingos.py doctor

product-capabilities:
	python3 product/cli/huggingos.py capabilities

product-run-status:
	python3 product/cli/huggingos.py run product.status

product-agent-ai-status:
	cd product/agent && cargo run -- ai status --json

product-agent-ai-plan:
	cd product/agent && cargo run -- ai plan "show product status" --json

product-agent-ai-run:
	cd product/agent && cargo run -- ai run "show product status" --json

product-agent-secrets:
	cd product/agent && cargo run -- secrets status --json

product-agent-smoke:
	cd product/agent && cargo test

product-smoke:
	python3 -m unittest discover -s product/tests -p "test_*.py"

# Help target
help:
	@echo "huggingOs Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all     - Build the kernel binary"
	@echo "  iso     - Build the kernel and create bootable ISO"
	@echo "  qemu    - Build the ISO and run it in QEMU"
	@echo "  product-status - Show Linux product status"
	@echo "  product-doctor - Run Linux product environment checks"
	@echo "  product-capabilities - List product capability APIs"
	@echo "  product-run-status - Run product.status through policy and audit"
	@echo "  product-agent-ai-status - Show Rust AI runtime/provider readiness"
	@echo "  product-agent-ai-plan - Build a deterministic local AI plan"
	@echo "  product-agent-ai-run - Execute a local AI plan through capabilities"
	@echo "  product-agent-secrets - Show redacted AI secret readiness"
	@echo "  product-agent-smoke - Run Rust product agent tests"
	@echo "  product-smoke  - Run Linux product smoke tests"
	@echo "  clean   - Remove build artifacts"
	@echo "  run     - Build ISO and provide instructions to run"
	@echo "  help    - Show this help message"
	@echo ""
	@echo "Requirements:"
	@echo "  - i686-elf-gcc cross-compiler or gcc with -m32 support"
	@echo "  - NASM assembler"
	@echo "  - grub-mkrescue and xorriso"
	@echo "  - QEMU or VirtualBox (for testing)"

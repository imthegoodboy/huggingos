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

.PHONY: all clean iso run qemu help product-status product-doctor product-capabilities product-run-status product-agent-ai-status product-agent-ai-plan product-agent-ai-run product-agent-secrets product-agent-desktop-status product-agent-apps-list product-agent-browser-dry-run product-agent-workspace-plan product-agent-screen-status product-agent-screen-capture-dry-run product-agent-context-snapshot product-agent-ocr-dry-run product-agent-memory-remember product-agent-memory-list product-agent-preference-set product-agent-semantic-index product-agent-semantic-search product-agent-resume-plan product-agent-agents-catalog product-agent-agents-plan product-agent-agents-orchestrate product-agent-agents-trace-list product-agent-smoke product-smoke

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

product-agent-desktop-status:
	cd product/agent && cargo run -- run desktop.status --json

product-agent-apps-list:
	cd product/agent && cargo run -- run apps.list --json

product-agent-browser-dry-run:
	cd product/agent && cargo run -- run browser.open_url --param url=https://example.com --dry-run --json

product-agent-workspace-plan:
	cd product/agent && cargo run -- run workspace.mode.plan --param mode=coding --json

product-agent-screen-status:
	cd product/agent && cargo run -- run screen.status --json

product-agent-screen-capture-dry-run:
	cd product/agent && cargo run -- run screen.capture --dry-run --json

product-agent-context-snapshot:
	cd product/agent && cargo run -- run context.snapshot --confirm --json

product-agent-ocr-dry-run:
	cd product/agent && cargo run -- run screen.ocr_image --param path=../../README.md --dry-run --json

product-agent-memory-remember:
	cd product/agent && cargo run -- run memory.session.remember --param key=current-goal --param value=phase-six-seven --json

product-agent-memory-list:
	cd product/agent && cargo run -- run memory.session.list --json

product-agent-preference-set:
	cd product/agent && cargo run -- run memory.preference.set --param key=theme --param value=dark --json

product-agent-semantic-index:
	cd product/agent && cargo run -- run files.semantic.index --param root=../../docs --confirm --json

product-agent-semantic-search:
	cd product/agent && cargo run -- run files.semantic.search --param query=capability --json

product-agent-resume-plan:
	cd product/agent && cargo run -- run workspace.resume.plan --json

product-agent-agents-catalog:
	cd product/agent && cargo run -- run agents.catalog --json

product-agent-agents-plan:
	cd product/agent && cargo run -- run agents.plan --param "goal=daily brief" --json

product-agent-agents-orchestrate:
	cd product/agent && cargo run -- run agents.orchestrate --param "goal=daily brief" --confirm --json

product-agent-agents-trace-list:
	cd product/agent && cargo run -- run agents.trace.list --json

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
	@echo "  product-agent-desktop-status - Show Linux desktop readiness"
	@echo "  product-agent-apps-list - List installed desktop applications"
	@echo "  product-agent-browser-dry-run - Dry-run browser URL opening"
	@echo "  product-agent-workspace-plan - Preview a workspace mode plan"
	@echo "  product-agent-screen-status - Show screen/context readiness"
	@echo "  product-agent-screen-capture-dry-run - Dry-run screenshot capture"
	@echo "  product-agent-context-snapshot - Capture active context metadata"
	@echo "  product-agent-ocr-dry-run - Dry-run OCR over a local file path"
	@echo "  product-agent-memory-remember - Store a short-term session memory fact"
	@echo "  product-agent-memory-list - List short-term session memory"
	@echo "  product-agent-preference-set - Store a local preference"
	@echo "  product-agent-semantic-index - Build an opt-in local file index"
	@echo "  product-agent-semantic-search - Search the local file index"
	@echo "  product-agent-resume-plan - Build a memory-backed resume plan"
	@echo "  product-agent-agents-catalog - List built-in agents"
	@echo "  product-agent-agents-plan - Preview a multi-agent plan"
	@echo "  product-agent-agents-orchestrate - Run a confirmed multi-agent plan"
	@echo "  product-agent-agents-trace-list - List multi-agent traces"
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

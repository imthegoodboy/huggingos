section .multiboot_header
align 4
header_start:
    dd 0x1BADB002                ; multiboot magic number
    dd 0x00000003                ; flags (align modules, provide memory map)
    dd -(0x1BADB002 + 0x00000003) ; checksum

section .text
global _start
extern kernel_main
extern kernel_stack_top

_start:
    cli                          ; disable interrupts
    
    ; Set up stack
    mov esp, kernel_stack_top
    
    ; Push multiboot info pointer (ebx) and magic (eax)
    push ebx
    push eax
    
    ; Clear direction flag
    cld
    
    ; Call kernel main
    call kernel_main
    
    ; If kernel_main returns, halt
.hang:
    cli
    hlt
    jmp .hang


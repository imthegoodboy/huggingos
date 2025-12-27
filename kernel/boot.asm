section .multiboot_header
align 4
header_start:
    dd 0x1BADB002                ; multiboot magic number
    dd 0x00000003                ; flags (align modules, provide memory map)
    dd -(0x1BADB002 + 0x00000003) ; checksum

section .text
global _start
extern kernel_main_multiboot

_start:
    cli                          ; disable interrupts
    
    ; Set up stack
    mov esp, stack_top
    
    ; Clear direction flag
    cld
    
    ; Push multiboot info pointer (ebx) and magic (eax)
    push ebx     ; multiboot info structure
    push eax     ; multiboot magic number
    
    ; Call kernel main
    call kernel_main_multiboot
    
    ; If kernel_main returns, halt
.hang:
    cli
    hlt
    jmp .hang

section .bss
align 16
stack_bottom:
    resb 16384                   ; 16KB stack
stack_top:


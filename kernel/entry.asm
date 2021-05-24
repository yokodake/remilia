[BITS 32]

%define BOOT_STACK_SIZE 4096

extern kernel_start
extern kernel_end

section .mboot
align 4
mboot:
    PAGE_ALIGN  equ (1 << 0)
    MEM_INFO    equ (1 << 0)
    MB_MAGIC    equ 0x1badb002
    MB_FLAGS    equ PAGE_ALIGN | MEM_INFO
    MB_CHECKSUM equ -(MB_MAGIC + MB_FLAGS)

    dd MB_MAGIC
    dd MB_FLAGS
    dd MB_CHECKSUM
    dd 0, 0, 0, 0, 0 ; address fields

align 4
GDT64:                      ; valid gdt is needed for long mode
    .Null: equ $ - GDT64    ; null descriptor
    dw 0                    ; Limit (low).
    dw 0                    ; Base (low).
    db 0                    ; Base (middle).
    db 0                    ; Access.
    db 0                    ; Granularity
    db 0                    ; Base (high)
    .Code: equ $ - GDT64    ; code descriptor
    dw 0                    ; Limit (low).
    dw 0                    ; Base (low).
    db 0                    ; Base (middle).
    db 10011010b            ; Access.
    db 00100000b            ; Granularity
    db 0                    ; Base (high)
    .Data: equ $ - GDT64    ; the data descriptor
    dw 0                    ; Limit (low).
    dw 0                    ; Base (low).
    db 0                    ; Base (middle).
    db 10011010b            ; Access.
    db 00000000b            ; Granularity
    db 0                    ; Base (high)
    .Pointer:               ; GDT-pointer
    dw $ - GDT64 - 1        ; Limit.
    dq GDT64                ; Base.

section .text
align 4
global _start
_start:
    cli ; avoid any interrupt
    ; init stack pointer
    mov esp, boot_stack
    add esp, BOOT_STACK_SIZE

    ; interpret multiboot information
    mov DWORD [mb_info], ebx


; this will set up the x86 control registers 
; caching + floating point enabled
; bootstrap page tables are loaded
; page size extension enabled
cpu_init:
    ; init page tables & map kernel 1:1
    push edi
    push ebx
    push ecx
    mov ecx, kernel_start
    mov ebx, kernel_end
    add ebx, 0x1000         ; size of bootloader?
L0: cmp ecx, ebx
    jae L1
    mov eax, ecx
    and eax, 0xfffff000     ; page align lower half
    mov edi, eax
    shr edi, 9              ; (edi >> 12) * 8 (index for boot_pgt)
    add edi, boot_pgt1      ; where does this come from
    or eax, 0x3             ; set present and writable bits
    mov DWORD [edi], eax    
    add ecx, 0x1000
    jmp L0
L1:
    pop ecx
    pop ebx
    pop edi
    
    ; check for long mode
    pushfd
    pop eax
    mov ecx, eax
    xor eax, 1 << 21
    push eax
    popfd
    pushfd
    pop eax
    push ecx
    popfd
    xor eax, ecx
    jz Linvalid

    ; cpuid > 0x80000000?
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb Linvalid ; It is less, there is no long mode.

    ; do we have a long mode?
    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29 ; Test if the LM-bit, which is bit 29, is set in the D-register.
    jz Linvalid ; They aren't, there is no long mode.

    ; Set CR3
    mov eax, boot_pml4
    ;or eax, (1 << 0)        ; set present bit
    mov cr3, eax

    ; enable PAE modus
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; switch to the compat mode
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; Set CR4
    mov eax, cr4
    and eax, 0xfffbf9ff     ; disable SSE
    ;or eax, (1 << 7)       ; enable PGE
    mov cr4, eax

    ; Set CR0 (PM-bit is already set)
    mov eax, cr0
    and eax, ~(1 << 2)      ; disable FPU emulation
    or eax, (1 << 1)        ; enable FPU monitoring
    and eax, ~(1 << 30)     ; enable caching
    and eax, ~(1 << 29)     ; disable write through caching
    and eax, ~(1 << 16)	    ; allow kernel write access to read-only pages
    or eax, (1 << 31)       ; enable paging
    mov cr0, eax

    lgdt [GDT64.Pointer] ; Load the 64-bit global descriptor table.
    jmp GDT64.Code:start64 ; Set the code segment and enter 64-bit long mode.

; there is no long mode
Linvalid:
    jmp $

[BITS 64]
start64:
    ; initialize segment registers
    mov ax, GDT64.Data
    mov ds, ax
    mov es, ax
    mov ss, ax
    xor ax, ax
    mov fs, ax
    mov gs, ax
    cld
    ; set default stack pointer
    mov rsp, boot_stack
    add rsp, BOOT_STACK_SIZE-16

    ; jump to the boot processors's C code
    extern rust_main
    jmp rust_main
    jmp $

section .data

global mb_info:
align 8
mb_info:
    dq 0

align 4096
global boot_stack
boot_stack:
    times (BOOT_STACK_SIZE) db 0xcd

align 4096
boot_pml4:
    dq boot_pdpt + 0x3  ; PG_PRESENT | PG_RW
    times 510 dq 0      ; PAGE_MAP_ENTRIES - 2
    dq boot_pml4 + 0x3  ; PG_PRESENT | PG_RW
boot_pdpt:
    dq boot_pgd + 0x3   ; PG_PRESENT | PG_RW
    times 511 dq 0      ; PAGE_MAP_ENTRIES - 1
boot_pgd:
    dq boot_pgt1 + 0x3  ; PG_PRESENT | PG_RW
    dq boot_pgt2 + 0x3  ; PG_PRESENT | PG_RW
    times 510 dq 0      ; PAGE_MAP_ENTRIES - 1
boot_pgt1:
    times 512 dq 0
boot_pgt2:
    times 512 dq 0

; add some hints to the ELF file
section .note.GNU-stack noalloc noexec nowrite progbits
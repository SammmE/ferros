.intel_syntax noprefix
.global syscall_dispatcher

syscall_dispatcher:
    swapgs
    // Save User Stack Pointer to GS offset 8
    mov qword ptr gs:[8], rsp
    
    // Load Kernel Stack Pointer from GS offset 0
    mov rsp, qword ptr gs:[0]

    // Save registers
    push r11  // RFLAGS
    push rcx  // RIP
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15

    // Handle arguments (R10 -> RCX)
    mov rcx, r10
    
    call syscall_rust_handler

    // Restore registers
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    pop rcx
    pop r11

    // Restore User Stack Pointer
    mov rsp, qword ptr gs:[8]
    
    swapgs
    sysretq

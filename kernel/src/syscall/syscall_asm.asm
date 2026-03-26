.global syscall_dispatcher

syscall_dispatcher:
    swapgs
    mov qword ptr gs:[8], rsp
    mov rsp, qword ptr gs:[0]

    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15
    
    push r11 # Save RFLAGS
    push rcx # Save RIP

    # Linux Syscall ABI -> System V ABI
    # RAX (ID) -> RDI (Arg1)
    # RDI (A1) -> RSI (Arg2)
    # RSI (A2) -> RDX (Arg3)
    # RDX (A3) -> RCX (Arg4)
    # R10 (A4) -> R8  (Arg5)
    # R8  (A5) -> R9  (Arg6)

    mov r9, r8
    mov r8, r10
    mov rcx, rdx
    mov rdx, rsi
    mov rsi, rdi
    mov rdi, rax
    
    call syscall_rust_handler

    pop rcx
    pop r11
    
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp

    mov rsp, qword ptr gs:[8]
    swapgs
    sysretq

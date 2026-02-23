use core::arch::global_asm;
use x86_64::VirtAddr;
use x86_64::registers::model_specific::{Efer, EferFlags, KernelGsBase, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};

use crate::gdt;

#[repr(C)]
pub struct KernelScratch {
    pub kernel_stack_top: u64,
    pub user_stack_scratch: u64,
}

const SYSCALL_STACK_SIZE: usize = 4096 * 4;
static mut SYSCALL_STACK: [u8; SYSCALL_STACK_SIZE] = [0; SYSCALL_STACK_SIZE];

static mut KERNEL_SCRATCH: KernelScratch = KernelScratch {
    kernel_stack_top: 0,
    user_stack_scratch: 0,
};

pub fn init_syscall() {
    unsafe {
        Efer::update(|flags| {
            flags.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
        });

        LStar::write(VirtAddr::new(syscall_dispatcher as *const () as u64));

        let code_selector = gdt::get_kernel_code_selector();
        let data_selector = gdt::get_kernel_data_selector();
        let (user_code_selector, user_data_selector) = gdt::get_user_selectors();

        Star::write(
            user_code_selector,
            user_data_selector,
            code_selector,
            data_selector,
        )
        .unwrap();

        SFMask::write(RFlags::INTERRUPT_FLAG | RFlags::TRAP_FLAG);

        let stack_top = VirtAddr::from_ptr(&raw const SYSCALL_STACK) + SYSCALL_STACK_SIZE as u64;
        KERNEL_SCRATCH.kernel_stack_top = stack_top.as_u64();

        let scratch_addr = VirtAddr::from_ptr(&raw const KERNEL_SCRATCH);
        KernelGsBase::write(scratch_addr);
    }
}

pub unsafe fn enter_userspace(entry_point: u64, stack_pointer: u64) -> ! {
    let (user_code_selector, user_data_selector) = crate::gdt::get_user_selectors();
    let rflags = (RFlags::INTERRUPT_FLAG | RFlags::from_bits_truncate(1 << 1)).bits();

    unsafe {
        core::arch::asm!(
            "push {ss:r}",
            "push {rsp}",
            "push {rflags}",
            "push {cs:r}",
            "push {rip}",
            "iretq",
            ss = in(reg) user_data_selector.0,
            rsp = in(reg) stack_pointer,
            rflags = in(reg) rflags,
            cs = in(reg) user_code_selector.0,
            rip = in(reg) entry_point,
            options(noreturn)
        );
    }
}

#[unsafe(no_mangle)]
extern "C" fn syscall_rust_handler(
    syscall_id: usize,
    arg1: usize,
    arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> usize {
    crate::serial_println!(
        "SYSCALL: ID={}, arg1={:#x}, arg2={:#x}",
        syscall_id,
        arg1,
        arg2
    );

    match syscall_id {
        1 => syscall_print(arg1, arg2),
        _ => {
            crate::serial_println!("Unknown syscall: {}", syscall_id);
            usize::MAX
        }
    }
}

fn syscall_print(msg_ptr: usize, len: usize) -> usize {
    if msg_ptr == 0 || msg_ptr > 0x800000000000 {
        return 1;
    }

    let msg_slice = unsafe { core::slice::from_raw_parts(msg_ptr as *const u8, len) };
    if let Ok(msg) = core::str::from_utf8(msg_slice) {
        crate::println!("{}", msg);
        0
    } else {
        1
    }
}

pub fn test_userspace_syscall() {
    crate::println!("Preparing to enter userspace...");

    let user_code_addr = VirtAddr::new(0x400000);
    let user_stack_addr = VirtAddr::new(0x800000);

    let msg = b"Hello from Ring 3 (Standard ABI)!\n";
    let code_size = 64;
    let string_addr = user_code_addr.as_u64() + code_size as u64;

    let mut code = alloc::vec![0u8; 4096];
    let mut writer = 0;

    let mut emit = |bytes: &[u8], offset: &mut usize| {
        for &b in bytes {
            code[*offset] = b;
            *offset += 1;
        }
    };

    // 1. mov rdi, string_addr (Arg 1)
    emit(&[0x48, 0xBF], &mut writer);
    emit(&string_addr.to_le_bytes(), &mut writer);

    // 2. mov rsi, msg_len (Arg 2)
    emit(&[0x48, 0xBE], &mut writer);
    emit(&(msg.len() as u64).to_le_bytes(), &mut writer);

    // 3. mov rax, 1 (Syscall ID)
    emit(&[0x48, 0xC7, 0xc0, 0x01, 0x00, 0x00, 0x00], &mut writer);

    // 4. syscall
    emit(&[0x0F, 0x05], &mut writer);

    // 5. jmp $
    emit(&[0xEB, 0xFE], &mut writer);

    while writer < code_size {
        emit(&[0x90], &mut writer);
    }

    emit(msg, &mut writer);

    let mut mapper = crate::memory::get_mapper().expect("Memory system not initialized");
    let mut frame_allocator = crate::memory::FRAME_ALLOCATOR.lock();
    let frame_allocator = frame_allocator
        .as_mut()
        .expect("Frame allocator not initialized");

    unsafe {
        let frame = frame_allocator.allocate_frame().expect("No frames left");
        let page = Page::<Size4KiB>::containing_address(user_code_addr);
        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        mapper
            .map_to(page, frame, flags, frame_allocator)
            .unwrap()
            .flush();

        let dest_ptr = user_code_addr.as_mut_ptr::<u8>();
        core::ptr::copy_nonoverlapping(code.as_ptr(), dest_ptr, code.len());

        let stack_frame = frame_allocator.allocate_frame().expect("No frames left");
        let stack_page = Page::<Size4KiB>::containing_address(user_stack_addr - 1u64);
        mapper
            .map_to(stack_page, stack_frame, flags, frame_allocator)
            .unwrap()
            .flush();
    }

    crate::println!("Jumping to Ring 3...");
    unsafe {
        crate::syscall::enter_userspace(user_code_addr.as_u64(), user_stack_addr.as_u64());
    }
}

global_asm!(include_str!("syscall_asm.asm"));

unsafe extern "C" {
    fn syscall_dispatcher();
}

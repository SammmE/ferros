use x86_64::VirtAddr;
use x86_64::registers::model_specific::{Efer, EferFlags, KernelGsBase, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;

use core::arch::global_asm;

use crate::gdt;

#[repr(C)]
pub struct KernelScratch {
    pub kernel_stack_top: u64,   // Offset 0
    pub user_stack_scratch: u64, // Offset 8
}

// 16KB system call stack
const SYSCALL_STACK_SIZE: usize = 4096 * 4;
static mut SYSCALL_STACK: [u8; SYSCALL_STACK_SIZE] = [0; SYSCALL_STACK_SIZE];

// The instance that GS will point to
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

        // STAR register takes (User CS, User SS, Kernel CS, Kernel SS)
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
            "swapgs",
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
    rdi: usize,
    rsi: usize,
    rdx: usize,
    _r10: usize,
    _r8: usize,
    _r9: usize,
) -> usize {
    crate::serial_println!("SYSCALL CAUGHT! Args: {}, {}, {}", rdi, rsi, rdx);
    0
}

global_asm!(include_str!("syscall_asm.asm"));

unsafe extern "C" {
    fn syscall_dispatcher();
}

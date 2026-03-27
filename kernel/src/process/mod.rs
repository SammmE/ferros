pub mod elf;
pub mod scheduler;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::arch::global_asm;
use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use x86_64::{PhysAddr, VirtAddr};

global_asm!(include_str!("switch.asm"));

pub enum State {
    Ready,
    Running,
    Blocked,
    Dead,
}

pub trait KernelObject: Send + Sync {
    fn read(&self, buf: &mut [u8]) -> usize {
        0
    }
    fn write(&self, buf: &[u8]) -> usize {
        0
    }
}

pub struct Process {
    pub id: ProcessId,
    pub state: State,
    pub saved_rsp: u64,
    pub page_table: PhysAddr,
    pub kstack_top: VirtAddr,
    pub fd_table: Mutex<BTreeMap<usize, Arc<dyn KernelObject>>>,
}

impl Process {
    pub fn new(
        entry_point: u64,
        user_stack_top: u64,
        page_table_phys: PhysAddr,
        kstack_top: VirtAddr,
        inherited_fds: BTreeMap<usize, Arc<dyn KernelObject>>,
    ) -> Process {
        let (user_code, user_data) = crate::gdt::get_user_selectors();

        let mut kstack_ptr = kstack_top.as_u64();

        let mut push = |val: u64| {
            kstack_ptr -= 8; // Stacks grow downwards
            unsafe {
                *(kstack_ptr as *mut u64) = val;
            }
        };

        push(user_data.0 as u64); // SS
        push(user_stack_top); // RSP (User Stack)
        push(0x202); // RFLAGS (Interrupts enabled)
        push(user_code.0 as u64); // CS
        push(entry_point); // RIP

        for _ in 0..15 {
            push(0); // Initialize rax through r15 to 0
        }

        Process {
            id: ProcessId::new(),
            state: State::Ready,
            saved_rsp: kstack_ptr,
            page_table: page_table_phys,
            kstack_top,
            fd_table: Mutex::new(inherited_fds),
        }
    }
}

impl fmt::Debug for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Process").field("id", &self.id).finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(u64);

impl ProcessId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ProcessId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

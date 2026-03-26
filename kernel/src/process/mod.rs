pub mod elf;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use x86_64::{PhysAddr, VirtAddr};

pub struct Regs {
    // GPRs
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rbp: u64,
    rsp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,

    // CSRs
    rip: u64,
    rflags: u64,
    cs: u64,
    ss: u64,
    cr3: u64,
}

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
    pub regs: Regs,
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
    ) -> Process {
        let (user_code, user_data) = crate::gdt::get_user_selectors();
        Process {
            id: ProcessId::new(),
            state: State::Ready,
            regs: Regs {
                rax: 0,
                rbx: 0,
                rcx: 0,
                rdx: 0,
                rsi: 0,
                rdi: 0,
                rbp: 0,
                r8: 0,
                r9: 0,
                r10: 0,
                r11: 0,
                r12: 0,
                r13: 0,
                r14: 0,
                r15: 0,

                rip: entry_point,
                rsp: user_stack_top,
                cr3: page_table_phys.as_u64(),

                cs: user_code.0 as u64,
                ss: user_data.0 as u64,
                rflags: 0x202,
            },
            page_table: page_table_phys,
            kstack_top,
            fd_table: Mutex::new(BTreeMap::new()),
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

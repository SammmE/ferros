use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use spin::Mutex;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB,
    },
    PhysAddr, VirtAddr,
};

static PHYS_MEM_OFFSET: Mutex<Option<VirtAddr>> = Mutex::new(None);
pub static FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    *PHYS_MEM_OFFSET.lock() = Some(physical_memory_offset);
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

pub fn get_mapper() -> Option<OffsetPageTable<'static>> {
    let offset = (*PHYS_MEM_OFFSET.lock())?;
    unsafe {
        let level_4_table = active_level_4_table(offset);
        Some(OffsetPageTable::new(level_4_table, offset))
    }
}

/// Unmaps the null page (address 0x0) to prevent null pointer dereferences.
/// This should be called after initialization to ensure null pointers cause page faults.
pub fn unmap_null_page() -> Result<(), &'static str> {
    let mut mapper = get_mapper().ok_or("Memory system not initialized")?;

    // Unmap the first page (0x0 to 0x1000)
    let null_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(0));

    match mapper.unmap(null_page) {
        Ok((_, flush)) => {
            flush.flush();
            Ok(())
        }
        Err(_) => {
            // Page might not be mapped, which is fine
            Ok(())
        }
    }
}

/// Allocates a kernel stack with a guard page at the bottom.
/// The guard page is unmapped, so stack overflow will trigger a page fault.
///
/// Returns the top of the stack (where the stack pointer should start).
pub fn allocate_kernel_stack_with_guard(size_in_pages: usize) -> Result<VirtAddr, &'static str> {
    if size_in_pages == 0 {
        return Err("Stack size must be at least 1 page");
    }

    let mut mapper = get_mapper().ok_or("Memory system not initialized")?;
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let frame_allocator = frame_allocator
        .as_mut()
        .ok_or("Frame allocator not initialized")?;

    // We need to find a free virtual address range
    // For simplicity, we'll use a static counter to allocate stacks in a dedicated region
    // Kernel stacks will be placed starting at 0xFFFF_F000_0000_0000
    static NEXT_STACK_ADDR: Mutex<u64> = Mutex::new(0xFFFF_F000_0000_0000);

    let stack_start = {
        let mut addr = NEXT_STACK_ADDR.lock();
        let start = *addr;
        // Reserve space for guard page + stack pages
        *addr += ((size_in_pages + 1) * 4096) as u64;
        start
    };

    // Map the guard page as NOT PRESENT (don't allocate a frame for it)
    let _guard_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(stack_start));

    // Note: We intentionally do NOT map the guard page, leaving it unmapped
    // This way, if the stack overflows, it will hit this page and cause a page fault

    // Map the actual stack pages (after the guard page)
    let stack_base = stack_start + 4096;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    for i in 0..size_in_pages {
        let page_addr = stack_base + (i as u64 * 4096);
        let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(page_addr));

        let frame = frame_allocator.allocate_frame().ok_or("Out of memory")?;

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .map_err(|_| "Failed to map stack page")?
                .flush();
        }
    }

    // Return the top of the stack (stack grows downward)
    let stack_top = stack_base + (size_in_pages as u64 * 4096);
    Ok(VirtAddr::new(stack_top))
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { &mut *page_table_ptr }
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// Translates a virtual address to a physical address by walking the page tables.
/// Returns None if the address is not mapped or not accessible.
pub fn translate_addr(addr: VirtAddr) -> Option<PhysAddr> {
    let offset = (*PHYS_MEM_OFFSET.lock())?;

    unsafe {
        use x86_64::registers::control::Cr3;

        let (level_4_table_frame, _) = Cr3::read();
        let phys = level_4_table_frame.start_address();
        let virt = offset + phys.as_u64();
        let level_4_table: &PageTable = &*(virt.as_ptr());

        // Extract page table indices from virtual address
        let l4_index = (addr.as_u64() >> 39) & 0x1FF;
        let l3_index = (addr.as_u64() >> 30) & 0x1FF;
        let l2_index = (addr.as_u64() >> 21) & 0x1FF;
        let l1_index = (addr.as_u64() >> 12) & 0x1FF;
        let page_offset = addr.as_u64() & 0xFFF;

        // Walk Level 4
        let l4_entry = &level_4_table[l4_index as usize];
        if !l4_entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        // Walk Level 3
        let l3_table_phys = l4_entry.addr();
        let l3_table_virt = offset + l3_table_phys.as_u64();
        let l3_table: &PageTable = &*(l3_table_virt.as_ptr());
        let l3_entry = &l3_table[l3_index as usize];
        if !l3_entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        // Walk Level 2
        let l2_table_phys = l3_entry.addr();
        let l2_table_virt = offset + l2_table_phys.as_u64();
        let l2_table: &PageTable = &*(l2_table_virt.as_ptr());
        let l2_entry = &l2_table[l2_index as usize];
        if !l2_entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        // Walk Level 1
        let l1_table_phys = l2_entry.addr();
        let l1_table_virt = offset + l1_table_phys.as_u64();
        let l1_table: &PageTable = &*(l1_table_virt.as_ptr());
        let l1_entry = &l1_table[l1_index as usize];
        if !l1_entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        // Calculate final physical address
        let frame_phys = l1_entry.addr();
        Some(frame_phys + page_offset)
    }
}

/// Checks if a user buffer is readable (present and user-accessible).
/// This function validates that all pages in the range [addr, addr+len) are:
/// 1. Present in memory
/// 2. User-accessible (not kernel-only)
/// 3. In the lower half of the address space (user space)
pub fn is_user_readable(addr: VirtAddr, len: usize) -> bool {
    // Check if address is in user space (lower half)
    if addr.as_u64() >= 0x0000_8000_0000_0000 {
        return false;
    }

    // Check for overflow
    let end_addr = match addr.as_u64().checked_add(len as u64) {
        Some(end) => end,
        None => return false,
    };

    // Ensure the entire range is in user space
    if end_addr >= 0x0000_8000_0000_0000 {
        return false;
    }

    // Check each page in the range
    let start_page = addr.as_u64() & !0xFFF;
    let end_page = (end_addr + 0xFFF) & !0xFFF;

    for page_addr in (start_page..end_page).step_by(4096) {
        let page_virt = VirtAddr::new(page_addr);

        // Check if page is mapped
        if translate_addr(page_virt).is_none() {
            return false;
        }

        // Check if page is user-accessible by walking page tables
        if !is_page_user_accessible(page_virt) {
            return false;
        }
    }

    true
}

/// Checks if a user buffer is writable (present, user-accessible, and writable).
pub fn is_user_writable(addr: VirtAddr, len: usize) -> bool {
    // Check if address is in user space (lower half)
    if addr.as_u64() >= 0x0000_8000_0000_0000 {
        return false;
    }

    // Check for overflow
    let end_addr = match addr.as_u64().checked_add(len as u64) {
        Some(end) => end,
        None => return false,
    };

    // Ensure the entire range is in user space
    if end_addr >= 0x0000_8000_0000_0000 {
        return false;
    }

    // Check each page in the range
    let start_page = addr.as_u64() & !0xFFF;
    let end_page = (end_addr + 0xFFF) & !0xFFF;

    for page_addr in (start_page..end_page).step_by(4096) {
        let page_virt = VirtAddr::new(page_addr);

        // Check if page is mapped
        if translate_addr(page_virt).is_none() {
            return false;
        }

        // Check if page is user-accessible and writable
        if !is_page_user_writable(page_virt) {
            return false;
        }
    }

    true
}

/// Helper function to walk page tables and check flags at each level.
/// Returns the flags of the final page table entry, or None if any level is not present.
fn walk_page_tables_for_flags(addr: VirtAddr) -> Option<PageTableFlags> {
    let offset = match *PHYS_MEM_OFFSET.lock() {
        Some(o) => o,
        None => return None,
    };

    unsafe {
        use x86_64::registers::control::Cr3;

        let (level_4_table_frame, _) = Cr3::read();
        let phys = level_4_table_frame.start_address();
        let virt = offset + phys.as_u64();
        let level_4_table: &PageTable = &*(virt.as_ptr());

        let l4_index = (addr.as_u64() >> 39) & 0x1FF;
        let l3_index = (addr.as_u64() >> 30) & 0x1FF;
        let l2_index = (addr.as_u64() >> 21) & 0x1FF;
        let l1_index = (addr.as_u64() >> 12) & 0x1FF;

        let l4_entry = &level_4_table[l4_index as usize];
        if !l4_entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        let l3_table_phys = l4_entry.addr();
        let l3_table_virt = offset + l3_table_phys.as_u64();
        let l3_table: &PageTable = &*(l3_table_virt.as_ptr());
        let l3_entry = &l3_table[l3_index as usize];
        if !l3_entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        let l2_table_phys = l3_entry.addr();
        let l2_table_virt = offset + l2_table_phys.as_u64();
        let l2_table: &PageTable = &*(l2_table_virt.as_ptr());
        let l2_entry = &l2_table[l2_index as usize];
        if !l2_entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        let l1_table_phys = l2_entry.addr();
        let l1_table_virt = offset + l1_table_phys.as_u64();
        let l1_table: &PageTable = &*(l1_table_virt.as_ptr());
        let l1_entry = &l1_table[l1_index as usize];

        // Check that all levels have USER_ACCESSIBLE flag
        if !l4_entry.flags().contains(PageTableFlags::USER_ACCESSIBLE)
            || !l3_entry.flags().contains(PageTableFlags::USER_ACCESSIBLE)
            || !l2_entry.flags().contains(PageTableFlags::USER_ACCESSIBLE)
            || !l1_entry.flags().contains(PageTableFlags::USER_ACCESSIBLE)
        {
            return None;
        }

        Some(l1_entry.flags())
    }
}

/// Helper to check if a specific page has USER_ACCESSIBLE flag
fn is_page_user_accessible(addr: VirtAddr) -> bool {
    walk_page_tables_for_flags(addr).is_some()
}

/// Helper to check if a specific page has USER_ACCESSIBLE and WRITABLE flags
fn is_page_user_writable(addr: VirtAddr) -> bool {
    walk_page_tables_for_flags(addr)
        .map(|flags| flags.contains(PageTableFlags::WRITABLE))
        .unwrap_or(false)
}

pub mod pmm;
pub mod vmm;

use bootloader_api::info::MemoryRegions;
use x86_64::VirtAddr;

pub use vmm::{
    get_mapper, is_user_readable, is_user_writable, translate as translate_addr,
    map_page, unmap_page, set_page_flags, create_address_space, switch_address_space,
    map_page_in,
};
pub use pmm::PMM;

pub fn init(phys_offset: VirtAddr, regions: &'static MemoryRegions) {
    unsafe {
        vmm::init(phys_offset);

        let allocator = pmm::BitmapAllocator::init(regions, phys_offset.as_u64());
        let (used, total) = allocator.stats();
        *pmm::PMM.lock() = Some(allocator);

        crate::serial_println!(
            "[PMM] Initialized: {}/{} frames used ({} MB free)",
            used,
            total,
            (total - used) * 4096 / (1024 * 1024)
        );
    }
}

pub fn unmap_null_page() -> Result<(), &'static str> {
    match unmap_page(VirtAddr::new(0)) {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}

pub fn allocate_kernel_stack_with_guard(size_in_pages: usize) -> Result<VirtAddr, &'static str> {
    use spin::Mutex;
    use x86_64::structures::paging::PageTableFlags;

    if size_in_pages == 0 {
        return Err("Stack size must be at least 1 page");
    }

    static NEXT_STACK_ADDR: Mutex<u64> = Mutex::new(0xFFFF_F000_0000_0000);

    let stack_start = {
        let mut addr = NEXT_STACK_ADDR.lock();
        let start = *addr;
        *addr += ((size_in_pages + 1) * 4096) as u64;
        start
    };

    let stack_base = stack_start + 4096;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    for i in 0..size_in_pages {
        let page_addr = stack_base + (i as u64 * 4096);
        let frame_addr = {
            let mut pmm_lock = PMM.lock();
            let pmm = pmm_lock.as_mut().ok_or("PMM not initialized")?;
            pmm.alloc_frame().ok_or("Out of memory for stack")?
        };
        map_page(VirtAddr::new(page_addr), frame_addr, flags)?;
    }

    let stack_top = stack_base + (size_in_pages as u64 * 4096);
    Ok(VirtAddr::new(stack_top))
}

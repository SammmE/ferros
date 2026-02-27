use spin::Mutex;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB,
    },
    PhysAddr, VirtAddr,
};

use crate::memory::pmm::PMM;

static PHYS_OFFSET: Mutex<Option<VirtAddr>> = Mutex::new(None);

pub unsafe fn init(phys_offset: VirtAddr) {
    *PHYS_OFFSET.lock() = Some(phys_offset);
}

pub fn phys_offset() -> VirtAddr {
    PHYS_OFFSET.lock().expect("VMM not initialized")
}

pub fn get_mapper() -> Option<OffsetPageTable<'static>> {
    let offset = (*PHYS_OFFSET.lock())?;
    unsafe {
        let l4 = active_l4_table(offset);
        Some(OffsetPageTable::new(l4, offset))
    }
}

unsafe fn active_l4_table(offset: VirtAddr) -> &'static mut PageTable {
    let (frame, _) = Cr3::read();
    let virt = offset + frame.start_address().as_u64();
    unsafe { &mut *(virt.as_mut_ptr()) }
}

pub fn map_page(virt: VirtAddr, phys: PhysAddr, flags: PageTableFlags) -> Result<(), &'static str> {
    let mut mapper = get_mapper().ok_or("VMM not initialized")?;
    let page: Page<Size4KiB> = Page::containing_address(virt);
    let frame = PhysFrame::containing_address(phys);
    let mut pmm = PMM.lock();
    let pmm = pmm.as_mut().ok_or("PMM not initialized")?;

    unsafe {
        mapper
            .map_to(page, frame, flags, pmm)
            .map_err(|_| "map_page failed")?
            .flush();
    }
    Ok(())
}

pub fn unmap_page(virt: VirtAddr) -> Result<PhysAddr, &'static str> {
    let mut mapper = get_mapper().ok_or("VMM not initialized")?;
    let page: Page<Size4KiB> = Page::containing_address(virt);

    let (frame, flush) = mapper.unmap(page).map_err(|_| "unmap_page failed")?;
    flush.flush();
    Ok(frame.start_address())
}

pub fn set_page_flags(virt: VirtAddr, flags: PageTableFlags) -> Result<(), &'static str> {
    let mut mapper = get_mapper().ok_or("VMM not initialized")?;
    let page: Page<Size4KiB> = Page::containing_address(virt);

    unsafe {
        mapper
            .update_flags(page, flags)
            .map_err(|_| "set_page_flags failed")?
            .flush();
    }
    Ok(())
}

pub fn translate(virt: VirtAddr) -> Option<PhysAddr> {
    let offset = (*PHYS_OFFSET.lock())?;

    unsafe {
        let (l4_frame, _) = Cr3::read();
        let l4_virt = offset + l4_frame.start_address().as_u64();
        let l4: &PageTable = &*(l4_virt.as_ptr());

        let indices = [
            ((virt.as_u64() >> 39) & 0x1FF) as usize,
            ((virt.as_u64() >> 30) & 0x1FF) as usize,
            ((virt.as_u64() >> 21) & 0x1FF) as usize,
            ((virt.as_u64() >> 12) & 0x1FF) as usize,
        ];
        let page_offset = virt.as_u64() & 0xFFF;

        let l4e = &l4[indices[0]];
        if !l4e.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        let l3: &PageTable = &*((offset + l4e.addr().as_u64()).as_ptr());
        let l3e = &l3[indices[1]];
        if !l3e.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        let l2: &PageTable = &*((offset + l3e.addr().as_u64()).as_ptr());
        let l2e = &l2[indices[2]];
        if !l2e.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        let l1: &PageTable = &*((offset + l2e.addr().as_u64()).as_ptr());
        let l1e = &l1[indices[3]];
        if !l1e.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        Some(l1e.addr() + page_offset)
    }
}

fn walk_flags(addr: VirtAddr) -> Option<PageTableFlags> {
    let offset = (*PHYS_OFFSET.lock())?;

    unsafe {
        let (l4_frame, _) = Cr3::read();
        let l4: &PageTable = &*((offset + l4_frame.start_address().as_u64()).as_ptr());

        let indices = [
            ((addr.as_u64() >> 39) & 0x1FF) as usize,
            ((addr.as_u64() >> 30) & 0x1FF) as usize,
            ((addr.as_u64() >> 21) & 0x1FF) as usize,
            ((addr.as_u64() >> 12) & 0x1FF) as usize,
        ];

        let l4e = &l4[indices[0]];
        if !l4e.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }
        let l3: &PageTable = &*((offset + l4e.addr().as_u64()).as_ptr());
        let l3e = &l3[indices[1]];
        if !l3e.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }
        let l2: &PageTable = &*((offset + l3e.addr().as_u64()).as_ptr());
        let l2e = &l2[indices[2]];
        if !l2e.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }
        let l1: &PageTable = &*((offset + l2e.addr().as_u64()).as_ptr());
        let l1e = &l1[indices[3]];

        if !l4e.flags().contains(PageTableFlags::USER_ACCESSIBLE)
            || !l3e.flags().contains(PageTableFlags::USER_ACCESSIBLE)
            || !l2e.flags().contains(PageTableFlags::USER_ACCESSIBLE)
            || !l1e.flags().contains(PageTableFlags::USER_ACCESSIBLE)
        {
            return None;
        }

        Some(l1e.flags())
    }
}

pub fn is_user_readable(addr: VirtAddr, len: usize) -> bool {
    if addr.as_u64() >= 0x0000_8000_0000_0000 {
        return false;
    }
    let end = match addr.as_u64().checked_add(len as u64) {
        Some(e) => e,
        None => return false,
    };
    if end >= 0x0000_8000_0000_0000 {
        return false;
    }

    let start_page = addr.as_u64() & !0xFFF;
    let end_page = (end + 0xFFF) & !0xFFF;

    for pa in (start_page..end_page).step_by(4096) {
        if walk_flags(VirtAddr::new(pa)).is_none() {
            return false;
        }
    }
    true
}

pub fn is_user_writable(addr: VirtAddr, len: usize) -> bool {
    if addr.as_u64() >= 0x0000_8000_0000_0000 {
        return false;
    }
    let end = match addr.as_u64().checked_add(len as u64) {
        Some(e) => e,
        None => return false,
    };
    if end >= 0x0000_8000_0000_0000 {
        return false;
    }

    let start_page = addr.as_u64() & !0xFFF;
    let end_page = (end + 0xFFF) & !0xFFF;

    for pa in (start_page..end_page).step_by(4096) {
        match walk_flags(VirtAddr::new(pa)) {
            Some(f) if f.contains(PageTableFlags::WRITABLE) => {}
            _ => return false,
        }
    }
    true
}

pub fn create_address_space() -> Result<PhysAddr, &'static str> {
    let offset = phys_offset();

    let frame = {
        let mut pmm = PMM.lock();
        let pmm = pmm.as_mut().ok_or("PMM not initialized")?;
        pmm.alloc_frame().ok_or("Out of frames for PML4")?
    };

    unsafe {
        let new_table: &mut PageTable =
            &mut *((offset + frame.as_u64()).as_mut_ptr());
        new_table.zero();

        let current_l4 = active_l4_table(offset);
        for i in 256..512 {
            new_table[i] = current_l4[i].clone();
        }
    }

    Ok(frame)
}

pub fn switch_address_space(pml4_phys: PhysAddr) {
    use x86_64::registers::control::Cr3Flags;
    let frame = PhysFrame::containing_address(pml4_phys);
    unsafe {
        Cr3::write(frame, Cr3Flags::empty());
    }
}

pub fn map_page_in(
    pml4_phys: PhysAddr,
    virt: VirtAddr,
    phys: PhysAddr,
    flags: PageTableFlags,
) -> Result<(), &'static str> {
    let offset = phys_offset();

    unsafe {
        let l4: &mut PageTable = &mut *((offset + pml4_phys.as_u64()).as_mut_ptr());

        let indices = [
            ((virt.as_u64() >> 39) & 0x1FF) as usize,
            ((virt.as_u64() >> 30) & 0x1FF) as usize,
            ((virt.as_u64() >> 21) & 0x1FF) as usize,
            ((virt.as_u64() >> 12) & 0x1FF) as usize,
        ];

        let l3 = ensure_table(l4, indices[0], offset)?;
        let l2 = ensure_table(l3, indices[1], offset)?;
        let l1 = ensure_table(l2, indices[2], offset)?;

        l1[indices[3]].set_addr(phys, flags);
    }
    Ok(())
}

unsafe fn ensure_table(
    parent: &mut PageTable,
    index: usize,
    offset: VirtAddr,
) -> Result<&'static mut PageTable, &'static str> {
    let entry = &mut parent[index];

    if !entry.flags().contains(PageTableFlags::PRESENT) {
        let frame_addr = {
            let mut pmm = PMM.lock();
            let pmm = pmm.as_mut().ok_or("PMM not initialized")?;
            pmm.alloc_frame().ok_or("Out of frames for page table")?
        };

        let table: &mut PageTable =
            unsafe { &mut *((offset + frame_addr.as_u64()).as_mut_ptr()) };
        table.zero();

        entry.set_addr(
            frame_addr,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
        );
    }

    let table_phys = entry.addr();
    unsafe { Ok(&mut *((offset + table_phys.as_u64()).as_mut_ptr())) }
}

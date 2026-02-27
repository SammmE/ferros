use alloc::string::String;
use alloc::vec::Vec;
use x86_64::{
    VirtAddr,
    structures::paging::{Page, PageTableFlags, Size4KiB},
};
use xmas_elf::ElfFile;
use xmas_elf::program::Type;

use crate::fs::FILESYSTEM;
use crate::memory;
use crate::memory::pmm::PMM;

pub fn load_elf(filename: &str) -> Result<(), String> {
    let file_data: Vec<u8> = {
        let mut fs_lock = FILESYSTEM.lock();
        let fs = fs_lock.as_mut().ok_or("Filesystem not initialized")?;
        fs.read_file(filename).ok_or("File not found")?
    };

    let elf = ElfFile::new(&file_data).map_err(|_| "Elf parse error")?;
    xmas_elf::header::sanity_check(&elf).map_err(|_| "ELF sanity check failed")?;

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::USER_ACCESSIBLE;

    for ph in elf.program_iter() {
        if ph.get_type().map_err(|_| "Invalid Segment Type")? == Type::Load {
            let virt_addr = ph.virtual_addr();
            let file_size = ph.file_size();
            let mem_size = ph.mem_size();
            let file_offset = ph.offset();

            if virt_addr == 0 {
                continue;
            }

            let start_addr = VirtAddr::new(virt_addr);
            let start_page: Page<Size4KiB> = Page::containing_address(start_addr);
            let end_addr = start_addr + mem_size;
            let end_page: Page<Size4KiB> = Page::containing_address(end_addr - 1u64);

            for page in Page::range_inclusive(start_page, end_page) {
                if memory::translate_addr(page.start_address()).is_none() {
                    let frame_addr = {
                        let mut pmm = PMM.lock();
                        let pmm = pmm.as_mut().ok_or("PMM not initialized")?;
                        pmm.alloc_frame().ok_or("Out of memory")?
                    };
                    memory::map_page(page.start_address(), frame_addr, flags)
                        .map_err(|e| String::from(e))?;
                }
            }

            unsafe {
                let src_ptr = file_data.as_ptr().add(file_offset as usize);
                let dest_ptr = virt_addr as *mut u8;
                core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, file_size as usize);
            }

            if mem_size > file_size {
                unsafe {
                    let zero_start = (virt_addr + file_size) as *mut u8;
                    let zero_len = (mem_size - file_size) as usize;
                    core::ptr::write_bytes(zero_start, 0, zero_len);
                }
            }
        }
    }

    let stack_start = VirtAddr::new(0x0000_7FFF_FFFF_0000);
    let stack_size_pages = 16;
    let stack_end_page: Page<Size4KiB> = Page::containing_address(stack_start - 1u64);
    let stack_start_page = stack_end_page - (stack_size_pages - 1) as u64;

    let stack_flags =
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    for page in Page::range_inclusive(stack_start_page, stack_end_page) {
        if memory::translate_addr(page.start_address()).is_none() {
            let frame_addr = {
                let mut pmm = PMM.lock();
                let pmm = pmm.as_mut().ok_or("PMM not initialized")?;
                pmm.alloc_frame().ok_or("No frames for stack")?
            };
            memory::map_page(page.start_address(), frame_addr, stack_flags)
                .map_err(|e| String::from(e))?;
        }
    }

    unsafe {
        crate::syscall::enter_userspace(elf.header.pt2.entry_point(), stack_start.as_u64());
    }
}

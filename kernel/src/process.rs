use alloc::string::{String, ToString};
use alloc::vec::Vec;
use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB},
};
use xmas_elf::ElfFile;
use xmas_elf::program::{ProgramHeader, Type};

use crate::fs::FILESYSTEM;
use crate::{memory, syscall};

pub fn load_elf(filename: &str) -> Result<(), String> {
    let file_data: Vec<u8> = {
        let mut fs_lock = FILESYSTEM.lock();
        let fs = fs_lock.as_mut().ok_or("Filesystem not initialized")?;
        fs.read_file(filename).ok_or("File not found")?
    }; // <- fs_lock is DROPPED here. 

    let elf = ElfFile::new(&file_data).map_err(|e| "Elf parse error")?;
    xmas_elf::header::sanity_check(&elf).map_err(|e| "ELF sanity check failed")?;

    let mut mapper = memory::get_mapper().ok_or("Memory map not initialized")?;
    let mut frame_allocator = memory::FRAME_ALLOCATOR.lock();
    let frame_allocator = frame_allocator
        .as_mut()
        .ok_or("Frame allocator not initialized")?;

    for ph in elf.program_iter() {
        if ph.get_type().map_err(|_| "Invalid Segment Type")? == Type::Load {
            let virt_addr = ph.virtual_addr();
            let file_size = ph.file_size();
            let mem_size = ph.mem_size();
            let file_offset = ph.offset();

            if virt_addr == 0 {
                continue;
            }

            // Round start address DOWN to nearest 4096
            let start_addr = VirtAddr::new(virt_addr);
            let start_page: Page<Size4KiB> = Page::containing_address(start_addr);

            // Round end address UP (virt_addr + mem_size)
            let end_addr = start_addr + mem_size;
            let end_page: Page<Size4KiB> = Page::containing_address(end_addr - 1u64);

            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE;

            for page in Page::range_inclusive(start_page, end_page) {
                // If page is not mapped, map it
                if memory::translate_addr(page.start_address()).is_none() {
                    let frame = frame_allocator.allocate_frame().ok_or("Out of memory")?;

                    unsafe {
                        mapper
                            .map_to(page, frame, flags, frame_allocator)
                            .map_err(|_| "Page mapping failed")?
                            .flush();
                    }
                }
            }

            unsafe {
                let src_ptr = file_data.as_ptr().add(file_offset as usize);
                let dest_ptr = virt_addr as *mut u8;
                core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, file_size as usize);
            }

            // If the memory segment is larger than the file data, the rest must be zero.
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
    let stack_size_pages = 16; // 64KiB stack
    let stack_end_page = Page::containing_address(stack_start - 1u64);
    let stack_start_page = stack_end_page - (stack_size_pages - 1) as u64;

    let stack_flags =
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    for page in Page::range_inclusive(stack_start_page, stack_end_page) {
        if memory::translate_addr(page.start_address()).is_none() {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or("No frames for stack")?;
            unsafe {
                mapper
                    .map_to(page, frame, stack_flags, frame_allocator)
                    .map_err(|_| "Stack map failed")?
                    .flush();
            }
        }
    }

    drop(frame_allocator);

    unsafe {
        syscall::enter_userspace(elf.header.pt2.entry_point(), stack_start.as_u64());
    }
}

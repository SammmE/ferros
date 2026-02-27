use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use spin::Mutex;
use x86_64::{
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
    PhysAddr,
};

pub static PMM: Mutex<Option<BitmapAllocator>> = Mutex::new(None);

const FRAME_SIZE: u64 = 4096;

pub struct BitmapAllocator {
    bitmap: &'static mut [u8],
    total_frames: usize,
    used_frames: usize,
    search_start: usize,
}

impl BitmapAllocator {
    pub unsafe fn init(regions: &'static MemoryRegions, phys_offset: u64) -> Self {
        let max_addr = regions
            .iter()
            .map(|r| r.end)
            .max()
            .unwrap_or(0);

        let total_frames = (max_addr / FRAME_SIZE) as usize;
        let bitmap_bytes = (total_frames + 7) / 8;
        let bitmap_frames = ((bitmap_bytes as u64) + FRAME_SIZE - 1) / FRAME_SIZE;

        let bitmap_phys = Self::find_free_region(regions, bitmap_frames as usize)
            .expect("No region large enough for PMM bitmap");

        let bitmap_virt = (phys_offset + bitmap_phys) as *mut u8;
        let bitmap = unsafe { core::slice::from_raw_parts_mut(bitmap_virt, bitmap_bytes) };

        bitmap.fill(0xFF);

        let mut alloc = BitmapAllocator {
            bitmap,
            total_frames,
            used_frames: total_frames,
            search_start: 0,
        };

        for region in regions.iter() {
            if region.kind == MemoryRegionKind::Usable {
                let start_frame = (region.start / FRAME_SIZE) as usize;
                let end_frame = (region.end / FRAME_SIZE) as usize;
                for frame in start_frame..end_frame {
                    alloc.clear_bit(frame);
                    alloc.used_frames -= 1;
                }
            }
        }

        let bitmap_start_frame = (bitmap_phys / FRAME_SIZE) as usize;
        for i in 0..bitmap_frames as usize {
            alloc.set_bit(bitmap_start_frame + i);
            alloc.used_frames += 1;
        }

        alloc
    }

    fn find_free_region(regions: &MemoryRegions, frames_needed: usize) -> Option<u64> {
        let bytes_needed = frames_needed as u64 * FRAME_SIZE;
        for region in regions.iter() {
            if region.kind == MemoryRegionKind::Usable {
                let size = region.end - region.start;
                if size >= bytes_needed {
                    let aligned = (region.start + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
                    if aligned + bytes_needed <= region.end {
                        return Some(aligned);
                    }
                }
            }
        }
        None
    }

    fn set_bit(&mut self, frame: usize) {
        if frame < self.total_frames {
            self.bitmap[frame / 8] |= 1 << (frame % 8);
        }
    }

    fn clear_bit(&mut self, frame: usize) {
        if frame < self.total_frames {
            self.bitmap[frame / 8] &= !(1 << (frame % 8));
        }
    }

    fn is_set(&self, frame: usize) -> bool {
        if frame >= self.total_frames {
            return true;
        }
        self.bitmap[frame / 8] & (1 << (frame % 8)) != 0
    }

    pub fn alloc_frame(&mut self) -> Option<PhysAddr> {
        for i in 0..self.total_frames {
            let idx = (self.search_start + i) % self.total_frames;
            if !self.is_set(idx) {
                self.set_bit(idx);
                self.used_frames += 1;
                self.search_start = idx + 1;
                return Some(PhysAddr::new(idx as u64 * FRAME_SIZE));
            }
        }
        None
    }

    pub fn free_frame(&mut self, addr: PhysAddr) {
        let frame = (addr.as_u64() / FRAME_SIZE) as usize;
        if self.is_set(frame) {
            self.clear_bit(frame);
            self.used_frames -= 1;
            if frame < self.search_start {
                self.search_start = frame;
            }
        }
    }

    pub fn alloc_contiguous(&mut self, count: usize) -> Option<PhysAddr> {
        if count == 0 {
            return None;
        }
        let mut run_start = 0;
        let mut run_len = 0;

        for i in 0..self.total_frames {
            if !self.is_set(i) {
                if run_len == 0 {
                    run_start = i;
                }
                run_len += 1;
                if run_len == count {
                    for j in run_start..run_start + count {
                        self.set_bit(j);
                    }
                    self.used_frames += count;
                    self.search_start = run_start + count;
                    return Some(PhysAddr::new(run_start as u64 * FRAME_SIZE));
                }
            } else {
                run_len = 0;
            }
        }
        None
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.used_frames, self.total_frames)
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.alloc_frame()
            .map(|addr| PhysFrame::containing_address(addr))
    }
}

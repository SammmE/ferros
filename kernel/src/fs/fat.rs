use crate::drivers::ata::AtaDrive;
use alloc::string::String;
use alloc::vec::Vec;

// stolen off OSDev
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct Bpb {
    jmp: [u8; 3],
    oem: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fats: u8,
    root_entries: u16,
    total_sectors_16: u16,
    media: u8,
    sectors_per_fat_16: u16,
    sectors_per_track: u16,
    heads: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    sectors_per_fat_32: u32,
    ext_flags: u16,
    fs_version: u16,
    root_cluster: u32,
    fs_info: u16,
    backup_boot_sector: u16,
    reserved: [u8; 12],
    drive_number: u8,
    reserved1: u8,
    boot_signature: u8,
    vol_id: u32,
    vol_label: [u8; 11],
    fs_type: [u8; 8],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct DirectoryEntry {
    pub name: [u8; 8],
    pub ext: [u8; 3],
    pub attributes: u8,
    pub reserved: u8,
    pub ctime_tenth: u8,
    pub ctime: u16,
    pub cdate: u16,
    pub adate: u16,
    pub cluster_high: u16,
    pub time: u16,
    pub date: u16,
    pub cluster_low: u16,
    pub size: u32,
}

impl DirectoryEntry {
    pub fn is_free(&self) -> bool {
        self.name[0] == 0xE5
    }

    pub fn is_end(&self) -> bool {
        self.name[0] == 0x00
    }

    pub fn is_long_name(&self) -> bool {
        self.attributes == 0x0F
    }

    pub fn get_cluster(&self) -> u32 {
        ((self.cluster_high as u32) << 16) | (self.cluster_low as u32)
    }

    pub fn get_filename(&self) -> String {
        let mut name = String::new();
        for &c in &self.name {
            if c != 0x20 {
                name.push(c as char);
            }
        }

        let mut ext = String::new();
        for &c in &self.ext {
            if c != 0x20 {
                ext.push(c as char);
            }
        }

        if !ext.is_empty() {
            name.push('.');
            name.push_str(&ext);
        }
        name
    }
}

pub struct Fat32Driver {
    pub drive: AtaDrive,
    pub fat_start_sector: u32,
    pub data_start_sector: u32,
    pub sectors_per_cluster: u32,
    pub root_cluster: u32,
}

impl Fat32Driver {
    /// Helper to bridge u16 ATA driver to u8 FAT driver
    fn read_sector_into_u8(&mut self, lba: u32, buffer: &mut [u8; 512]) {
        let mut raw_buffer = [0u16; 256];
        // Read 1 sector, passing the u16 buffer
        self.drive.read(lba, 1, &mut raw_buffer).unwrap();

        // Convert back to u8
        for (i, &word) in raw_buffer.iter().enumerate() {
            buffer[i * 2] = (word & 0xFF) as u8;
            buffer[i * 2 + 1] = ((word >> 8) & 0xFF) as u8;
        }
    }

    pub fn new(mut drive: AtaDrive) -> Self {
        let mut raw_buffer = [0u16; 256];
        // FIX: call read with sector count 1 and u16 buffer
        drive.read(0, 1, &mut raw_buffer).unwrap();

        // Manual conversion for BPB parsing
        let mut buf = [0u8; 512];
        for (i, &word) in raw_buffer.iter().enumerate() {
            buf[i * 2] = (word & 0xFF) as u8;
            buf[i * 2 + 1] = ((word >> 8) & 0xFF) as u8;
        }

        crate::serial_println!("DEBUG: Reading Sector 0...");
        crate::serial_print!("Hex: ");
        for i in 0..16 {
            crate::serial_print!("{:02X} ", buf[i]);
        }
        crate::serial_println!();

        let bpb = unsafe { &*(buf.as_ptr() as *const Bpb) };

        if bpb.bytes_per_sector != 512 {
            panic!("FAT32: Only 512 byte sectors supported");
        }

        let fat_size = bpb.sectors_per_fat_32;
        let fat_start_sector = bpb.reserved_sectors as u32;
        let root_cluster = bpb.root_cluster;
        let data_start_sector = fat_start_sector + (bpb.fats as u32 * fat_size);
        let sectors_per_cluster = bpb.sectors_per_cluster as u32;

        Self {
            drive,
            fat_start_sector,
            data_start_sector,
            sectors_per_cluster,
            root_cluster,
        }
    }

    fn cluster_to_lba(&self, cluster: u32) -> u32 {
        self.data_start_sector + ((cluster - 2) * self.sectors_per_cluster)
    }

    fn next_cluster(&mut self, current_cluster: u32) -> Option<u32> {
        let fat_offset = current_cluster * 4;
        let fat_sector = self.fat_start_sector + (fat_offset / 512);
        let ent_offset = (fat_offset % 512) as usize;

        let mut buf = [0u8; 512];
        // FIX: Removed `* 512` (ATA takes LBA, not bytes) and used helper
        self.read_sector_into_u8(fat_sector, &mut buf);

        let entry = unsafe {
            let ptr = buf.as_ptr().add(ent_offset) as *const u32;
            *ptr
        };

        let val = entry & 0x0FFF_FFFF;
        if val >= 0x0FFF_FFF8 { None } else { Some(val) }
    }

    fn read_cluster(&mut self, cluster: u32) -> Vec<u8> {
        let start_lba = self.cluster_to_lba(cluster);
        let mut data = Vec::with_capacity((self.sectors_per_cluster * 512) as usize);
        let mut buf = [0u8; 512];

        for i in 0..self.sectors_per_cluster {
            // FIX: Removed `* 512` and used helper
            self.read_sector_into_u8(start_lba + i, &mut buf);
            data.extend_from_slice(&buf);
        }
        data
    }

    pub fn list_root(&mut self) -> Vec<String> {
        let mut files = Vec::new();
        let mut current_cluster = Some(self.root_cluster);

        while let Some(cluster) = current_cluster {
            let data = self.read_cluster(cluster);

            for chunk in data.chunks(32) {
                if chunk.len() != 32 {
                    break;
                }
                let entry = unsafe { &*(chunk.as_ptr() as *const DirectoryEntry) };

                if entry.is_end() {
                    return files;
                }
                if entry.is_free() {
                    continue;
                }
                if entry.is_long_name() {
                    continue;
                }

                if entry.attributes != 0x0F {
                    files.push(entry.get_filename());
                }
            }
            current_cluster = self.next_cluster(cluster);
        }
        files
    }

    pub fn read_file(&mut self, filename: &str) -> Option<Vec<u8>> {
        let mut target_entry: Option<DirectoryEntry> = None;
        let mut current_cluster = Some(self.root_cluster);

        'search: while let Some(cluster) = current_cluster {
            let data = self.read_cluster(cluster);
            for chunk in data.chunks(32) {
                let entry = unsafe { &*(chunk.as_ptr() as *const DirectoryEntry) };
                if entry.is_end() {
                    break 'search;
                }
                if !entry.is_free() && !entry.is_long_name() {
                    if entry.get_filename().eq_ignore_ascii_case(filename) {
                        target_entry = Some(*entry);
                        break 'search;
                    }
                }
            }
            current_cluster = self.next_cluster(cluster);
        }

        if let Some(entry) = target_entry {
            let mut file_data = Vec::new();
            let mut current_cluster = Some(entry.get_cluster());

            while let Some(cluster) = current_cluster {
                let cluster_data = self.read_cluster(cluster);
                file_data.extend_from_slice(&cluster_data);
                current_cluster = self.next_cluster(cluster);
            }

            file_data.truncate(entry.size as usize);
            return Some(file_data);
        }

        None
    }
}

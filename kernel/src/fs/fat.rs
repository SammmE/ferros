use crate::drivers::ata::AtaDrive;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

// FAT32 implementation based on OSDev wiki
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
        drive.read(0, 1, &mut raw_buffer).unwrap();

        let mut buf = [0u8; 512];
        for (i, &word) in raw_buffer.iter().enumerate() {
            buf[i * 2] = (word & 0xFF) as u8;
            buf[i * 2 + 1] = ((word >> 8) & 0xFF) as u8;
        }

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
        self.read_sector_into_u8(fat_sector, &mut buf);

        let entry = unsafe {
            let ptr = buf.as_ptr().add(ent_offset) as *const u32;
            *ptr
        };

        let val = entry & 0x0FFF_FFFF;
        if val >= 0x0FFF_FFF8 {
            None
        } else {
            Some(val)
        }
    }

    fn read_cluster(&mut self, cluster: u32) -> Vec<u8> {
        let start_lba = self.cluster_to_lba(cluster);
        let mut data = Vec::with_capacity((self.sectors_per_cluster * 512) as usize);
        let mut buf = [0u8; 512];

        for i in 0..self.sectors_per_cluster {
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

    fn write_sector_from_u8(&mut self, lba: u32, buffer: &[u8; 512]) {
        let mut raw_buffer = [0u16; 256];
        for (i, word) in raw_buffer.iter_mut().enumerate() {
            *word = (buffer[i * 2] as u16) | ((buffer[i * 2 + 1] as u16) << 8);
        }
        self.drive.write(lba, 1, &raw_buffer).unwrap();
    }

    fn find_free_cluster(&mut self) -> Option<u32> {
        let mut cluster = 2;
        loop {
            let fat_offset = cluster * 4;
            let fat_sector = self.fat_start_sector + (fat_offset / 512);
            let ent_offset = (fat_offset % 512) as usize;

            let mut buf = [0u8; 512];
            self.read_sector_into_u8(fat_sector, &mut buf);

            let entry = unsafe {
                let ptr = buf.as_ptr().add(ent_offset) as *const u32;
                *ptr
            };

            if (entry & 0x0FFF_FFFF) == 0 {
                return Some(cluster);
            }
            cluster += 1;
            if cluster >= 0x0FFF_FFF7 {
                break;
            }
        }
        None
    }

    fn get_fat_entry(&mut self, cluster: u32) -> u32 {
        let fat_offset = cluster * 4;
        let fat_sector = self.fat_start_sector + (fat_offset / 512);
        let ent_offset = (fat_offset % 512) as usize;

        let mut buf = [0u8; 512];
        self.read_sector_into_u8(fat_sector, &mut buf);

        unsafe {
            let ptr = buf.as_ptr().add(ent_offset) as *const u32;
            (*ptr) & 0x0FFF_FFFF
        }
    }

    fn set_fat_entry(&mut self, cluster: u32, value: u32) {
        let fat_offset = cluster * 4;
        let fat_sector = self.fat_start_sector + (fat_offset / 512);
        let ent_offset = (fat_offset % 512) as usize;

        let mut buf = [0u8; 512];
        self.read_sector_into_u8(fat_sector, &mut buf);

        // Update the value in the buffer
        unsafe {
            let ptr = buf.as_mut_ptr().add(ent_offset) as *mut u32;
            *ptr = (*ptr & 0xF000_0000) | (value & 0x0FFF_FFFF);
        }

        // Write it back
        self.write_sector_from_u8(fat_sector, &buf);
    }

    fn total_clusters(&mut self) -> u32 {
        let total_sectors = self.drive.get_total_sectors().unwrap_or(0);

        if total_sectors <= self.data_start_sector {
            return 0;
        }

        let data_sectors = total_sectors - self.data_start_sector;
        data_sectors / self.sectors_per_cluster as u32
    }

    pub fn create_file(&mut self, filename: &str, data: &[u8]) -> Result<(), &'static str> {
        let clusters_needed = (data.len() as u32 + (self.sectors_per_cluster * 512 - 1))
            / (self.sectors_per_cluster * 512);

        if clusters_needed == 0 {
            return Err("Cannot create empty file (logic limitation)");
        }

        if self.file_exists(filename) {
            return Err("File already exists");
        }

        let mut allocated_clusters = Vec::new();

        // Find enough clusters
        for _ in 0..clusters_needed {
            if let Some(cluster) = self.find_free_cluster() {
                allocated_clusters.push(cluster);
                // Temporarily mark as EOF so find_free_cluster doesn't find it again immediately
                self.set_fat_entry(cluster, 0x0FFF_FFFF);
            } else {
                // Rollback: Free what we allocated if we run out of space
                for &c in &allocated_clusters {
                    self.set_fat_entry(c, 0);
                }
                return Err("Not enough free clusters");
            }
        }

        // Write Data
        for i in 0..allocated_clusters.len() {
            let start = (i as usize) * (self.sectors_per_cluster as usize) * 512;
            let end = core::cmp::min(
                start + (self.sectors_per_cluster as usize) * 512,
                data.len(),
            );
            let cluster_data = &data[start..end];

            // Buffer needs to be full cluster size to act as padding for the last sector
            let mut cluster_buffer = vec![0u8; (self.sectors_per_cluster * 512) as usize];
            cluster_buffer[..cluster_data.len()].copy_from_slice(cluster_data);

            let start_lba = self.cluster_to_lba(allocated_clusters[i]);

            // Write sectors for this cluster
            for j in 0..self.sectors_per_cluster {
                let sector_offset = (j * 512) as usize;
                let mut sector_buf = [0u8; 512];
                sector_buf.copy_from_slice(&cluster_buffer[sector_offset..sector_offset + 512]);
                self.write_sector_from_u8(start_lba + j, &sector_buf);
            }

            // Link FAT Chain
            if i < allocated_clusters.len() - 1 {
                self.set_fat_entry(allocated_clusters[i], allocated_clusters[i + 1]);
            } else {
                self.set_fat_entry(allocated_clusters[i], 0x0FFF_FFFF); // EOF
            }
        }

        // Add Directory Entry after checking if file exists
        self.add_directory_entry(filename, allocated_clusters[0], data.len() as u32)?;

        Ok(())
    }

    pub fn file_exists(&mut self, filename: &str) -> bool {
        let mut current_cluster = Some(self.root_cluster);

        while let Some(cluster) = current_cluster {
            let data = self.read_cluster(cluster);
            for chunk in data.chunks(32) {
                let entry = unsafe { &*(chunk.as_ptr() as *const DirectoryEntry) };
                if entry.is_end() {
                    return false;
                }
                if !entry.is_free() && !entry.is_long_name() {
                    if entry.get_filename().eq_ignore_ascii_case(filename) {
                        return true;
                    }
                }
            }
            current_cluster = self.next_cluster(cluster);
        }

        false
    }

    // Helper to add the directory entry (extracted for clarity)
    fn add_directory_entry(
        &mut self,
        filename: &str,
        start_cluster: u32,
        size: u32,
    ) -> Result<(), &'static str> {
        // 1. Format Filename (8.3 format)
        let mut name = [0x20u8; 8];
        let mut ext = [0x20u8; 3];

        let upper_name = filename.to_ascii_uppercase();
        let parts: Vec<&str> = upper_name.split('.').collect();

        if parts.is_empty() || parts[0].len() > 8 || (parts.len() > 1 && parts[1].len() > 3) {
            return Err("Invalid filename (Must be 8.3 format)");
        }

        for (i, byte) in parts[0].bytes().enumerate() {
            name[i] = byte;
        }
        if parts.len() > 1 {
            for (i, byte) in parts[1].bytes().enumerate() {
                ext[i] = byte;
            }
        }

        // 2. Find free slot in root directory
        let dir_sector = self.cluster_to_lba(self.root_cluster);
        let mut dir_buf = [0u8; 512];

        // Scan root cluster (Limitation: Only scans first cluster of root dir)
        self.read_sector_into_u8(dir_sector, &mut dir_buf);

        let mut entry_offset = 0;
        let mut found_spot = false;

        for i in (0..512).step_by(32) {
            if dir_buf[i] == 0x00 || dir_buf[i] == 0xE5 {
                entry_offset = i;
                found_spot = true;
                break;
            }
        }

        if !found_spot {
            return Err("Root directory full");
        }

        // 3. Write Entry
        let new_entry = DirectoryEntry {
            name,
            ext,
            attributes: 0x20,
            reserved: 0,
            ctime_tenth: 0,
            ctime: 0,
            cdate: 0,
            adate: 0,
            cluster_high: ((start_cluster >> 16) & 0xFFFF) as u16,
            time: 0,
            date: 0,
            cluster_low: (start_cluster & 0xFFFF) as u16,
            size,
        };

        unsafe {
            let ptr = dir_buf.as_mut_ptr().add(entry_offset) as *mut DirectoryEntry;
            *ptr = new_entry;
        }

        self.write_sector_from_u8(dir_sector, &dir_buf);
        Ok(())
    }
}

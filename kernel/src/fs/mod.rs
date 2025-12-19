pub mod fat;

use crate::drivers::ata::{AtaDrive, Bus};
use crate::fs::fat::Fat32Driver;
use crate::println;

use spin::Mutex;

pub static DRIVE: Mutex<Option<AtaDrive>> = Mutex::new(None);
pub static FILESYSTEM: Mutex<Option<Fat32Driver>> = Mutex::new(None);

pub fn init_fs() {
    let drive = AtaDrive::new(Bus::Primary, false);

    let driver = Fat32Driver::new(drive);

    // Lock the global mutex and move the drive instance into it
    *FILESYSTEM.lock() = Some(driver);

    // Optional: Print status
    println!("[Filesystem]: FAT32 Initialized on Primary Bus");
}

pub fn read_sector(lba: u32) -> Result<[u8; 512], &'static str> {
    // Lock the drive
    let mut lock = DRIVE.lock();

    if let Some(drive) = lock.as_mut() {
        // Create a buffer for the raw 16-bit data
        let mut raw_buffer = [0u16; 256];

        // Perform the read
        drive.read(lba, 1, &mut raw_buffer)?;

        // Convert [u16; 256] -> [u8; 512]
        let mut byte_buffer = [0u8; 512];
        for (i, &word) in raw_buffer.iter().enumerate() {
            // Split 16-bit word into two 8-bit bytes (Little Endian)
            byte_buffer[i * 2] = (word & 0xFF) as u8;
            byte_buffer[i * 2 + 1] = ((word >> 8) & 0xFF) as u8;
        }

        Ok(byte_buffer)
    } else {
        Err("Drive not initialized")
    }
}

pub mod fat;

use crate::drivers::ata::{AtaDrive, Bus};
use crate::fs::fat::Fat32Driver;

use spin::Mutex;

pub static FILESYSTEM: Mutex<Option<Fat32Driver>> = Mutex::new(None);

pub fn init_fs() {
    let drive = AtaDrive::new(Bus::Primary, false);

    let driver = Fat32Driver::new(drive);

    // Lock the global mutex and move the drive instance into it
    *FILESYSTEM.lock() = Some(driver);
}

pub fn read_sector(lba: u32) -> Result<[u8; 512], &'static str> {
    let mut lock = FILESYSTEM.lock();

    if let Some(fs) = lock.as_mut() {
        let mut raw_buffer = [0u16; 256];

        fs.drive.read(lba, 1, &mut raw_buffer)?;

        let mut byte_buffer = [0u8; 512];
        for (i, &word) in raw_buffer.iter().enumerate() {
            byte_buffer[i * 2] = (word & 0xFF) as u8;
            byte_buffer[i * 2 + 1] = ((word >> 8) & 0xFF) as u8;
        }

        Ok(byte_buffer)
    } else {
        Err("Filesystem not initialized")
    }
}

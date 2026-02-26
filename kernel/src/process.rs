use alloc::string::String;
use xmas_elf;

use crate::fs::FILESYSTEM;
use crate::println;

pub fn load_elf(filename: &str) {
    let mut fs_lock = FILESYSTEM.lock();

    if let Some(fs) = fs_lock.as_mut() {
        match fs.read_file(filename) {
            Some(bytes) => {
                let file = xmas_elf::ElfFile::new(&bytes);
                println!("{:?}", file);
            }
            None => {
                println!("File not found: {}", filename);
            }
        }
    } else {
        println!("Filesystem not initialized!");
    }
}

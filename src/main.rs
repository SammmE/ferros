use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};

const DISK_SIZE_MB: usize = 64;
const INPUT_DIR: &str = "disk";
const IMAGE_FILE: &str = "user_disk.img";
const OUTPUT_DIR: &str = "disk_modified";

fn main() {
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    let args: Vec<String> = env::args().collect();
    let uefi = match args.get(1).map(|s| s.as_str()) {
        Some("uefi") => true,
        Some("bios") => false,
        _ => {
            println!("Usage: cargo run -- [uefi|bios]");
            exit(1);
        }
    };

    // Prepare the Disk Image
    prepare_disk_image();

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.arg("-serial").arg("mon:stdio");
    cmd.arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

    // Attach generated disk image
    cmd.arg("-drive").arg(format!(
        "file={},format=raw,if=ide,index=1,media=disk",
        IMAGE_FILE
    ));

    if uefi {
        let prebuilt =
            Prebuilt::fetch(Source::LATEST, "target/ovmf").expect("failed to update prebuilt");
        let code = prebuilt.get_file(Arch::X64, FileType::Code);
        let vars = prebuilt.get_file(Arch::X64, FileType::Vars);

        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,unit=0,file={},readonly=on",
            code.display()
        ));
        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,unit=1,file={},snapshot=on",
            vars.display()
        ));
    } else {
        cmd.arg("-drive")
            .arg(format!("format=raw,file={bios_path}"));
    }

    let mut child = cmd.spawn().expect("failed to start qemu");
    let _ = child.wait(); // Wait for QEMU to close

    // This pulls everything out of user_disk.img into 'disk_modified/'
    extract_disk_image();
}

fn prepare_disk_image() {
    let disk_path = Path::new(INPUT_DIR);
    if !disk_path.exists() {
        fs::create_dir(disk_path).expect("failed to create input 'disk' directory");
        fs::write(disk_path.join("README.txt"), "Put files in this folder!").unwrap();
    }

    println!("Creating {}MB FAT32 disk image...", DISK_SIZE_MB);

    let file = fs::File::create(IMAGE_FILE).expect("failed to create img file");
    file.set_len((DISK_SIZE_MB * 1024 * 1024) as u64)
        .expect("failed to set file length");

    let status = Command::new("mkfs.fat")
        .arg("-F")
        .arg("32")
        .arg(IMAGE_FILE)
        .output()
        .expect("Failed to run mkfs.fat. Is 'dosfstools' installed?");

    if !status.status.success() {
        panic!(
            "mkfs.fat failed: {}",
            String::from_utf8_lossy(&status.stderr)
        );
    }

    println!("Copying files from '{}' to disk image...", INPUT_DIR);

    // pass each file individually because Rust doesn't expand "disk/*"
    let mut mcopy_cmd = Command::new("mcopy");
    mcopy_cmd.arg("-i").arg(IMAGE_FILE).arg("-s");

    let mut has_files = false;
    for entry in fs::read_dir(INPUT_DIR).expect("failed to read input dir") {
        let entry = entry.expect("failed to read directory entry");
        mcopy_cmd.arg(entry.path());
        has_files = true;
    }

    if has_files {
        // Destination is root of image
        mcopy_cmd.arg("::/");

        let status = mcopy_cmd.status().expect("Failed to run mcopy");
        if !status.success() {
            eprintln!("Warning: mcopy exited with error");
        }
    } else {
        println!("No files found in '{}', skipping copy.", INPUT_DIR);
    }
}

fn extract_disk_image() {
    println!("Extracting disk state to '{}'...", OUTPUT_DIR);
    let output_path = Path::new(OUTPUT_DIR);

    if output_path.exists() {
        fs::remove_dir_all(output_path).expect("failed to clear old output dir");
    }
    fs::create_dir(output_path).expect("failed to create output dir");

    // mcopy -i image.img -s -n ::/ disk_modified/
    // -n = no overwrite (doesn't matter since dir is empty)
    // -s = recursive
    let status = Command::new("mcopy")
        .arg("-i")
        .arg(IMAGE_FILE)
        .arg("-s")
        .arg("-n")
        .arg("::/") // Source (Root of image)
        .arg(OUTPUT_DIR) // Dest (Host folder)
        .status()
        .expect("Failed to run mcopy. Is 'mtools' installed?");

    if status.success() {
        println!("Disk snapshot saved to: {}", OUTPUT_DIR);
    } else {
        eprintln!("Warning: Failed to extract disk image.");
    }
}

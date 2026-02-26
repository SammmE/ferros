use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, ExitStatus};

const DISK_SIZE_MB: usize = 64;
const INPUT_DIR: &str = "disk";
const USER_PROGRAM_DIR: &str = "user_programs";
const IMAGE_FILE: &str = "user_disk.img";
const OUTPUT_DIR: &str = "disk_modified";

fn main() -> Result<(), Box<dyn Error>> {
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    let args: Vec<String> = env::args().collect();
    let uefi = match args.get(1).map(|s| s.as_str()) {
        Some("uefi") => true,
        Some("bios") => false,
        _ => return Err("Usage: cargo run -- [uefi|bios]".into()),
    };

    prepare_disk_image()?;

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.arg("-serial").arg("mon:stdio");
    cmd.arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

    cmd.arg("-drive").arg(format!(
        "file={},format=raw,if=ide,index=1,media=disk",
        IMAGE_FILE
    ));

    if uefi {
        let prebuilt = Prebuilt::fetch(Source::LATEST, "target/ovmf")?;
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

    let mut child = cmd.spawn()?;
    child.wait()?;

    extract_disk_image()?;
    Ok(())
}

fn prepare_user_programs() -> Result<(), Box<dyn Error>> {
    let user_programs_path = Path::new(USER_PROGRAM_DIR);
    if !user_programs_path.exists() {
        fs::create_dir_all(user_programs_path)?;
    }

    for entry in fs::read_dir(user_programs_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "rs") {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Invalid filename")?;
            let output_path = format!("{}/{}", INPUT_DIR, stem);

            println!("Compiling {} user program...", stem);
            let status = Command::new("rustc")
                .arg("-O")
                .arg("--target")
                .arg("x86_64-unknown-none")
                .arg("--crate-type")
                .arg("bin")
                .arg(&path)
                .arg("-o")
                .arg(&output_path)
                .status()?;

            if !status.success() {
                return Err(format!("Failed to compile {:?}", path).into());
            }
        }
    }
    Ok(())
}

fn prepare_disk_image() -> Result<(), Box<dyn Error>> {
    let disk_path = Path::new(INPUT_DIR);
    if !disk_path.exists() {
        fs::create_dir(disk_path)?;
        fs::write(disk_path.join("README.txt"), "Put files in this folder!")?;
    }

    let _ = prepare_user_programs();

    println!("Creating {}MB FAT32 disk image...", DISK_SIZE_MB);

    let file = fs::File::create(IMAGE_FILE)?;
    file.set_len((DISK_SIZE_MB * 1024 * 1024) as u64)?;

    let status = Command::new("mkfs.fat")
        .arg("-F")
        .arg("32")
        .arg(IMAGE_FILE)
        .output()?;

    if !status.status.success() {
        io::stderr().write_all(&status.stderr)?;
        return Err("mkfs.fat failed".into());
    }

    println!("Copying files from '{}' to disk image...", INPUT_DIR);

    let mut mcopy_cmd = Command::new("mcopy");
    mcopy_cmd.arg("-i").arg(IMAGE_FILE).arg("-s");

    let mut has_files = false;
    for entry in fs::read_dir(INPUT_DIR)? {
        let entry = entry?;
        mcopy_cmd.arg(entry.path());
        has_files = true;
    }

    if has_files {
        mcopy_cmd.arg("::/");
        let status = mcopy_cmd.status()?;
        if !status.success() {
            return Err("mcopy failed to copy files".into());
        }
    } else {
        println!("No files found in '{}', skipping copy.", INPUT_DIR);
    }
    Ok(())
}

fn extract_disk_image() -> Result<(), Box<dyn Error>> {
    println!("Extracting disk state to '{}'...", OUTPUT_DIR);
    let output_path = Path::new(OUTPUT_DIR);

    if output_path.exists() {
        fs::remove_dir_all(output_path)?;
    }
    fs::create_dir(output_path)?;

    let status = Command::new("mcopy")
        .arg("-i")
        .arg(IMAGE_FILE)
        .arg("-s")
        .arg("-n")
        .arg("::/")
        .arg(OUTPUT_DIR)
        .status()?;

    if !status.success() {
        return Err("Failed to extract disk image".into());
    }

    println!("Disk snapshot saved to: {}", OUTPUT_DIR);
    Ok(())
}

use crate::framebuffer::WRITER;
use crate::fs;
use crate::fs::FILESYSTEM;
use crate::task::keyboard::ScancodeStream;
use crate::{print, println};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1, layouts};

pub async fn runshell() {
    let prompt = "samux> ";

    let mut scancode_stream = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    loop {
        print!("{}", prompt);

        let mut input_buffer: String = String::new();

        while let Some(scancode) = scancode_stream.next().await {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => match character {
                            '\n' => {
                                println!();
                                break;
                            }
                            '\x08' => {
                                if !input_buffer.is_empty() {
                                    input_buffer.pop();
                                    print!("\x08 \x08");
                                }
                            }
                            _ => {
                                input_buffer.push(character);
                                print!("{}", character);
                            }
                        },
                        DecodedKey::RawKey(KeyCode::Backspace) => {
                            if !input_buffer.is_empty() {
                                input_buffer.pop();
                                print!("\x08 \x08");
                            }
                        }
                        DecodedKey::RawKey(_) => {}
                    }
                }
            }
        }

        let mut parts = input_buffer.split_whitespace();
        if let Some(command) = parts.next() {
            let args: Vec<&str> = parts.collect();
            let output = execute_command(command, &args);
            for line in output {
                println!("{}", line);
            }
        }
    }
}

fn execute_command(command: &str, args: &[&str]) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();

    match command {
        "help" => {
            output.push("BASIC COMMANDS:".to_string());
            output.push("  help - Show this help message".to_string());
            output.push("  echo [text] - Echo the provided text".to_string());
            output.push("  clear - Clear the screen".to_string());
            output.push("  exit - shutdown the system".to_string());

            output.push("FILESYSTEM COMMANDS:".to_string());
            output.push("  read_disk [lba] - Read a sector from disk".to_string());
            output.push("  ls - List files in the root directory".to_string());
            output.push("  cat [filename] - Display contents of a file".to_string());
            output.push("  write [filename] [content] - Create or overwrite a file".to_string());
            output.push("  disk_info - Show information about the disk".to_string());

            output.push("SYSTEM COMMANDS:".to_string());
            output.push("  test_userspace - Test syscall from userspace".to_string());
        }
        "echo" => {
            let echoed = args.join(" ");
            output.push(echoed);
        }
        "clear" => {
            if let Some(writer) = WRITER.lock().as_mut() {
                writer.clear();
            }
        }
        "exit" => {
            panic!("exit command issued!");
        }
        "read_disk" => {
            if args.len() < 1 {
                println!("Usage: read_disk <lba>");
                return output;
            }

            // Parse the sector number
            if let Ok(lba) = args[0].parse::<u32>() {
                println!("Reading sector {}...", lba);

                match fs::read_sector(lba) {
                    Ok(data) => {
                        println!("-- Hex Dump (First 64 bytes) --");
                        for i in 0..64 {
                            print!("{:02X} ", data[i]);
                            if (i + 1) % 16 == 0 {
                                println!();
                            }
                        }

                        println!("\n-- ASCII Preview --");
                        for i in 0..64 {
                            let c = data[i] as char;
                            if c.is_ascii_graphic() {
                                print!("{}", c);
                            } else {
                                print!(".");
                            }
                        }
                        println!();
                    }
                    Err(e) => println!("Error reading disk: {}", e),
                }
            } else {
                println!("Invalid sector number");
            }
        }

        "cat" => {
            if args.len() < 1 {
                println!("Usage: cat <filename>");
                return output;
            }

            let filename = args[0];

            // Lock the filesystem
            let mut fs_lock = FILESYSTEM.lock();

            if let Some(fs) = fs_lock.as_mut() {
                // Try to read the file
                match fs.read_file(filename) {
                    Some(data) => {
                        // Convert bytes to string (lossy ensures it doesn't crash on binary data)
                        let content = String::from_utf8_lossy(&data);
                        println!("{}", content);
                    }
                    None => {
                        println!("File not found: {}", filename);
                    }
                }
            } else {
                println!("Filesystem not initialized!");
            }
        }

        "ls" => {
            let mut fs_lock = FILESYSTEM.lock();
            if let Some(fs) = fs_lock.as_mut() {
                println!("Directory listing:");
                let files = fs.list_root();
                for file in files {
                    println!("  {}", file);
                }
            } else {
                println!("Filesystem not initialized!");
            }
        }

        "write" => {
            if args.len() < 2 {
                println!("Usage: write <filename> <content>");
                return output;
            }

            let filename = args[0];
            // Join the rest of the arguments into the content string
            let content = args[1..].join(" ");

            let mut fs_lock = FILESYSTEM.lock();
            if let Some(fs) = fs_lock.as_mut() {
                match fs.create_file(filename, content.as_bytes()) {
                    Ok(_) => println!("File '{}' written successfully.", filename),
                    Err(e) => println!("Error writing file: {}", e),
                }
            } else {
                println!("Filesystem not initialized!");
            }
        }

        "disk_info" => {
            let mut fs_lock = FILESYSTEM.lock();
            if let Some(fs) = fs_lock.as_mut() {
                match fs.drive.get_total_sectors() {
                    Ok(sectors) => {
                        let size_mb = (sectors * 512) / 1024 / 1024;
                        println!("Disk Info:");
                        println!("  Total Sectors: {}", sectors);
                        println!("  Size: {} MB", size_mb);
                    }
                    Err(e) => println!("Error identifying drive: {}", e),
                }
            } else {
                println!("Filesystem/Drive not initialized!");
            }
        }

        "test_userspace" => {
            crate::syscall::test_userspace_syscall();
        }

        _ => {
            println!("Unknown command: {}", command);
        }
    }

    output
}

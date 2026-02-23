#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};

use kernel::init_all;
use kernel::println;
use kernel::shell;
use kernel::task::{executor::Executor, Task};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init_all(boot_info);

    println!("Hello World from the Framebuffer!");

    let executor = Executor::new();
    executor.spawn(Task::new(shell::runshell()));

    executor.run();
}

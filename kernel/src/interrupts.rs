use crate::serial_println;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

// Solve Overlapping issue (PIC offsets start 1-15 and CPU exceptions 0-31)
pub const PIC_1_OFFSET: u8 = 32; // 32 and onwards are free now
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Set handlers for exceptions
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        // Double Fault needs special treatment with its own stack
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        // Set handlers for hardware interrupts
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        self as u8 as usize
    }
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_pics() {
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
        pics.write_masks(0xFC, 0xFF); // Enable timer and keyboard IRQs only
    }
}

pub fn init_pit() {
    let mut command_port = Port::new(0x43);
    let mut data_port = Port::new(0x40);

    // 0x36 = 0011 0110
    // Channel 0 | Access Lo/Hi byte | Mode 3 (Square Wave) | Binary
    unsafe {
        command_port.write(0x36 as u8);

        // 1193182 / 65536 = 18.2 Hz (Standard rate)
        // Send Low byte (0x00) then High byte (0x00) for divisor 65536
        data_port.write(0x00 as u8);
        data_port.write(0x00 as u8);
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::segmentation::{Segment, CS};

    serial_println!("EXCEPTION: PAGE FAULT");
    serial_println!("Accessed Address: {:?}", Cr2::read());
    serial_println!("Error Code: {:?}", error_code);
    serial_println!("{:#?}", stack_frame);

    // Check if the fault occurred in user mode (Ring 3)
    // The CS register's bottom 2 bits contain the Current Privilege Level (CPL)
    let cs = CS::get_reg();
    let privilege_level = cs.0 & 0x3;

    if privilege_level == 3 {
        // User mode fault - kill the process instead of panicking
        serial_println!("User process caused a page fault. Terminating process.");
        crate::println!("\nSegmentation Fault: Process terminated due to invalid memory access");

        // LIMITATION: No process management yet. In a full OS, this would terminate
        // the process and return control to the scheduler/shell. For now, we halt.
        loop {
            x86_64::instructions::hlt();
        }
    } else {
        // Kernel mode fault - this is a kernel bug, panic
        panic!("Kernel page fault - this is a bug in the OS!");
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

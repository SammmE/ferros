# FerrOS

![Language](https://img.shields.io/badge/language-Rust-rust.svg)
![Status](https://img.shields.io/badge/status-in%20development-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/SammmE/ferros/rust.yml)


A barebones, `no_std` operating system kernel written from scratch in Rust, primarily for educational purposes.

---

## 🦀 Why FerrOS?

This project is a journey into the world of low-level systems programming. The primary goals are:

* To learn the fundamentals of operating system design and implementation.
* To explore the power of Rust for writing safe and efficient systems-level code.
* To understand hardware interaction, memory management, and process scheduling from the ground up.
* To have fun building something challenging and rewarding!

---

## 🛠️ Building and Running

### Prerequisites

You'll need the following tools to build and run FerrOS:

1.  **Install QEMU:**
    ```sh
    sudo apt-get update && sudo apt-get install -y qemu-system-x86
    ```

2.  **Install Rust nightly toolchain:**
    ```sh
    rustup toolchain install nightly
    rustup override set nightly
    ```

3.  **Install rust-src component:**
    ```sh
    rustup component add rust-src
    ```

4.  **Install llvm-tools-preview component:**
    ```sh
    rustup component add llvm-tools-preview
    ```

5.  **Install bootimage:**
    ```sh
    cargo install bootimage
    ```

### Running

1.  **Clone the repository:**
    ```sh
    git clone [https://github.com/](https://github.com/)<YOUR_USERNAME>/ferros.git
    cd ferros
    ```

2.  **Run the OS:**
    ```sh
    cargo run -- uefi
    ```

---

## 🗺️ Project Roadmap

This is the high-level plan for FerrOS, from a "Hello, World!" kernel to a basic interactive system.

### Phase 1: The Core Kernel & Bootstrapping
* [x] **Project Setup:** Create a `no_std` Rust binary.
* [x] **Bootloader:** Integrate the `bootimage` crate to create a bootable image.
~~* [] **VGA Text Mode Driver:** Implement a basic logger to print formatted text to the screen.~~
* [] **Set up FrameBuffer**: Implement a basic framebuffer to log and print text.
* [x] **Panic Handler:** Implement a kernel panic function that prints info and halts.

### Phase 2: Interrupts & Memory Management
* [x] **GDT (Global Descriptor Table):** Set up a basic GDT.
* [x] **IDT (Interrupt Descriptor Table):** Implement an IDT to handle CPU exceptions (e.g., page faults, double faults).
* [ ] **Paging (Paging V4):** Implement a virtual memory manager, including mapping the kernel and setting up page tables.
* [ ] **Heap Allocator:** Provide a global allocator (like `linked_list_allocator`) to enable using `alloc` (e.g., `Box`, `Vec`).
* [ ] **Physical Frame Allocator:** Create an allocator to manage physical memory frames (e.g., a simple bitmap or free list).

### Phase 3: Hardware & Concurrency
* [ ] **PIC/APIC:** Handle hardware interrupts from the PIC (or APIC).
* [ ] **PIT (Programmable Interval Timer):** Set up the timer for scheduling.
* [ ] **PS/2 Keyboard Driver:** Read scancodes from the keyboard and translate them to characters.
* [ ] **Preemptive Multitasking:** Implement basic task switching and a simple scheduler.

### Phase 4: Userspace & Filesystems
* [ ] **Syscalls:** Design and implement a basic syscall interface.
* [ ] **Userspace:** Create the ability to load and run a simple program in user mode.
* [ ] **Basic Filesystem:** Implement a RAM-based filesystem (initrd) to load initial user programs.
* [ ] **Basic Shell:** Create a minimal interactive shell to test keyboard input and run commands.

---

## 🤝 Contributing

Contributions are welcome! This is a learning project, so feel free to open issues or pull requests.

### Guidelines

1.  **Fork** the repository.
2.  Create a new branch: `git checkout -b feature/your-new-feature`
3.  Make your changes.
4.  **Format your code:** `cargo fmt`
5.  **Lint your code:** `cargo clippy`
6.  Ensure the project builds: `cargo build`
7.  **Open a Pull Request** with a clear description of your changes.

---

## 📄 License

This project is licensed under the **MIT License**. See the `LICENSE` file for full details.

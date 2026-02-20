# Samux

![Language](https://img.shields.io/badge/language-Rust-rust.svg)
![Status](https://img.shields.io/badge/status-in%20development-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/SammmE/samux/rust.yml)


A barebones, `no_std` operating system kernel written from scratch in Rust, primarily for educational purposes.

---

## Why Samux?

This project is a journey into the world of low-level systems programming. The primary goals are:

* To learn the fundamentals of operating system design and implementation.
* To explore the power of Rust for writing safe and efficient systems-level code.
* To understand hardware interaction, memory management, and process scheduling from the ground up.
* To have fun building something challenging and rewarding!

---

## 🛠️ Building and Running

### Prerequisites

You'll need the following tools to build and run Samux:

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
    git clone https://github.com/sammme/samux.git
    cd samux
    ```

2.  **Run the OS:**
    ```sh
    cargo run -- uefi
    ```

---

## 🗺️ Project Roadmap

This is the high-level plan for Samux, from a "Hello, World!" kernel to a basic interactive system.

### Phase 1: The Core Kernel & Bootstrapping
* [x] **Project Setup:** Create a `no_std` Rust binary.
* [x] **Bootloader:** Integrate the `bootimage` crate to create a bootable image.
~~* [] **VGA Text Mode Driver:** Implement a basic logger to print formatted text to the screen.~~
* [x] **Set up FrameBuffer**: Implement a basic framebuffer to log and print text.
* [x] **Panic Handler:** Implement a kernel panic function that prints info and halts.

### Phase 2: Interrupts & Memory Management
* [x] **GDT (Global Descriptor Table):** Set up a basic GDT.
* [x] **IDT (Interrupt Descriptor Table):** Implement an IDT to handle CPU exceptions (e.g., page faults, double faults).
* [x] **Paging (Paging V4):** Implement a virtual memory manager, including mapping the kernel and setting up page tables.
* [x] **Heap Allocator:** Provide a global allocator (like `linked_list_allocator`) to enable using `alloc` (e.g., `Box`, `Vec`).
* [x] **Physical Frame Allocator:** Create an allocator to manage physical memory frames (e.g., a simple bitmap or free list).

### Phase 3: Hardware & Concurrency
* [x] **PIC/APIC:** Handle hardware interrupts from the PIC (or APIC).
* [x] **PIT (Programmable Interval Timer):** Set up the timer for scheduling.
* [x] **PS/2 Keyboard Driver:** Read scancodes from the keyboard and translate them to characters.
* [x] **Preemptive Multitasking:** Implement basic task switching and a simple scheduler.

### Phase 4: Userspace & Filesystems
* [ ] **Syscalls:** Design and implement a basic syscall interface.
* [ ] **Userspace:** Create the ability to load and run a simple program in user mode.
* [x] **Basic Filesystem:** Implement a FAT filesystem to load initial user programs.
* [x] **Basic Shell:** Create a minimal interactive shell to test keyboard input and run commands.

---
## Documentation

### Memory
[![](https://mermaid.ink/img/pako:eNqVVW1vmzAQ_iuWq-5TkkEgDfHWSk2zpFVDi9o1H5ZUmYsNQSUYGViTVf3vs40hL42q1UjIPt_z3PmeM7xCnxEKEQw5ThdgfDdLgBhZ8VQaJu50EvG8wDFw6ZLx9WPpIAeJOPXziCXgZ39jncwvo3AxNVZDMea7r-9P_Oya8oTG4D7FPn1ECD2r9TZ-hNNpq9UCNyxpXuCEJZEvwgsrEFYJCXG67T_si2i2YRhzo3rJQC5OU0rAkOMlfSqCgHKFlceK_Gyb4D7H_rPgUPDuJmNN9JBRDpTPb8lQiOVOeE6pyldOQFUtdb4q4UDsbEMuRM2rePplbxL_scLLNKbA40xkuwTSG3wBA5zjQ_FvijjeI9uqgtwFowJzAjwcqopzKhj-UFKS0ERM9lT33Km3WGeq7v8puzd33atbKftgK_rIexAVIZSBu3P3cPk91S9iW9VQzqXv4cp5pVRK0t0jt7eC1qkrPxAwrd774nlzWeMDdObHdFqZQ4TXl1S0r25yOT_U48JNWfZkMzdBNcGe9O95JqPzfe37c0eTjGmI_TUQPqBfX4Bd9UuWMXt530GmZulf3d5_7TOWxwwTcRPuNMGHrXR8DOT9i5JQS11-FkCz1Tyrj1_tDPvg9PRMt1BlVJKBpnLfqF7tqsKUm7WEVWw_xlk2oAEoywWCKI7RURD4YjSynLNnio4sy9Lz5ktE8gUy09W3PbzUVqN9X-I_ha5avWaQHJ9ikM1fZy_Hh-jKQHC2wJzjNQId0NnnrBTTvNSQz-fOJT7EOqmefGowIWQXbAgwbIhfS0QgynlBG3BJ-RLLJXyVtDOYL6hQDiIxjUWD5DM4S94EKMXJL8aWFY6zIlxAFOA4E6siJTingwjLW1i7iPaj_IIVSQ5Rx1EUEL3CFURW22z1nLZh2WbH7Nqm3YBriOyTVrfdtmyn51idjmM5bw34V8U0Wj3jpG1bvW6vZxlOt9ttQEqinHG3_Fv6LAmiEL79A2SXO2w?type=png)](https://mermaid.live/edit#pako:eNqVVW1vmzAQ_iuWq-5TkkEgDfHWSk2zpFVDi9o1H5ZUmYsNQSUYGViTVf3vs40hL42q1UjIPt_z3PmeM7xCnxEKEQw5ThdgfDdLgBhZ8VQaJu50EvG8wDFw6ZLx9WPpIAeJOPXziCXgZ39jncwvo3AxNVZDMea7r-9P_Oya8oTG4D7FPn1ECD2r9TZ-hNNpq9UCNyxpXuCEJZEvwgsrEFYJCXG67T_si2i2YRhzo3rJQC5OU0rAkOMlfSqCgHKFlceK_Gyb4D7H_rPgUPDuJmNN9JBRDpTPb8lQiOVOeE6pyldOQFUtdb4q4UDsbEMuRM2rePplbxL_scLLNKbA40xkuwTSG3wBA5zjQ_FvijjeI9uqgtwFowJzAjwcqopzKhj-UFKS0ERM9lT33Km3WGeq7v8puzd33atbKftgK_rIexAVIZSBu3P3cPk91S9iW9VQzqXv4cp5pVRK0t0jt7eC1qkrPxAwrd774nlzWeMDdObHdFqZQ4TXl1S0r25yOT_U48JNWfZkMzdBNcGe9O95JqPzfe37c0eTjGmI_TUQPqBfX4Bd9UuWMXt530GmZulf3d5_7TOWxwwTcRPuNMGHrXR8DOT9i5JQS11-FkCz1Tyrj1_tDPvg9PRMt1BlVJKBpnLfqF7tqsKUm7WEVWw_xlk2oAEoywWCKI7RURD4YjSynLNnio4sy9Lz5ktE8gUy09W3PbzUVqN9X-I_ha5avWaQHJ9ikM1fZy_Hh-jKQHC2wJzjNQId0NnnrBTTvNSQz-fOJT7EOqmefGowIWQXbAgwbIhfS0QgynlBG3BJ-RLLJXyVtDOYL6hQDiIxjUWD5DM4S94EKMXJL8aWFY6zIlxAFOA4E6siJTingwjLW1i7iPaj_IIVSQ5Rx1EUEL3CFURW22z1nLZh2WbH7Nqm3YBriOyTVrfdtmyn51idjmM5bw34V8U0Wj3jpG1bvW6vZxlOt9ttQEqinHG3_Fv6LAmiEL79A2SXO2w)
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

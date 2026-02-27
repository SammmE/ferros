# Samux

![Language](https://img.shields.io/badge/language-Rust-rust.svg)
![Status](https://img.shields.io/badge/status-in%20development-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/SammmE/samux/rust.yml)

A `no_std` operating system kernel written from scratch in Rust.

---

## Goals

* Learn OS design and implementation fundamentals.
* Explore Rust for safe, efficient systems-level code.
* Understand hardware interaction, memory management, and process scheduling from the ground up.

---

## Building and Running

### Prerequisites

1.  **QEMU:**
    ```sh
    sudo apt-get update && sudo apt-get install -y qemu-system-x86
    ```

2.  **Rust nightly toolchain:**
    ```sh
    rustup toolchain install nightly
    rustup override set nightly
    ```

3.  **rust-src component:**
    ```sh
    rustup component add rust-src
    ```

4.  **llvm-tools-preview component:**
    ```sh
    rustup component add llvm-tools-preview
    ```

5.  **bootimage:**
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

## Roadmap

### Phase 1: Core Kernel and Bootstrapping
* [x] Project setup (`no_std` Rust binary)
* [x] Bootloader integration (`bootimage` crate)
* [x] Framebuffer-based text output
* [x] Panic handler

### Phase 2: Interrupts and Memory Management
* [x] GDT (Global Descriptor Table)
* [x] IDT (Interrupt Descriptor Table)
* [x] Paging (4-level page tables)
* [x] Heap allocator (`linked_list_allocator`)
* [x] Physical frame allocator

### Phase 3: Hardware and Concurrency
* [x] PIC/APIC interrupt handling
* [x] PIT (Programmable Interval Timer)
* [x] PS/2 keyboard driver
* [x] Preemptive multitasking

### Phase 4: Userspace and Filesystems
* [ ] Syscall interface
* [ ] Userspace program execution
* [x] FAT32 filesystem
* [x] Basic shell

---

## Documentation

### Memory Layout

[![](https://mermaid.ink/img/pako:eNqVVW1vmzAQ_iuWq-5TkkEgDfHWSk2zpFVDi9o1H5ZUmYsNQSUYGViTVf3vs40hL42q1UjIPt_z3PmeM7xCnxEKEQw5ThdgfDdLgBhZ8VQaJu50EvG8wDFw6ZLx9WPpIAeJOPXziCXgZ39jncwvo3AxNVZDMea7r-9P_Oya8oTG4D7FPn1ECD2r9TZ-hNNpq9UCNyxpXuCEJZEvwgsrEFYJCXG67T_si2i2YRhzo3rJQC5OU0rAkOMlfSqCgHKFlceK_Gyb4D7H_rPgUPDuJmNN9JBRDpTPb8lQiOVOeE6pyldOQFUtdb4q4UDsbEMuRM2rePplbxL_scLLNKbA40xkuwTSG3wBA5zjQ_FvijjeI9uqgtwFowJzAjwcqopzKhj-UFKS0ERM9lT33Km3WGeq7v8puzd33atbKftgK_rIexAVIZSBu3P3cPk91S9iW9VQzqXv4cp5pVRK0t0jt7eC1qkrPxAwrd774nlzWeMDdObHdFqZQ4TXl1S0r25yOT_U48JNWfZkMzdBNcGe9O95JqPzfe37c0eTjGmI_TUQPqBfX4Bd9UuWMXt530GmZulf3d5_7TOWxwwTcRPuNMGHrXR8DOT9i5JQS11-FkCz1Tyrj1_tDPvg9PRMt1BlVJKBpnLfqF7tqsKUm7WEVWw_xlk2oAEoywWCKI7RURD4YjSynLNnio4sy9Lz5ktE8gUy09W3PbzUVqN9X-I_ha5avWaQHJ9ikM1fZy_Hh-jKQHC2wJzjNQId0NnnrBTTvNSQz-fOJT7EOqmefGowIWQXbAgwbIhfS0QgynlBG3BJ-RLLJXyVtDOYL6hQDiIxjUWD5DM4S94EKMXJL8aWFY6zIlxAFOA4E6siJTingwjLW1i7iPaj_IIVSQ5Rx1EUEL3CFURW22z1nLZh2WbH7Nqm3YBriOyTVrfdtmyn51idjmM5bw34V8U0Wj3jpG1bvW6vZxlOt9ttQEqinHG3_Fv6LAmiEL79A2SXO2w?type=png)](https://mermaid.live/edit#pako:eNqVVW1vmzAQ_iuWq-5TkkEgDfHWSk2zpFVDi9o1H5ZUmYsNQSUYGViTVf3vs40hL42q1UjIPt_z3PmeM7xCnxEKEQw5ThdgfDdLgBhZ8VQaJu50EvG8wDFw6ZLx9WPpIAeJOPXziCXgZ39jncwvo3AxNVZDMea7r-9P_Oya8oTG4D7FPn1ECD2r9TZ-hNNpq9UCNyxpXuCEJZEvwgsrEFYJCXG67T_si2i2YRhzo3rJQC5OU0rAkOMlfSqCgHKFlceK_Gyb4D7H_rPgUPDuJmNN9JBRDpTPb8lQiOVOeE6pyldOQFUtdb4q4UDsbEMuRM2rePplbxL_scLLNKbA40xkuwTSG3wBA5zjQ_FvijjeI9uqgtwFowJzAjwcqopzKhj-UFKS0ERM9lT33Km3WGeq7v8puzd33atbKftgK_rIexAVIZSBu3P3cPk91S9iW9VQzqXv4cp5pVRK0t0jt7eC1qkrPxAwrd774nlzWeMDdObHdFqZQ4TXl1S0r25yOT_U48JNWfZkMzdBNcGe9O95JqPzfe37c0eTjGmI_TUQPqBfX4Bd9UuWMXt530GmZulf3d5_7TOWxwwTcRPuNMGHrXR8DOT9i5JQS11-FkCz1Tyrj1_tDPvg9PRMt1BlVJKBpnLfqF7tqsKUm7WEVWw_xlk2oAEoywWCKI7RURD4YjSynLNnio4sy9Lz5ktE8gUy09W3PbzUVqN9X-I_ha5avWaQHJ9ikM1fZy_Hh-jKQHC2wJzjNQId0NnnrBTTvNSQz-fOJT7EOqmefGowIWQXbAgwbIhfS0QgynlBG3BJ-RLLJXyVtDOYL6hQDiIxjUWD5DM4S94EKMXJL8aWFY6zIlxAFOA4E6siJTingwjLW1i7iPaj_IIVSQ5Rx1EUEL3CFURW22z1nLZh2WbH7Nqm3YBriOyTVrfdtmyn51idjmM5bw34V8U0Wj3jpG1bvW6vZxlOt9ttQEqinHG3_Fv6LAmiEL79A2SXO2w)

---

## Contributing

Contributions are welcome. Please follow these guidelines:

1.  **Fork** the repository.
2.  Create a new branch: `git checkout -b feature/your-new-feature`
3.  Make your changes.
4.  **Format your code:** `cargo fmt`
5.  **Lint your code:** `cargo clippy`
6.  Ensure the project builds: `cargo build`
7.  **Open a Pull Request** with a clear description of your changes.

---

## License

This project is licensed under the **MIT License**. See the `LICENSE` file for details.

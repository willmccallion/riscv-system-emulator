//! Binary Loader and System Initialization.
//!
//! This module provides utilities for loading binary files (kernels, firmware,
//! disk images) into system memory and setting up the initial CPU state for
//! execution.

use crate::config::Config;
use crate::core::arch::csr;
use crate::core::arch::mode::PrivilegeMode;
use crate::core::Cpu;
use crate::isa::abi;
use crate::isa::privileged::opcodes as sys_ops;
use std::fs;
use std::process;

/// Loads a binary file from disk into memory.
pub fn load_binary(path: &str) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| {
        eprintln!("\n[!] FATAL: Could not read file '{}': {}", path, e);
        process::exit(1);
    })
}

/// Sets up kernel loading for Linux or bare metal execution.
pub fn setup_kernel_load(
    cpu: &mut Cpu,
    config: &Config,
    _disk_path: &str,
    dtb_path: Option<String>,
    kernel_path_override: Option<String>,
) {
    let ram_base = config.system.ram_base;

    let opensbi_addr = ram_base;
    let kernel_addr = ram_base + 0x200000;
    let dtb_addr = ram_base + 0x2200000;

    if let Some(path) = dtb_path {
        let dtb_data = load_binary(&path);
        println!(
            "[Loader] Loading DTB ({} bytes) @ {:#x}",
            dtb_data.len(),
            dtb_addr
        );
        cpu.bus.load_binary_at(&dtb_data, dtb_addr);
    }

    let sbi_path = "software/linux/output/fw_jump.bin";

    if fs::metadata(sbi_path).is_ok() {
        println!("[Loader] Linux Mode Detected.");

        let sbi_data = load_binary(sbi_path);
        println!(
            "[Loader] Loading OpenSBI ({} bytes) @ {:#x}",
            sbi_data.len(),
            opensbi_addr
        );
        cpu.bus.load_binary_at(&sbi_data, opensbi_addr);

        let default_kernel_path = "software/linux/output/Image";
        let kernel_path = kernel_path_override
            .as_deref()
            .unwrap_or(default_kernel_path);

        if fs::metadata(kernel_path).is_ok() {
            let kernel_data = load_binary(kernel_path);
            println!(
                "[Loader] Loading Linux Kernel ({} bytes) @ {:#x}",
                kernel_data.len(),
                kernel_addr
            );
            cpu.bus.load_binary_at(&kernel_data, kernel_addr);
        } else {
            println!("[Loader] WARNING: Linux Image not found at {}", kernel_path);
        }

        cpu.pc = opensbi_addr;
        cpu.privilege = PrivilegeMode::Machine;
        cpu.regs.write(abi::REG_A0, 0);
        cpu.regs.write(abi::REG_A1, dtb_addr);
        cpu.regs.write(abi::REG_A2, 0);
    } else {
        println!("[Loader] Bare Metal Mode.");
        let load_addr = ram_base + config.system.kernel_offset;

        cpu.bus
            .load_binary_at(&sys_ops::MRET.to_le_bytes(), ram_base);
        cpu.pc = ram_base;
        cpu.privilege = PrivilegeMode::Machine;
        cpu.csr_write(csr::MEPC, load_addr);
        cpu.regs.write(abi::REG_A0, 0);
        cpu.regs.write(abi::REG_A1, dtb_addr);
    }
}

//! RISC-V System Simulator CLI.
//!
//! The main executable for the simulator. It handles command-line argument
//! parsing, system initialization, and the main simulation loop.
//!
//! # Usage
//!
//! The simulator can run in two modes:
//! 1. **Direct Mode**: Loads a flat binary file directly into RAM and executes it.
//! 2. **OS Mode**: Loads a kernel image, device tree, and optional disk image for
//!    full operating system simulation.

use clap::Parser;
use std::{fs, process};

extern crate riscv_emulator;

use riscv_emulator::config::Config;
use riscv_emulator::core::arch::mode::PrivilegeMode;
use riscv_emulator::core::Cpu;
use riscv_emulator::isa::abi;
use riscv_emulator::sim::loader;
use riscv_emulator::soc::System;

/// Command-line arguments for the RISC-V system simulator.
///
/// Supports two execution modes: direct binary execution and full OS boot
/// with kernel, device tree, and disk image support.
#[derive(Parser, Debug)]
#[command(author, version, about = "RISC-V Cycle-Accurate Simulator")]
struct Args {
    #[arg(short, long, default_value = "hardware/configs/default.toml")]
    config: String,

    #[arg(short, long)]
    file: Option<String>,

    #[arg(long)]
    kernel: Option<String>,

    #[arg(long, default_value = "")]
    disk: String,

    #[arg(long)]
    dtb: Option<String>,
}

/// Main entry point for the RISC-V system simulator.
///
/// # Behavior
///
/// 1. **Configuration**: Parses command-line arguments and loads the TOML configuration file.
/// 2. **Initialization**: Constructs the `System` (bus, memory, devices) and the `Cpu`.
/// 3. **Loader**:
///    - If `--kernel` is provided, sets up the environment for an OS boot (OpenSBI + Linux).
///    - If `--file` is provided, loads a raw binary for bare-metal execution.
/// 4. **Simulation Loop**: Ticks the CPU cycle-by-cycle until a shutdown signal (exit code)
///    is received or a fatal error occurs.
/// 5. **Teardown**: Prints simulation statistics and exits with the target's exit code.
fn main() {
    let args = Args::parse();
    let config_content = fs::read_to_string(&args.config).expect("Failed to read config");
    let config: Config = toml::from_str(&config_content).expect("Failed to parse config");

    let system = System::new(&config, &args.disk);
    let mut cpu = Cpu::new(system, &config);

    println!("Global Configuration");
    println!("--------------------");
    println!("General:");
    println!(
        "  Trace Instructions: {}",
        config.general.trace_instructions
    );
    println!("  Start PC:           {:#x}", config.general.start_pc);
    println!("System:");
    println!("  RAM Base:           {:#x}", config.system.ram_base);
    println!(
        "  RAM Size:           {} MB",
        config.memory.ram_size / 1024 / 1024
    );
    println!("  Kernel Offset:      {:#x}", config.system.kernel_offset);
    println!("Pipeline:");
    println!("  Width:              {}", config.pipeline.width);
    println!(
        "  Branch Predictor:   {:?}",
        config.pipeline.branch_predictor
    );
    println!("  BTB Size:           {}", config.pipeline.btb_size);
    println!("  RAS Size:           {}", config.pipeline.ras_size);
    println!("Cache Hierarchy:");
    println!(
        "  L1-I:               {} ({} KB, {} ways)",
        if config.cache.l1_i.enabled {
            "Enabled"
        } else {
            "Disabled"
        },
        config.cache.l1_i.size_bytes / 1024,
        config.cache.l1_i.ways
    );
    println!(
        "  L1-D:               {} ({} KB, {} ways)",
        if config.cache.l1_d.enabled {
            "Enabled"
        } else {
            "Disabled"
        },
        config.cache.l1_d.size_bytes / 1024,
        config.cache.l1_d.ways
    );
    println!(
        "  L2:                 {} ({} KB, {} ways)",
        if config.cache.l2.enabled {
            "Enabled"
        } else {
            "Disabled"
        },
        config.cache.l2.size_bytes / 1024,
        config.cache.l2.ways
    );
    println!(
        "  L3:                 {} ({} KB, {} ways)",
        if config.cache.l3.enabled {
            "Enabled"
        } else {
            "Disabled"
        },
        config.cache.l3.size_bytes / 1024,
        config.cache.l3.ways
    );
    println!("--------------------");

    if let Some(kernel_path) = args.kernel {
        println!("[*] OS Boot Mode");
        println!("    Kernel: {}", kernel_path);
        if !args.disk.is_empty() {
            println!("    Disk:   {}", args.disk);
        }
        if let Some(ref dtb) = args.dtb {
            println!("    DTB:    {}", dtb);
        }
        loader::setup_kernel_load(&mut cpu, &config, &args.disk, args.dtb, Some(kernel_path));
    } else if let Some(bin_path) = args.file {
        println!("[*] Direct Execution Mode");
        let bin_data = loader::load_binary(&bin_path);
        let load_addr = config.system.ram_base;

        println!(
            "[Loader] Writing {} bytes to {:#x}",
            bin_data.len(),
            load_addr
        );
        cpu.bus.load_binary_at(&bin_data, load_addr);
        cpu.pc = load_addr;

        let stack_top = load_addr + 0x1000000;
        cpu.regs.write(abi::REG_SP, stack_top);

        cpu.direct_mode = true;
        cpu.privilege = PrivilegeMode::User;

        use riscv_emulator::core::arch::csr;
        let trap_handler_addr = stack_top + 0x1000;
        cpu.csr_write(csr::MTVEC, trap_handler_addr);

        cpu.csr_write(csr::MEDELEG, 0);
        cpu.csr_write(csr::MIDELEG, 0);
    } else {
        eprintln!("Error: No binary specified.");
        eprintln!("Usage:");
        eprintln!("  Direct mode:  --file <binary.bin>");
        eprintln!("  OS mode:      --kernel <Image> [--disk <disk.img>] [--dtb <system.dtb>]");
        process::exit(1);
    }

    loop {
        if let Err(e) = cpu.tick() {
            eprintln!("\n[!] FATAL TRAP: {}", e);
            cpu.dump_state();
            cpu.stats.print();
            process::exit(1);
        }

        if let Some(code) = cpu.take_exit() {
            println!("\n[*] Exiting with code {}", code);
            cpu.stats.print();

            use std::io::Write;
            std::io::stdout().flush().ok();

            process::exit(code as i32);
        }
    }
}

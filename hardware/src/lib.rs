//! RISC-V 64-bit System Simulator Library.
//!
//! This crate implements a cycle-accurate simulator for the RISC-V 64-bit architecture
//! (RV64IMAFDC). It includes a 5-stage pipeline, memory hierarchy (cache/MMU), and
//! System-on-Chip peripherals.
//!
//! # Architecture
//!
//! * **Core**: 5-stage in-order pipeline (Fetch, Decode, Execute, Memory, Writeback).
//! * **Memory**: SV39 Virtual Memory, TLBs, and multi-level set-associative caches.
//! * **Peripherals**: UART, CLINT, PLIC, VirtIO Block Device.
//!
//! # Modules
//!
//! * `common`: Shared types, constants, and error handling.
//! * `config`: Configuration loading and parsing.
//! * `core`: CPU core implementation.
//! * `isa`: Instruction Set Architecture definitions.
//! * `sim`: Simulation harness and loaders.
//! * `soc`: System-on-Chip component implementations.
//! * `stats`: Performance statistics collection.

/// Shared types, constants, error handling, and register definitions.
///
/// Provides fundamental data structures and error types used throughout
/// the simulator, including register file abstractions and common constants.
pub mod common;

/// Configuration system for processor, memory, cache, and pipeline settings.
///
/// Loads and parses TOML configuration files to customize simulator behavior
/// for different simulation scenarios and hardware configurations.
pub mod config;

/// CPU core implementation including pipeline stages and execution units.
///
/// Implements the 5-stage in-order pipeline (Fetch, Decode, Execute, Memory, Writeback),
/// architectural state management, and trap handling.
pub mod core;

/// Instruction Set Architecture definitions and decoders.
///
/// Implements RISC-V RV64IMAFDC instruction decoding, encoding, and ISA-specific
/// modules for base integer, multiply, atomic, floating-point, and compressed extensions.
pub mod isa;

/// Simulation harness, binary loaders, and execution orchestration.
///
/// Handles loading binaries and kernel images, setting up execution environments,
/// and coordinating the simulation loop.
pub mod sim;

/// System-on-Chip components including memory controllers and peripherals.
///
/// Implements the memory hierarchy, bus interconnect, and MMIO devices (UART,
/// CLINT, PLIC, VirtIO disk) that form the complete system.
pub mod soc;

/// Performance statistics collection and reporting.
///
/// Tracks cycle counts, instruction counts, cache statistics, and other
/// performance metrics during simulation execution.
pub mod stats;

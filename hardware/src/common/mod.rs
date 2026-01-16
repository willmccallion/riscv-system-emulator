//! Common utilities and types used throughout the RISC-V system simulator.
//!
//! This module provides fundamental types for addresses, memory access,
//! error handling, and register management that are shared across
//! different components of the simulator.

/// Address type definitions (physical and virtual addresses).
pub mod addr;

/// Common constants used throughout the simulator.
pub mod constants;

/// Memory access type definitions.
pub mod data;

/// Error types and trap definitions.
pub mod error;

/// Register file implementation.
pub mod reg;

pub use addr::{PhysAddr, VirtAddr};
pub use data::AccessType;
pub use error::{TranslationResult, Trap};
pub use reg::RegisterFile;

pub use constants::{PAGE_SHIFT, VPN_MASK};

//! RISC-V architecture-specific components.
//!
//! This module contains implementations of core RISC-V architectural
//! elements including control and status registers (CSRs), register files,
//! privilege modes, and trap handling.

/// Control and Status Register (CSR) definitions and access logic.
pub mod csr;

/// Floating-Point Register file implementation.
pub mod fpr;

/// General-Purpose Register file implementation.
pub mod gpr;

/// Privilege mode definitions and transitions.
pub mod mode;

/// Trap handling and exception processing.
pub mod trap;

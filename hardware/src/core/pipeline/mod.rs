//! Instruction pipeline implementation.
//!
//! This module contains the five-stage instruction pipeline (fetch, decode,
//! execute, memory, writeback), pipeline latches for inter-stage communication,
//! hazard detection and forwarding logic, and control signals.

/// Pipeline hazard detection and forwarding logic.
pub mod hazards;

/// Inter-stage pipeline latches (IF/ID, ID/EX, EX/MEM, MEM/WB).
pub mod latches;

/// Control signals generated during instruction decode.
pub mod signals;

/// Pipeline stage implementations (fetch, decode, execute, memory, writeback).
pub mod stages;

/// Traits for pipeline stage components.
pub mod traits;

//! Trap Handling Utilities.
//!
//! This module provides helper functions for mapping hardware interrupt
//! pending bits (from the MIP register) to high-level `Trap` enum variants.

use crate::common::error::Trap;

/// Trap handler utility functions.
///
/// Provides helper functions for converting between interrupt
/// representations and trap types.
pub struct TrapHandler;

impl TrapHandler {
    /// Converts an interrupt pending bit to a corresponding trap type.
    ///
    /// # Arguments
    ///
    /// * `bit` - The interrupt pending bit from the MIP register
    ///
    /// # Returns
    ///
    /// The `Trap` variant corresponding to the interrupt type, or
    /// `MachineTimerInterrupt` as a default for unrecognized bits.
    pub fn irq_to_trap(bit: u64) -> Trap {
        use crate::core::arch::csr;
        match bit {
            csr::MIP_USIP => Trap::UserSoftwareInterrupt,
            csr::MIP_SSIP => Trap::SupervisorSoftwareInterrupt,
            csr::MIP_MSIP => Trap::MachineSoftwareInterrupt,
            csr::MIP_STIP => Trap::SupervisorTimerInterrupt,
            csr::MIP_MTIP => Trap::MachineTimerInterrupt,
            csr::MIP_SEIP | csr::MIP_MEIP => Trap::ExternalInterrupt,
            _ => Trap::MachineTimerInterrupt,
        }
    }
}

//! RISC-V Privilege Modes.
//!
//! This module defines the privilege levels supported by the processor:
//! User (U), Supervisor (S), and Machine (M). It provides utilities for
//! converting between numeric representations and the `PrivilegeMode` enum.

/// RISC-V privilege mode levels.
///
/// RISC-V defines three privilege modes that control access to system
/// resources and instructions. Machine mode is the highest privilege
/// level and is required for all implementations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrivilegeMode {
    /// User mode (U-mode).
    ///
    /// Lowest privilege level for application code. Limited access to
    /// system resources and CSRs.
    User = 0,

    /// Supervisor mode (S-mode).
    ///
    /// Intermediate privilege level for operating system kernel code.
    /// Provides access to supervisor-level CSRs and memory management.
    Supervisor = 1,

    /// Machine mode (M-mode).
    ///
    /// Highest privilege level for firmware and hypervisor code.
    /// Full access to all system resources and CSRs.
    Machine = 3,
}

impl PrivilegeMode {
    /// Converts a u8 value to a privilege mode.
    ///
    /// # Arguments
    ///
    /// * `val` - The numeric privilege mode value (0, 1, or 3)
    ///
    /// # Returns
    ///
    /// The corresponding `PrivilegeMode`, defaulting to `Machine` for invalid values.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => PrivilegeMode::User,
            1 => PrivilegeMode::Supervisor,
            3 => PrivilegeMode::Machine,
            _ => PrivilegeMode::Machine,
        }
    }

    /// Converts a privilege mode to its u8 representation.
    ///
    /// # Returns
    ///
    /// The numeric value of the privilege mode (0, 1, or 3).
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Returns the human-readable name of the privilege mode.
    ///
    /// # Returns
    ///
    /// A static string slice containing the mode name.
    pub fn name(&self) -> &'static str {
        match self {
            PrivilegeMode::User => "User",
            PrivilegeMode::Supervisor => "Supervisor",
            PrivilegeMode::Machine => "Machine",
        }
    }
}

impl std::fmt::Display for PrivilegeMode {
    /// Formats the privilege mode for display.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to
    ///
    /// # Returns
    ///
    /// A formatting result indicating success or failure.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

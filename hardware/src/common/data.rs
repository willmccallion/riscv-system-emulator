//! Memory Access Types.
//!
//! This module defines the classification of memory accesses used throughout
//! the simulator. These types are used by the Memory Management Unit (MMU)
//! and Physical Memory Protection (PMP) logic to validate permissions
//! and handle page faults correctly.

/// Type of memory access operation.
///
/// Used to distinguish between instruction fetches, data reads,
/// and data writes for proper memory access handling and permission
/// checking in the memory management unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessType {
    /// Instruction fetch access.
    ///
    /// Used when fetching instructions from memory for execution.
    /// Requires Execute (X) permission in the page table.
    Fetch,

    /// Data read access.
    ///
    /// Used when loading data from memory into registers.
    /// Requires Read (R) permission in the page table.
    Read,

    /// Data write access.
    ///
    /// Used when storing data from registers to memory.
    /// Requires Write (W) permission in the page table.
    Write,
}

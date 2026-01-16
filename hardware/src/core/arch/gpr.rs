//! RISC-V General-Purpose Register File.
//!
//! This module implements the General-Purpose Register (GPR) file, containing
//! 32 registers (x0-x31). It enforces the architectural invariant that
//! register x0 is always hardwired to zero.

/// General-Purpose Register file.
///
/// Contains 32 general-purpose registers (x0-x31) used for integer
/// operations. Register x0 is hardwired to zero and cannot be modified.
pub struct Gpr {
    regs: [u64; 32],
}

impl Gpr {
    /// Creates a new general-purpose register file with all registers initialized to zero.
    ///
    /// # Returns
    ///
    /// A new `Gpr` instance with all 32 registers set to 0.
    pub fn new() -> Self {
        Self { regs: [0; 32] }
    }

    /// Reads a general-purpose register value.
    ///
    /// # Arguments
    ///
    /// * `idx` - Register index (0-31)
    ///
    /// # Returns
    ///
    /// The 64-bit value stored in the specified register.
    /// Register x0 (index 0) always returns 0 regardless of storage.
    pub fn read(&self, idx: usize) -> u64 {
        if idx == 0 {
            0
        } else {
            self.regs[idx]
        }
    }

    /// Writes a value to a general-purpose register.
    ///
    /// # Arguments
    ///
    /// * `idx` - Register index (0-31)
    /// * `val` - The 64-bit value to write
    ///
    /// # Note
    ///
    /// Writes to register x0 (index 0) are silently ignored as x0 is hardwired to zero.
    pub fn write(&mut self, idx: usize, val: u64) {
        if idx != 0 {
            self.regs[idx] = val;
        }
    }

    /// Dumps the contents of all general-purpose registers to stdout.
    ///
    /// Displays registers in pairs (two per line) with hexadecimal formatting.
    /// Useful for debugging and tracing register state during simulation.
    pub fn dump(&self) {
        for i in (0..32).step_by(2) {
            println!(
                "x{:<2}={:#018x} x{:<2}={:#018x}",
                i,
                self.regs[i],
                i + 1,
                self.regs[i + 1]
            );
        }
    }
}

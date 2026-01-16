//! Memory Timing Controller.
//!
//! This module defines the `MemoryController` trait and implementations for
//! simulating memory access latencies. It supports both simple fixed-latency
//! models and more complex DRAM timing models that account for row buffer
//! locality.

/// Trait for memory controller implementations.
pub trait MemoryController {
    /// Calculates the latency for a memory access at a specific address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The physical address being accessed.
    ///
    /// # Returns
    ///
    /// The latency in CPU cycles.
    fn access_latency(&mut self, addr: u64) -> u64;
}

/// A simple memory controller with fixed latency.
///
/// Models an ideal memory system where every access takes a constant amount of time,
/// ignoring row buffer locality or refresh cycles.
pub struct SimpleController {
    /// Fixed latency per access.
    latency: u64,
}

impl SimpleController {
    /// Creates a new SimpleController.
    ///
    /// # Arguments
    ///
    /// * `latency` - The fixed latency in cycles.
    pub fn new(latency: u64) -> Self {
        Self { latency }
    }
}

impl MemoryController for SimpleController {
    /// Returns the fixed latency regardless of the address.
    fn access_latency(&mut self, _addr: u64) -> u64 {
        self.latency
    }
}

/// A DRAM-aware memory controller.
///
/// Simulates basic DRAM timing parameters including Row Access Strobe (RAS),
/// Column Access Strobe (CAS), and Precharge (PRE). It tracks the currently
/// open row to simulate row buffer hits (lower latency) and misses (higher latency).
pub struct DramController {
    /// The index of the currently open row, if any.
    last_row: Option<u64>,
    /// Column Access Strobe latency (Column command to data).
    t_cas: u64,
    /// Row Access Strobe latency (Row Active command to Column command).
    t_ras: u64,
    /// Precharge latency (Precharge command to Row Active command).
    t_pre: u64,
    /// Bitmask used to extract the row index from a physical address.
    row_mask: u64,
}

impl DramController {
    /// Creates a new DramController.
    ///
    /// # Arguments
    ///
    /// * `t_cas` - CAS latency.
    /// * `t_ras` - RAS latency.
    /// * `t_pre` - Precharge latency.
    pub fn new(t_cas: u64, t_ras: u64, t_pre: u64) -> Self {
        Self {
            last_row: None,
            t_cas,
            t_ras,
            t_pre,
            row_mask: !2047,
        }
    }
}

impl MemoryController for DramController {
    /// Calculates latency based on row buffer state.
    ///
    /// * **Row Hit:** If the requested row is already open, latency is just `t_cas`.
    /// * **Row Miss (Open):** If a different row is open, it must be precharged first: `t_pre + t_ras + t_cas`.
    /// * **Row Miss (Closed):** If no row is open, the row must be activated: `t_ras + t_cas`.
    fn access_latency(&mut self, addr: u64) -> u64 {
        let row = addr & self.row_mask;

        match self.last_row {
            Some(open_row) if open_row == row => self.t_cas,
            Some(_) => {
                self.last_row = Some(row);
                self.t_pre + self.t_ras + self.t_cas
            }
            None => {
                self.last_row = Some(row);
                self.t_ras + self.t_cas
            }
        }
    }
}

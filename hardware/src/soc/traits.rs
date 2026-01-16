//! System-on-Chip Traits.
//!
//! This module defines the common interfaces that must be implemented by
//! hardware components in the SoC, such as memory-mapped devices. It allows
//! the system bus to interact with disparate devices uniformly.

use crate::soc::devices::{Plic, Uart};
use crate::soc::memory::Memory;

/// Trait for memory-mapped I/O devices.
///
/// All devices attached to the system bus must implement this trait to
/// handle read and write operations at specific offsets. It also provides
/// hooks for clock ticking and interrupt management.
pub trait Device {
    /// Returns the user-friendly name of the device.
    ///
    /// Used for debugging and logging purposes.
    fn name(&self) -> &str;

    /// Returns the address range (Base Address, Size) of the device.
    ///
    /// Used by the system bus to route memory accesses to the correct device.
    fn address_range(&self) -> (u64, u64);

    /// Reads a byte from the device at the specified offset.
    fn read_u8(&mut self, offset: u64) -> u8;

    /// Reads a half-word (16-bit) from the device at the specified offset.
    fn read_u16(&mut self, offset: u64) -> u16;

    /// Reads a word (32-bit) from the device at the specified offset.
    fn read_u32(&mut self, offset: u64) -> u32;

    /// Reads a double-word (64-bit) from the device at the specified offset.
    fn read_u64(&mut self, offset: u64) -> u64;

    /// Writes a byte to the device at the specified offset.
    fn write_u8(&mut self, offset: u64, val: u8);

    /// Writes a half-word (16-bit) to the device at the specified offset.
    fn write_u16(&mut self, offset: u64, val: u16);

    /// Writes a word (32-bit) to the device at the specified offset.
    fn write_u32(&mut self, offset: u64, val: u32);

    /// Writes a double-word (64-bit) to the device at the specified offset.
    fn write_u64(&mut self, offset: u64, val: u64);

    /// Writes a slice of bytes to the device starting at the specified offset.
    ///
    /// Default implementation iterates and writes bytes individually.
    /// Devices may override this for optimized block writes (e.g., DMA).
    fn write_bytes(&mut self, offset: u64, data: &[u8]) {
        for (i, byte) in data.iter().enumerate() {
            self.write_u8(offset + i as u64, *byte);
        }
    }

    /// Advances the device state by one clock cycle.
    ///
    /// # Returns
    ///
    /// `true` if the device has raised an interrupt, `false` otherwise.
    fn tick(&mut self) -> bool {
        false
    }

    /// Returns the Interrupt Request (IRQ) ID associated with this device.
    ///
    /// Returns `None` if the device does not generate interrupts.
    fn get_irq_id(&self) -> Option<u32> {
        None
    }

    /// Downcasts the device to a mutable PLIC reference if applicable.
    ///
    /// Used by the system bus to route interrupt updates to the PLIC.
    fn as_plic_mut(&mut self) -> Option<&mut Plic> {
        None
    }

    /// Downcasts the device to a mutable UART reference if applicable.
    ///
    /// Used by the system bus to check for kernel panic strings in UART output.
    fn as_uart_mut(&mut self) -> Option<&mut Uart> {
        None
    }

    /// Downcasts the device to a mutable Memory reference if applicable.
    ///
    /// Used by the system bus to extract raw pointers for fast-path access.
    fn as_memory_mut(&mut self) -> Option<&mut Memory> {
        None
    }
}

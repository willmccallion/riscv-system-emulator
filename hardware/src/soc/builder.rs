//! System-on-Chip (SoC) Builder.
//!
//! This module defines the `System` structure, which acts as the container
//! for all hardware components (Bus, Memory, Devices). It handles the
//! initialization and wiring of the system based on the provided configuration.

use crate::config::{Config, MemoryController as MemControllerType};
use crate::soc::devices::{Clint, GoldfishRtc, Plic, SysCon, Uart, VirtioBlock};
use crate::soc::interconnect::Bus;
use crate::soc::memory::buffer::DramBuffer;
use crate::soc::memory::controller::{DramController, MemoryController, SimpleController};
use crate::soc::memory::Memory;
use std::fs;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

/// System-on-Chip (SoC) structure containing all system components.
///
/// Represents the complete system including memory, devices, interconnect,
/// and memory controller. This is the top-level system structure that
/// the CPU interacts with.
pub struct System {
    /// System interconnect bus for device communication.
    pub bus: Bus,
    /// Memory controller for main memory access timing.
    pub mem_controller: Box<dyn MemoryController>,
    /// Atomic exit request flag for system shutdown.
    pub exit_request: Arc<AtomicU64>,
}

impl System {
    /// Creates a new system instance with the specified configuration.
    ///
    /// Initializes all system components including memory, devices (UART,
    /// disk, timers, interrupt controllers), and the system bus according
    /// to the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - System configuration specifying memory map and device parameters
    /// * `disk_path` - Path to disk image file (empty string if no disk)
    ///
    /// # Returns
    ///
    /// A new `System` instance ready for simulation.
    pub fn new(config: &Config, disk_path: &str) -> Self {
        let mut bus = Bus::new(config.system.bus_width, config.system.bus_latency);
        let exit_request = Arc::new(AtomicU64::new(u64::MAX));

        let ram_base = config.system.ram_base;
        let ram_size = config.memory.ram_size;

        let ram_buffer = Arc::new(DramBuffer::new(ram_size));

        let mem = Memory::new(ram_buffer.clone(), ram_base);

        let uart_base = config.system.uart_base;
        let uart = Uart::new(uart_base);

        let clint_addr = config.system.clint_base;
        let clint = Clint::new(clint_addr, config.system.clint_divider);

        let plic_addr = 0x0c00_0000;
        let plic = Plic::new(plic_addr);

        let disk_base = config.system.disk_base;
        let mut disk = VirtioBlock::new(disk_base, ram_base, ram_buffer);
        if !disk_path.is_empty() {
            if let Ok(disk_data) = fs::read(disk_path) {
                if !disk_data.is_empty() {
                    disk.load(disk_data);
                }
            }
        }

        let syscon_addr = config.system.syscon_base;
        let syscon = SysCon::new(syscon_addr, exit_request.clone());

        let rtc = GoldfishRtc::new(0x101000);

        bus.add_device(Box::new(mem));
        bus.add_device(Box::new(uart));
        bus.add_device(Box::new(disk));
        bus.add_device(Box::new(clint));
        bus.add_device(Box::new(plic));
        bus.add_device(Box::new(syscon));
        bus.add_device(Box::new(rtc));

        let mem_controller: Box<dyn MemoryController> = match config.memory.controller {
            MemControllerType::Dram => Box::new(DramController::new(
                config.memory.t_cas,
                config.memory.t_ras,
                config.memory.t_pre,
            )),
            MemControllerType::Simple => {
                Box::new(SimpleController::new(config.memory.row_miss_latency))
            }
        };

        Self {
            bus,
            mem_controller,
            exit_request,
        }
    }

    /// Loads a binary blob into memory at a specific address.
    ///
    /// # Arguments
    ///
    /// * `data` - The binary data to load
    /// * `addr` - The physical address to load the data at
    pub fn load_binary_at(&mut self, data: &[u8], addr: u64) {
        self.bus.load_binary_at(data, addr);
    }

    /// Advances the system state by one cycle.
    ///
    /// Ticks all attached devices and the bus.
    ///
    /// # Returns
    ///
    /// A tuple `(timer_irq, meip, seip)` indicating active interrupt lines.
    pub fn tick(&mut self) -> (bool, bool, bool) {
        self.bus.tick()
    }

    /// Checks if a system exit has been requested via SysCon.
    ///
    /// # Returns
    ///
    /// `Some(code)` if an exit was requested, `None` otherwise.
    pub fn check_exit(&self) -> Option<u64> {
        let val = self.exit_request.load(std::sync::atomic::Ordering::Relaxed);
        if val != u64::MAX {
            Some(val)
        } else {
            None
        }
    }

    /// Checks if a kernel panic has been detected via UART output.
    ///
    /// # Returns
    ///
    /// `true` if a panic string was detected, `false` otherwise.
    pub fn check_kernel_panic(&mut self) -> bool {
        self.bus.check_kernel_panic()
    }
}

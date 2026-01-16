//! System Bus Interconnect.
//!
//! This module implements the system bus, which routes memory accesses
//! to the appropriate devices based on the physical address map. It manages
//! a list of devices and handles address decoding and latency simulation.

use super::devices::Device;

/// System interconnect bus for device communication.
///
/// Manages memory-mapped I/O access to all system devices including
/// RAM, UART, disk, timers, and other peripherals. Routes memory
/// accesses to the appropriate device based on address ranges.
pub struct Bus {
    devices: Vec<Box<dyn Device>>,
    /// Bus width in bytes (determines transfer size).
    pub width_bytes: u64,
    /// Bus latency in cycles (base latency per access).
    pub latency_cycles: u64,

    last_device_idx: usize,

    ram_idx: Option<usize>,

    uart_idx: Option<usize>,
}

impl Bus {
    /// Creates a new bus instance with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `width_bytes` - Bus width in bytes (typically 8 for 64-bit systems)
    /// * `latency_cycles` - Base latency in cycles for bus transactions
    ///
    /// # Returns
    ///
    /// A new `Bus` instance with no devices attached.
    pub fn new(width_bytes: u64, latency_cycles: u64) -> Self {
        Self {
            devices: Vec::new(),
            width_bytes,
            latency_cycles,
            last_device_idx: 0,
            ram_idx: None,
            uart_idx: None,
        }
    }

    /// Adds a device to the bus.
    ///
    /// Registers a memory-mapped device on the bus. Devices are
    /// automatically sorted by base address for efficient lookup.
    ///
    /// # Arguments
    ///
    /// * `dev` - The device to add to the bus
    pub fn add_device(&mut self, dev: Box<dyn Device>) {
        let (base, size) = dev.address_range();
        println!(
            "[Bus] Registered device: {:<12} @ {:#010x} - {:#010x} ({} bytes)",
            dev.name(),
            base,
            base + size,
            size
        );
        self.devices.push(dev);

        self.devices.sort_by_key(|d| d.address_range().0);

        self.ram_idx = self.devices.iter().position(|d| d.name() == "DRAM");
        self.uart_idx = self.devices.iter().position(|d| d.name() == "UART0");
        self.last_device_idx = 0;
    }

    /// Calculates the bus transit time for a transfer of the specified size.
    ///
    /// Computes the number of cycles required to transfer data over the bus,
    /// accounting for bus width and base latency.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Number of bytes to transfer
    ///
    /// # Returns
    ///
    /// The number of cycles required for the transfer.
    pub fn calculate_transit_time(&self, bytes: usize) -> u64 {
        let transfers = (bytes as u64 + self.width_bytes - 1) / self.width_bytes;
        self.latency_cycles + transfers
    }

    /// Loads a binary blob into memory at a specific address.
    ///
    /// Writes data byte-by-byte to the device mapped at the target address.
    ///
    /// # Arguments
    ///
    /// * `data` - The binary data to load
    /// * `addr` - The physical address to load the data at
    pub fn load_binary_at(&mut self, data: &[u8], addr: u64) {
        println!("[Loader] Writing {} bytes to {:#x}", data.len(), addr);
        if let Some((dev, offset)) = self.find_device(addr) {
            let (_, size) = dev.address_range();
            if offset + (data.len() as u64) <= size {
                dev.write_bytes(offset, data);
                return;
            }
        }
        for (i, byte) in data.iter().enumerate() {
            self.write_u8(addr + i as u64, *byte);
        }
    }

    /// Checks if a physical address maps to a valid device.
    ///
    /// # Arguments
    ///
    /// * `paddr` - The physical address to check
    ///
    /// # Returns
    ///
    /// `true` if the address is mapped to a device, `false` otherwise.
    pub fn is_valid_address(&self, paddr: u64) -> bool {
        if let Some(idx) = self.ram_idx {
            let (start, size) = self.devices[idx].address_range();
            if paddr >= start && paddr < start + size {
                return true;
            }
        }

        for dev in &self.devices {
            let (start, size) = dev.address_range();
            if paddr >= start && paddr < start + size {
                return true;
            }
        }
        false
    }

    /// Advances the state of all devices on the bus by one cycle.
    ///
    /// # Returns
    ///
    /// A tuple `(timer_irq, meip, seip)` indicating active interrupt lines.
    pub fn tick(&mut self) -> (bool, bool, bool) {
        let mut timer_irq = false;
        let mut active_irqs = 0u64;

        for i in 0..self.devices.len() {
            let dev = &mut self.devices[i];
            if dev.tick() {
                if let Some(id) = dev.get_irq_id() {
                    if id < 64 {
                        active_irqs |= 1 << id;
                    }
                }
                if dev.name() == "CLINT" {
                    timer_irq = true;
                }
            }
        }

        let (meip, seip) = if let Some(plic) = self.find_plic() {
            plic.update_irqs(active_irqs);
            plic.check_interrupts()
        } else {
            (false, false)
        };

        (timer_irq, meip, seip)
    }

    /// Checks if a kernel panic has been detected by the UART device.
    ///
    /// # Returns
    ///
    /// `true` if a panic string was detected, `false` otherwise.
    pub fn check_kernel_panic(&mut self) -> bool {
        if let Some(idx) = self.uart_idx {
            if idx < self.devices.len() {
                if let Some(uart) = self.devices[idx].as_uart_mut() {
                    return uart.check_kernel_panic();
                }
            }
        }
        false
    }

    /// Retrieves the raw pointer and range of the main RAM memory if present.
    ///
    /// This allows the CPU to bypass the bus for high-frequency RAM accesses
    /// by performing direct pointer arithmetic.
    pub fn get_ram_info(&mut self) -> Option<(*mut u8, u64, u64)> {
        if let Some(idx) = self.ram_idx {
            if let Some(mem) = self.devices[idx].as_memory_mut() {
                let (base, size) = mem.address_range();
                return Some((mem.as_mut_ptr(), base, base + size));
            }
        }
        None
    }

    /// Helper to find the PLIC device in the device list.
    fn find_plic(&mut self) -> Option<&mut crate::soc::devices::Plic> {
        for dev in &mut self.devices {
            if let Some(plic) = dev.as_plic_mut() {
                return Some(plic);
            }
        }
        None
    }

    /// Helper to find the device mapped to a specific physical address.
    ///
    /// Returns a mutable reference to the device and the offset within that device.
    #[inline(always)]
    fn find_device(&mut self, paddr: u64) -> Option<(&mut Box<dyn Device>, u64)> {
        if self.last_device_idx < self.devices.len() {
            let (start, size) = self.devices[self.last_device_idx].address_range();
            if paddr >= start && paddr < start + size {
                return Some((&mut self.devices[self.last_device_idx], paddr - start));
            }
        }

        if let Some(idx) = self.ram_idx {
            let (start, size) = self.devices[idx].address_range();
            if paddr >= start && paddr < start + size {
                self.last_device_idx = idx;
                return Some((&mut self.devices[idx], paddr - start));
            }
        }

        for (i, dev) in self.devices.iter_mut().enumerate() {
            let (start, size) = dev.address_range();
            if paddr >= start && paddr < start + size {
                self.last_device_idx = i;
                return Some((dev, paddr - start));
            }
        }
        None
    }

    /// Reads a byte from the specified physical address.
    #[inline(always)]
    pub fn read_u8(&mut self, paddr: u64) -> u8 {
        if let Some((dev, offset)) = self.find_device(paddr) {
            dev.read_u8(offset)
        } else {
            0
        }
    }

    /// Reads a half-word (16-bit) from the specified physical address.
    #[inline(always)]
    pub fn read_u16(&mut self, paddr: u64) -> u16 {
        if let Some((dev, offset)) = self.find_device(paddr) {
            dev.read_u16(offset)
        } else {
            0
        }
    }

    /// Reads a word (32-bit) from the specified physical address.
    #[inline(always)]
    pub fn read_u32(&mut self, paddr: u64) -> u32 {
        if let Some((dev, offset)) = self.find_device(paddr) {
            dev.read_u32(offset)
        } else {
            0
        }
    }

    /// Reads a double-word (64-bit) from the specified physical address.
    #[inline(always)]
    pub fn read_u64(&mut self, paddr: u64) -> u64 {
        if let Some((dev, offset)) = self.find_device(paddr) {
            dev.read_u64(offset)
        } else {
            0
        }
    }

    /// Writes a byte to the specified physical address.
    #[inline(always)]
    pub fn write_u8(&mut self, paddr: u64, val: u8) {
        if let Some((dev, offset)) = self.find_device(paddr) {
            dev.write_u8(offset, val);
        }
    }

    /// Writes a half-word (16-bit) to the specified physical address.
    #[inline(always)]
    pub fn write_u16(&mut self, paddr: u64, val: u16) {
        if let Some((dev, offset)) = self.find_device(paddr) {
            dev.write_u16(offset, val);
        }
    }

    /// Writes a word (32-bit) to the specified physical address.
    #[inline(always)]
    pub fn write_u32(&mut self, paddr: u64, val: u32) {
        if let Some((dev, offset)) = self.find_device(paddr) {
            dev.write_u32(offset, val);
        }
    }

    /// Writes a double-word (64-bit) to the specified physical address.
    #[inline(always)]
    pub fn write_u64(&mut self, paddr: u64, val: u64) {
        if let Some((dev, offset)) = self.find_device(paddr) {
            dev.write_u64(offset, val);
        }
    }
}

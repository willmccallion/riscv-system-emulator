//! Load/Store Unit (LSU) Helpers.
//!
//! This module provides helper functions for the Load/Store Unit, specifically
//! for handling Atomic Memory Operations (AMOs). It implements the read-modify-write
//! logic required for the RISC-V 'A' extension.

use crate::core::pipeline::signals::{AtomicOp, MemWidth};

/// Load/Store Unit (LSU) for atomic memory operations.
///
/// Implements atomic read-modify-write operations for the RISC-V
/// atomic extension (A extension), including swap, arithmetic, and
/// min/max operations.
pub struct Lsu;

impl Lsu {
    /// Performs an atomic ALU operation for atomic memory instructions.
    ///
    /// Computes the result of an atomic operation that combines a value
    /// from memory with a value from a register. Used by AMO instructions
    /// (AMOSWAP, AMOADD, AMOXOR, etc.) to compute the new value to be
    /// written back to memory.
    ///
    /// # Arguments
    ///
    /// * `op` - The atomic operation type
    /// * `mem_val` - The current value read from memory
    /// * `reg_val` - The value from the source register
    /// * `width` - The width of the operation (Word or Double)
    ///
    /// # Returns
    ///
    /// The computed result that will be written back to memory.
    /// For 32-bit operations, the result is sign-extended to 64 bits.
    pub fn atomic_alu(op: AtomicOp, mem_val: u64, reg_val: u64, width: MemWidth) -> u64 {
        if matches!(width, MemWidth::Word) {
            let a = mem_val as i32;
            let b = reg_val as i32;
            let res = match op {
                AtomicOp::Swap => b,
                AtomicOp::Add => a.wrapping_add(b),
                AtomicOp::Xor => a ^ b,
                AtomicOp::And => a & b,
                AtomicOp::Or => a | b,
                AtomicOp::Min => a.min(b),
                AtomicOp::Max => a.max(b),
                AtomicOp::Minu => (mem_val as u32).min(reg_val as u32) as i32,
                AtomicOp::Maxu => (mem_val as u32).max(reg_val as u32) as i32,
                _ => 0,
            };
            res as i64 as u64
        } else {
            let a = mem_val as i64;
            let b = reg_val as i64;
            let res = match op {
                AtomicOp::Swap => b,
                AtomicOp::Add => a.wrapping_add(b),
                AtomicOp::Xor => a ^ b,
                AtomicOp::And => a & b,
                AtomicOp::Or => a | b,
                AtomicOp::Min => a.min(b),
                AtomicOp::Max => a.max(b),
                AtomicOp::Minu => (mem_val).min(reg_val) as i64,
                AtomicOp::Maxu => (mem_val).max(reg_val) as i64,
                _ => 0,
            };
            res as u64
        }
    }
}

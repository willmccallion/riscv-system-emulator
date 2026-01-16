//! Arithmetic Logic Unit (ALU).
//!
//! This module implements the integer ALU used in the Execute stage.
//! It handles standard arithmetic, logical operations, and shifts
//! for both 32-bit and 64-bit operands. It also implements the
//! Multiply/Divide (M) extension operations.

use crate::core::pipeline::signals::AluOp;

/// Arithmetic Logic Unit (ALU) for integer operations.
///
/// Implements all RISC-V integer arithmetic and logical operations
/// including addition, subtraction, shifts, comparisons, and
/// multiply/divide operations from the I and M extensions.
pub struct Alu;

impl Alu {
    /// Executes an integer ALU operation.
    ///
    /// Performs the specified ALU operation on operands `a` and `b`,
    /// supporting both 32-bit and 64-bit operations based on the
    /// `is32` flag. The `_c` parameter is reserved for future use
    /// (e.g., three-operand operations).
    ///
    /// # Arguments
    ///
    /// * `op` - The ALU operation to perform
    /// * `a` - First operand (64-bit value)
    /// * `b` - Second operand (64-bit value, also used as shift amount)
    /// * `_c` - Third operand (currently unused)
    /// * `is32` - If true, perform 32-bit operation (RV32 mode)
    ///
    /// # Returns
    ///
    /// The 64-bit result of the ALU operation. For 32-bit operations,
    /// the result is sign-extended to 64 bits.
    pub fn execute(op: AluOp, a: u64, b: u64, _c: u64, is32: bool) -> u64 {
        /// Bit mask for shift amount in RV64 (6 bits: 0-63).
        const SHAMT_MASK_RV64: u64 = 0x3f;

        /// Bit mask for shift amount in RV32 (5 bits: 0-31).
        const SHAMT_MASK_RV32: u32 = 0x1f;

        /// Number of bits in a 32-bit word.
        const WORD_BITS: u32 = 32;

        /// Number of bits in XLEN (64 for RV64).
        const XLEN_BITS: u32 = 64;

        let sh6 = (b & SHAMT_MASK_RV64) as u32;
        match op {
            AluOp::Add => {
                if is32 {
                    (a as i32).wrapping_add(b as i32) as i64 as u64
                } else {
                    a.wrapping_add(b)
                }
            }
            AluOp::Sub => {
                if is32 {
                    (a as i32).wrapping_sub(b as i32) as i64 as u64
                } else {
                    a.wrapping_sub(b)
                }
            }
            AluOp::Sll => {
                if is32 {
                    (a as i32).wrapping_shl(b as u32 & SHAMT_MASK_RV32) as i64 as u64
                } else {
                    a.wrapping_shl(sh6)
                }
            }
            AluOp::Srl => {
                if is32 {
                    ((a as u32).wrapping_shr(b as u32 & SHAMT_MASK_RV32)) as i32 as i64 as u64
                } else {
                    a.wrapping_shr(sh6)
                }
            }
            AluOp::Sra => {
                if is32 {
                    ((a as i32) >> (b as u32 & SHAMT_MASK_RV32)) as i64 as u64
                } else {
                    ((a as i64) >> sh6) as u64
                }
            }
            AluOp::Or => a | b,
            AluOp::And => a & b,
            AluOp::Xor => a ^ b,
            AluOp::Slt => {
                if is32 {
                    ((a as i32) < (b as i32)) as u64
                } else {
                    ((a as i64) < (b as i64)) as u64
                }
            }
            AluOp::Sltu => {
                if is32 {
                    ((a as u32) < (b as u32)) as u64
                } else {
                    (a < b) as u64
                }
            }
            AluOp::Mul => {
                if is32 {
                    (a as i32).wrapping_mul(b as i32) as i64 as u64
                } else {
                    a.wrapping_mul(b)
                }
            }
            AluOp::Mulh => {
                if is32 {
                    ((a as i32 as i64 * b as i32 as i64) >> WORD_BITS) as u64
                } else {
                    (((a as i128) * (b as i128)) >> XLEN_BITS) as u64
                }
            }
            AluOp::Mulhsu => {
                if is32 {
                    ((a as i32 as i64 * (b as u32) as i64) >> WORD_BITS) as u64
                } else {
                    (((a as i128) * (b as u128 as i128)) >> XLEN_BITS) as u64
                }
            }
            AluOp::Mulhu => {
                if is32 {
                    (((a as u32) as u64 * (b as u32) as u64) >> WORD_BITS) as i64 as u64
                } else {
                    (((a as u128) * (b as u128)) >> XLEN_BITS) as u64
                }
            }
            AluOp::Div => {
                if is32 {
                    if (b as i32) == 0 {
                        -1i64 as u64
                    } else {
                        (a as i32).wrapping_div(b as i32) as i64 as u64
                    }
                } else if b == 0 {
                    -1i64 as u64
                } else {
                    (a as i64).wrapping_div(b as i64) as u64
                }
            }
            AluOp::Divu => {
                if is32 {
                    if (b as i32) == 0 {
                        -1i64 as u64
                    } else {
                        ((a as u32) / (b as u32)) as i64 as u64
                    }
                } else if b == 0 {
                    -1i64 as u64
                } else {
                    a / b
                }
            }
            AluOp::Rem => {
                if is32 {
                    if (b as i32) == 0 {
                        a
                    } else {
                        (a as i32).wrapping_rem(b as i32) as i64 as u64
                    }
                } else if b == 0 {
                    a
                } else {
                    (a as i64).wrapping_rem(b as i64) as u64
                }
            }
            AluOp::Remu => {
                if is32 {
                    if (b as i32) == 0 {
                        a
                    } else {
                        ((a as u32) % (b as u32)) as i64 as u64
                    }
                } else if b == 0 {
                    a
                } else {
                    a % b
                }
            }
            _ => 0,
        }
    }
}

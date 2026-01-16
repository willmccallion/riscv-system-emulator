//! Floating-Point Unit (FPU).
//!
//! This module implements the floating-point arithmetic unit used in the
//! Execute stage. It handles single-precision (F) and double-precision (D)
//! floating-point operations, including fused multiply-add, comparisons,
//! and conversions between integer and floating-point formats.

use crate::core::pipeline::signals::AluOp;

/// Floating-Point Unit (FPU) for floating-point operations.
///
/// Implements all RISC-V floating-point operations including arithmetic,
/// comparisons, conversions, and fused multiply-add operations from
/// the F (single-precision) and D (double-precision) extensions.
pub struct Fpu;

impl Fpu {
    /// Converts a 32-bit float to 64-bit format with sign extension.
    ///
    /// Used for single-precision floating-point operations in RV64,
    /// where 32-bit floats are stored in 64-bit registers with the
    /// upper 32 bits sign-extended.
    ///
    /// # Arguments
    ///
    /// * `f` - The 32-bit floating-point value
    ///
    /// # Returns
    ///
    /// The 64-bit representation with sign extension in the upper bits.
    pub fn box_f32(f: f32) -> u64 {
        /// Bit mask to sign-extend 32-bit float to 64-bit (sets upper 32 bits to 1s if sign bit is set).
        const F32_SIGN_EXTEND_MASK: u64 = 0xFFFF_FFFF_0000_0000;
        (f.to_bits() as u64) | F32_SIGN_EXTEND_MASK
    }

    /// Executes a floating-point operation.
    ///
    /// Performs the specified floating-point operation on operands `a`, `b`,
    /// and optionally `c` (for fused multiply-add operations). Supports
    /// both single-precision (32-bit) and double-precision (64-bit) operations
    /// based on the `is32` flag.
    ///
    /// # Arguments
    ///
    /// * `op` - The floating-point operation to perform
    /// * `a` - First operand (64-bit IEEE 754 representation)
    /// * `b` - Second operand (64-bit IEEE 754 representation)
    /// * `c` - Third operand for FMA operations (64-bit IEEE 754 representation)
    /// * `is32` - If true, perform single-precision operation (32-bit)
    ///
    /// # Returns
    ///
    /// The 64-bit result of the floating-point operation. For single-precision
    /// operations, the result is sign-extended to 64 bits.
    pub fn execute(op: AluOp, a: u64, b: u64, c: u64, is32: bool) -> u64 {
        if is32 {
            let fa = f32::from_bits(a as u32);
            let fb = f32::from_bits(b as u32);
            let fc = f32::from_bits(c as u32);
            match op {
                AluOp::FAdd => Self::box_f32(fa + fb),
                AluOp::FSub => Self::box_f32(fa - fb),
                AluOp::FMul => Self::box_f32(fa * fb),
                AluOp::FDiv => Self::box_f32(fa / fb),
                AluOp::FSqrt => Self::box_f32(fa.sqrt()),
                AluOp::FMin => Self::box_f32(fa.min(fb)),
                AluOp::FMax => Self::box_f32(fa.max(fb)),
                AluOp::FMAdd => Self::box_f32(fa.mul_add(fb, fc)),
                AluOp::FMSub => Self::box_f32(fa.mul_add(fb, -fc)),
                AluOp::FNMAdd => Self::box_f32((-fa).mul_add(fb, -fc)),
                AluOp::FNMSub => Self::box_f32((-fa).mul_add(fb, fc)),
                AluOp::FSgnJ => {
                    /// Bit mask for the sign bit in a 32-bit IEEE 754 float (bit 31).
                    const F32_SIGN_BIT_MASK: u32 = 0x8000_0000;
                    Self::box_f32(f32::from_bits(
                        (fa.to_bits() & !F32_SIGN_BIT_MASK) | (fb.to_bits() & F32_SIGN_BIT_MASK),
                    ))
                }
                AluOp::FSgnJN => {
                    /// Bit mask for the sign bit in a 32-bit IEEE 754 float (bit 31).
                    const F32_SIGN_BIT_MASK: u32 = 0x8000_0000;
                    Self::box_f32(f32::from_bits(
                        (fa.to_bits() & !F32_SIGN_BIT_MASK) | (!fb.to_bits() & F32_SIGN_BIT_MASK),
                    ))
                }
                AluOp::FSgnJX => {
                    /// Bit mask for the sign bit in a 32-bit IEEE 754 float (bit 31).
                    const F32_SIGN_BIT_MASK: u32 = 0x8000_0000;
                    Self::box_f32(f32::from_bits(
                        fa.to_bits() ^ (fb.to_bits() & F32_SIGN_BIT_MASK),
                    ))
                }
                AluOp::FEq => (fa == fb) as u64,
                AluOp::FLt => (fa < fb) as u64,
                AluOp::FLe => (fa <= fb) as u64,
                AluOp::FCvtWS => (fa as i32) as i64 as u64,
                AluOp::FCvtLS => (fa as i64) as u64,
                AluOp::FCvtSD => Self::box_f32(fa as f32),
                AluOp::FCvtSW => ((a as i32) as f64).to_bits(),
                AluOp::FCvtSL => ((a as i64) as f64).to_bits(),
                AluOp::FCvtDS => (f32::from_bits(a as u32) as f64).to_bits(),
                AluOp::FMvToF => Self::box_f32(f32::from_bits(a as u32)),
                AluOp::FMvToX => (a as i32) as u64,
                _ => 0,
            }
        } else {
            let fa = f64::from_bits(a);
            let fb = f64::from_bits(b);
            let fc = f64::from_bits(c);
            match op {
                AluOp::FAdd => (fa + fb).to_bits(),
                AluOp::FSub => (fa - fb).to_bits(),
                AluOp::FMul => (fa * fb).to_bits(),
                AluOp::FDiv => (fa / fb).to_bits(),
                AluOp::FSqrt => fa.sqrt().to_bits(),
                AluOp::FMin => fa.min(fb).to_bits(),
                AluOp::FMax => fa.max(fb).to_bits(),
                AluOp::FMAdd => fa.mul_add(fb, fc).to_bits(),
                AluOp::FMSub => fa.mul_add(fb, -fc).to_bits(),
                AluOp::FNMAdd => (-fa).mul_add(fb, -fc).to_bits(),
                AluOp::FNMSub => (-fa).mul_add(fb, fc).to_bits(),
                AluOp::FSgnJ => {
                    /// Bit mask for the sign bit in a 64-bit IEEE 754 float (bit 63).
                    const F64_SIGN_BIT_MASK: u64 = 0x8000_0000_0000_0000;
                    f64::from_bits(
                        (fa.to_bits() & !F64_SIGN_BIT_MASK) | (fb.to_bits() & F64_SIGN_BIT_MASK),
                    )
                    .to_bits()
                }
                AluOp::FSgnJN => {
                    /// Bit mask for the sign bit in a 64-bit IEEE 754 float (bit 63).
                    const F64_SIGN_BIT_MASK: u64 = 0x8000_0000_0000_0000;
                    f64::from_bits(
                        (fa.to_bits() & !F64_SIGN_BIT_MASK) | (!fb.to_bits() & F64_SIGN_BIT_MASK),
                    )
                    .to_bits()
                }
                AluOp::FSgnJX => {
                    /// Bit mask for the sign bit in a 64-bit IEEE 754 float (bit 63).
                    const F64_SIGN_BIT_MASK: u64 = 0x8000_0000_0000_0000;
                    f64::from_bits(fa.to_bits() ^ (fb.to_bits() & F64_SIGN_BIT_MASK)).to_bits()
                }
                AluOp::FEq => (fa == fb) as u64,
                AluOp::FLt => (fa < fb) as u64,
                AluOp::FLe => (fa <= fb) as u64,
                AluOp::FCvtWS => (fa as i32) as i64 as u64,
                AluOp::FCvtLS => (fa as i64) as u64,
                AluOp::FCvtSD => Self::box_f32(fa as f32),
                AluOp::FCvtSW => ((a as i32) as f64).to_bits(),
                AluOp::FCvtSL => ((a as i64) as f64).to_bits(),
                AluOp::FMvToF => a,
                AluOp::FMvToX => a,
                _ => 0,
            }
        }
    }
}

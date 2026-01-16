//! Unit tests for FPU operations.

use riscv_emulator::core::pipeline::signals::AluOp;
use riscv_emulator::core::units::fpu::Fpu;

/// Tests f32 to u64 boxing with sign extension.
#[test]
fn test_fpu_box_f32() {
    let f = 1.5f32;
    let boxed = Fpu::box_f32(f);
    let upper = (boxed >> 32) as u32;
    assert_eq!(upper, 0xFFFF_FFFF);
}

/// Tests 32-bit floating-point addition.
#[test]
fn test_fpu_fadd_32bit() {
    let a = 1.5f32.to_bits() as u64;
    let b = 2.5f32.to_bits() as u64;
    let result = Fpu::execute(AluOp::FAdd, a, b, 0, true);
    let result_f = f32::from_bits((result & 0xFFFF_FFFF) as u32);
    assert!((result_f - 4.0).abs() < 0.001);
}

/// Tests 64-bit floating-point addition.
#[test]
fn test_fpu_fadd_64bit() {
    let a = 1.5f64.to_bits();
    let b = 2.5f64.to_bits();
    let result = Fpu::execute(AluOp::FAdd, a, b, 0, false);
    let result_f = f64::from_bits(result);
    assert!((result_f - 4.0).abs() < 0.0001);
}

/// Tests floating-point subtraction.
#[test]
fn test_fpu_fsub() {
    let a = 5.0f64.to_bits();
    let b = 2.0f64.to_bits();
    let result = Fpu::execute(AluOp::FSub, a, b, 0, false);
    let result_f = f64::from_bits(result);
    assert!((result_f - 3.0).abs() < 0.0001);
}

/// Tests floating-point multiplication.
#[test]
fn test_fpu_fmul() {
    let a = 3.0f64.to_bits();
    let b = 4.0f64.to_bits();
    let result = Fpu::execute(AluOp::FMul, a, b, 0, false);
    let result_f = f64::from_bits(result);
    assert!((result_f - 12.0).abs() < 0.0001);
}

/// Tests floating-point division.
#[test]
fn test_fpu_fdiv() {
    let a = 15.0f64.to_bits();
    let b = 3.0f64.to_bits();
    let result = Fpu::execute(AluOp::FDiv, a, b, 0, false);
    let result_f = f64::from_bits(result);
    assert!((result_f - 5.0).abs() < 0.0001);
}

/// Tests floating-point square root.
#[test]
fn test_fpu_fsqrt() {
    let a = 16.0f64.to_bits();
    let result = Fpu::execute(AluOp::FSqrt, a, 0, 0, false);
    let result_f = f64::from_bits(result);
    assert!((result_f - 4.0).abs() < 0.0001);
}

/// Tests floating-point minimum and maximum operations.
#[test]
fn test_fpu_fmin_fmax() {
    let a = 1.5f64.to_bits();
    let b = 2.5f64.to_bits();

    let min_result = Fpu::execute(AluOp::FMin, a, b, 0, false);
    let min_f = f64::from_bits(min_result);
    assert!((min_f - 1.5).abs() < 0.0001);

    let max_result = Fpu::execute(AluOp::FMax, a, b, 0, false);
    let max_f = f64::from_bits(max_result);
    assert!((max_f - 2.5).abs() < 0.0001);
}

/// Tests floating-point multiply-add operation.
#[test]
fn test_fpu_fmadd() {
    let a = 2.0f64.to_bits();
    let b = 3.0f64.to_bits();
    let c = 4.0f64.to_bits();
    let result = Fpu::execute(AluOp::FMAdd, a, b, c, false);
    let result_f = f64::from_bits(result);
    assert!((result_f - 10.0).abs() < 0.0001);
}

/// Tests floating-point multiply-subtract operation.
#[test]
fn test_fpu_fmsub() {
    let a = 2.0f64.to_bits();
    let b = 3.0f64.to_bits();
    let c = 4.0f64.to_bits();
    let result = Fpu::execute(AluOp::FMSub, a, b, c, false);
    let result_f = f64::from_bits(result);
    assert!((result_f - 2.0).abs() < 0.0001);
}

/// Tests floating-point comparison operations (equal, less than, less than or equal).
#[test]
fn test_fpu_feq_flt_fle() {
    let a = 1.5f64.to_bits();
    let b = 1.5f64.to_bits();
    let c = 2.5f64.to_bits();

    assert_eq!(Fpu::execute(AluOp::FEq, a, b, 0, false), 1);
    assert_eq!(Fpu::execute(AluOp::FEq, a, c, 0, false), 0);

    assert_eq!(Fpu::execute(AluOp::FLt, a, c, 0, false), 1);
    assert_eq!(Fpu::execute(AluOp::FLt, c, a, 0, false), 0);

    assert_eq!(Fpu::execute(AluOp::FLe, a, b, 0, false), 1);
    assert_eq!(Fpu::execute(AluOp::FLe, a, c, 0, false), 1);
    assert_eq!(Fpu::execute(AluOp::FLe, c, a, 0, false), 0);
}

/// Tests floating-point to signed integer conversion.
#[test]
fn test_fpu_fcvt_ws() {
    let a = 42.7f32.to_bits() as u64;
    let result = Fpu::execute(AluOp::FCvtWS, a, 0, 0, true);
    assert_eq!(result, 42);
}

/// Tests signed integer to floating-point conversion.
#[test]
fn test_fpu_fcvt_sw() {
    let a = 42u64;
    let result = Fpu::execute(AluOp::FCvtSW, a, 0, 0, true);
    let result_f = f32::from_bits((result & 0xFFFF_FFFF) as u32);
    assert!((result_f - 42.0).abs() < 0.001);
}

/// Tests double-precision to single-precision conversion.
#[test]
fn test_fpu_fcvt_ds() {
    let a = 3.14159f32.to_bits() as u64;
    let result = Fpu::execute(AluOp::FCvtDS, a, 0, 0, false);
    let result_f = f64::from_bits(result);
    assert!((result_f - 3.14159).abs() < 0.0001);
}

/// Tests single-precision to double-precision conversion.
#[test]
fn test_fpu_fcvt_sd() {
    let a = 3.14159f64.to_bits();
    let result = Fpu::execute(AluOp::FCvtSD, a, 0, 0, false);
    let result_f = f32::from_bits((result & 0xFFFF_FFFF) as u32);
    assert!((result_f - 3.14159).abs() < 0.01);
}

/// Tests floating-point sign injection operation.
#[test]
fn test_fpu_fsgnj() {
    let a = (-1.5f64).to_bits();
    let b = 2.5f64.to_bits();
    let result = Fpu::execute(AluOp::FSgnJ, a, b, 0, false);
    let result_f = f64::from_bits(result);
    assert!((result_f - 1.5).abs() < 0.0001);
    assert!(result_f > 0.0);
}

/// Tests floating-point sign injection with negation.
#[test]
fn test_fpu_fsgnjn() {
    let a = 1.5f64.to_bits();
    let b = 2.5f64.to_bits();
    let result = Fpu::execute(AluOp::FSgnJN, a, b, 0, false);
    let result_f = f64::from_bits(result);
    assert!((result_f - (-1.5)).abs() < 0.0001);
    assert!(result_f < 0.0);
}

/// Tests floating-point to integer register move.
#[test]
fn test_fpu_fmv_to_x() {
    let a = 42.5f64.to_bits();
    let result = Fpu::execute(AluOp::FMvToX, a, 0, 0, false);
    assert_eq!(result, a);
}

/// Tests integer register to floating-point register move.
#[test]
fn test_fpu_fmv_to_f() {
    let a = 0x4042_8000_0000_0000u64;
    let result = Fpu::execute(AluOp::FMvToF, a, 0, 0, false);
    assert_eq!(result, a);
}

/// Tests floating-point operations with special values (zero, negative zero).
#[test]
fn test_fpu_special_values() {
    let zero = 0.0f64.to_bits();
    let one = 1.0f64.to_bits();
    let result = Fpu::execute(AluOp::FAdd, zero, one, 0, false);
    let result_f = f64::from_bits(result);
    assert!((result_f - 1.0).abs() < 0.0001);

    let neg_zero = (-0.0f64).to_bits();
    let result2 = Fpu::execute(AluOp::FAdd, neg_zero, one, 0, false);
    let result2_f = f64::from_bits(result2);
    assert!((result2_f - 1.0).abs() < 0.0001);
}

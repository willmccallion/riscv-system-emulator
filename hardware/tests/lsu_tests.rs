//! Unit tests for Load/Store Unit atomic operations.

use riscv_emulator::core::pipeline::signals::{AtomicOp, MemWidth};
use riscv_emulator::core::units::lsu::Lsu;

/// Tests atomic swap operation for word width.
#[test]
fn test_atomic_swap_word() {
    let mem_val = 0x1234_5678u64;
    let reg_val = 0x9ABC_DEF0u64;
    let result = Lsu::atomic_alu(AtomicOp::Swap, mem_val, reg_val, MemWidth::Word);
    assert_eq!(result as u32, 0x9ABC_DEF0u32);
}

/// Tests atomic swap operation for double word width.
#[test]
fn test_atomic_swap_double() {
    let mem_val = 0x1234_5678_9ABC_DEF0u64;
    let reg_val = 0xFEDC_BA98_7654_3210u64;
    let result = Lsu::atomic_alu(AtomicOp::Swap, mem_val, reg_val, MemWidth::Double);

    assert_eq!(result, reg_val);
}

/// Tests atomic add operation for word width.
#[test]
fn test_atomic_add_word() {
    let mem_val = 0x0000_0010u64;
    let reg_val = 0x0000_0020u64;
    let result = Lsu::atomic_alu(AtomicOp::Add, mem_val, reg_val, MemWidth::Word);
    assert_eq!(result as i32, 48i32);
}

/// Tests atomic add operation for double word width.
#[test]
fn test_atomic_add_double() {
    let mem_val = 100u64;
    let reg_val = 200u64;
    let result = Lsu::atomic_alu(AtomicOp::Add, mem_val, reg_val, MemWidth::Double);

    assert_eq!(result, 300u64);
}

/// Tests atomic add operation with overflow handling.
#[test]
fn test_atomic_add_overflow() {
    let mem_val = 0x7FFF_FFFFu64 as i32 as u64;
    let reg_val = 1u64;
    let result = Lsu::atomic_alu(AtomicOp::Add, mem_val, reg_val, MemWidth::Word);
    assert_eq!(result as i32, 0x8000_0000u32 as i32);
}

/// Tests atomic XOR operation.
#[test]
fn test_atomic_xor() {
    let mem_val = 0x1234_5678u64;
    let reg_val = 0x0000_FFFFu64;
    let result = Lsu::atomic_alu(AtomicOp::Xor, mem_val, reg_val, MemWidth::Word);

    assert_eq!(result as u32, 0x1234_A987u32);
}

/// Tests atomic AND operation.
#[test]
fn test_atomic_and() {
    let mem_val = 0x1234_5678u64;
    let reg_val = 0x0000_FFFFu64;
    let result = Lsu::atomic_alu(AtomicOp::And, mem_val, reg_val, MemWidth::Word);

    assert_eq!(result as u32, 0x0000_5678u32);
}

/// Tests atomic OR operation.
#[test]
fn test_atomic_or() {
    let mem_val = 0x1234_0000u64;
    let reg_val = 0x0000_5678u64;
    let result = Lsu::atomic_alu(AtomicOp::Or, mem_val, reg_val, MemWidth::Word);

    assert_eq!(result as u32, 0x1234_5678u32);
}

/// Tests atomic signed minimum operation.
#[test]
fn test_atomic_min_signed() {
    let mem_val = 10i32 as u64;
    let reg_val = 5i32 as u64;
    let result = Lsu::atomic_alu(AtomicOp::Min, mem_val, reg_val, MemWidth::Word);

    assert_eq!(result as i32, 5i32);
}

/// Tests atomic signed minimum with negative values.
#[test]
fn test_atomic_min_negative() {
    let mem_val = (-10i32) as u64;
    let reg_val = (-5i32) as u64;
    let result = Lsu::atomic_alu(AtomicOp::Min, mem_val, reg_val, MemWidth::Word);

    assert_eq!(result as i32, -10i32);
}

/// Tests atomic signed maximum operation.
#[test]
fn test_atomic_max_signed() {
    let mem_val = 10i32 as u64;
    let reg_val = 5i32 as u64;
    let result = Lsu::atomic_alu(AtomicOp::Max, mem_val, reg_val, MemWidth::Word);

    assert_eq!(result as i32, 10i32);
}

/// Tests atomic unsigned minimum operation.
#[test]
fn test_atomic_minu_unsigned() {
    let mem_val = 10u64;
    let reg_val = 5u64;
    let result = Lsu::atomic_alu(AtomicOp::Minu, mem_val, reg_val, MemWidth::Word);

    assert_eq!(result as u32, 5u32);
}

/// Tests atomic unsigned maximum operation.
#[test]
fn test_atomic_maxu_unsigned() {
    let mem_val = 10u64;
    let reg_val = 5u64;
    let result = Lsu::atomic_alu(AtomicOp::Maxu, mem_val, reg_val, MemWidth::Word);

    assert_eq!(result as u32, 10u32);
}

/// Tests atomic operations with double word precision.
#[test]
fn test_atomic_double_precision() {
    let mem_val = 0x1234_5678_9ABC_DEF0u64;
    let reg_val = 0xFEDC_BA98_7654_3210u64;

    let add_result = Lsu::atomic_alu(AtomicOp::Add, mem_val, reg_val, MemWidth::Double);
    assert_eq!(add_result, mem_val.wrapping_add(reg_val));

    let xor_result = Lsu::atomic_alu(AtomicOp::Xor, mem_val, reg_val, MemWidth::Double);
    assert_eq!(xor_result, mem_val ^ reg_val);
}

/// Tests sign extension of 32-bit results to 64 bits.
#[test]
fn test_atomic_sign_extension_word() {
    let mem_val = 0x8000_0000u64;
    let reg_val = 1u64;
    let result = Lsu::atomic_alu(AtomicOp::Add, mem_val, reg_val, MemWidth::Word);
    assert_eq!(result as i64, 0xFFFF_FFFF_8000_0001u64 as i64);
}

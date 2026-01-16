//! Unit tests for ALU operations.

use riscv_emulator::core::pipeline::signals::AluOp;
use riscv_emulator::core::units::alu::Alu;

/// Tests 64-bit addition operations.
#[test]
fn test_alu_add() {
    assert_eq!(Alu::execute(AluOp::Add, 10, 20, 0, false), 30);
    assert_eq!(
        Alu::execute(AluOp::Add, 0xFFFF_FFFF_FFFF_FFFF, 1, 0, false),
        0
    );
    assert_eq!(Alu::execute(AluOp::Add, 100, 200, 0, false), 300);
}

/// Tests 32-bit addition operations with sign extension.
#[test]
fn test_alu_add_32bit() {
    assert_eq!(Alu::execute(AluOp::Add, 10, 20, 0, true), 30);
    assert_eq!(
        Alu::execute(AluOp::Add, 0x7FFF_FFFF, 1, 0, true),
        0xFFFF_FFFF_8000_0000
    );
}

/// Tests 64-bit subtraction operations.
#[test]
fn test_alu_sub() {
    assert_eq!(Alu::execute(AluOp::Sub, 30, 10, 0, false), 20);
    assert_eq!(
        Alu::execute(AluOp::Sub, 0, 1, 0, false),
        0xFFFF_FFFF_FFFF_FFFF
    );
    assert_eq!(Alu::execute(AluOp::Sub, 100, 50, 0, false), 50);
}

/// Tests 32-bit subtraction operations with sign extension.
#[test]
fn test_alu_sub_32bit() {
    assert_eq!(Alu::execute(AluOp::Sub, 30, 10, 0, true), 20);
    assert_eq!(
        Alu::execute(AluOp::Sub, 0x8000_0000, 1, 0, true),
        0x0000_0000_7FFF_FFFF
    );
}

/// Tests logical left shift operations.
#[test]
fn test_alu_sll() {
    assert_eq!(Alu::execute(AluOp::Sll, 1, 3, 0, false), 8);
    assert_eq!(
        Alu::execute(AluOp::Sll, 0x1234_5678, 16, 0, false),
        0x0000_1234_5678_0000
    );
    assert_eq!(
        Alu::execute(AluOp::Sll, 1, 63, 0, false),
        0x8000_0000_0000_0000
    );
}

/// Tests logical right shift operations.
#[test]
fn test_alu_srl() {
    assert_eq!(Alu::execute(AluOp::Srl, 8, 3, 0, false), 1);
    assert_eq!(
        Alu::execute(AluOp::Srl, 0x8000_0000_0000_0000, 1, 0, false),
        0x4000_0000_0000_0000
    );
    assert_eq!(
        Alu::execute(AluOp::Srl, 0xFFFF_FFFF_FFFF_FFFF, 32, 0, false),
        0xFFFF_FFFF
    );
}

/// Tests arithmetic right shift operations with sign preservation.
#[test]
fn test_alu_sra() {
    assert_eq!(Alu::execute(AluOp::Sra, 8, 3, 0, false), 1);
    assert_eq!(
        Alu::execute(AluOp::Sra, 0x8000_0000_0000_0000, 1, 0, false),
        0xC000_0000_0000_0000
    );
    assert_eq!(
        Alu::execute(AluOp::Sra, 0xFFFF_FFFF_FFFF_FFFF, 1, 0, false),
        0xFFFF_FFFF_FFFF_FFFF
    );
}

/// Tests logical operations (OR, AND, XOR).
#[test]
fn test_alu_logical() {
    assert_eq!(Alu::execute(AluOp::Or, 0x1234, 0x5678, 0, false), 0x567C);
    assert_eq!(Alu::execute(AluOp::And, 0x1234, 0x5678, 0, false), 0x1230);
    assert_eq!(Alu::execute(AluOp::Xor, 0x1234, 0x5678, 0, false), 0x444C);
}

/// Tests set less than (signed) operations.
#[test]
fn test_alu_slt() {
    assert_eq!(Alu::execute(AluOp::Slt, 10, 20, 0, false), 1);
    assert_eq!(Alu::execute(AluOp::Slt, 20, 10, 0, false), 0);
    assert_eq!(
        Alu::execute(AluOp::Slt, 0x8000_0000_0000_0000, 0, 0, false),
        1
    );
    assert_eq!(
        Alu::execute(AluOp::Slt, 0, 0x8000_0000_0000_0000, 0, false),
        0
    );
}

/// Tests set less than (unsigned) operations.
#[test]
fn test_alu_sltu() {
    assert_eq!(Alu::execute(AluOp::Sltu, 10, 20, 0, false), 1);
    assert_eq!(Alu::execute(AluOp::Sltu, 20, 10, 0, false), 0);
    assert_eq!(
        Alu::execute(AluOp::Sltu, 0x8000_0000_0000_0000, 0, 0, false),
        0
    );
}

/// Tests multiplication operations.
#[test]
fn test_alu_mul() {
    assert_eq!(Alu::execute(AluOp::Mul, 6, 7, 0, false), 42);
    assert_eq!(
        Alu::execute(AluOp::Mul, 0xFFFF_FFFF, 2, 0, false),
        0x1FFFF_FFFE
    );
    assert_eq!(Alu::execute(AluOp::Mul, 0, 100, 0, false), 0);
}

/// Tests high bits of signed multiplication.
#[test]
fn test_alu_mulh() {
    let result = Alu::execute(AluOp::Mulh, 0x8000_0000_0000_0000, 2, 0, false);
    assert_eq!(result, 0x1);
}

/// Tests signed division operations.
#[test]
fn test_alu_div() {
    assert_eq!(Alu::execute(AluOp::Div, 20, 4, 0, false), 5);
    assert_eq!(
        Alu::execute(AluOp::Div, -20i64 as u64, 4, 0, false),
        (-5i64) as u64
    );
    assert_eq!(
        Alu::execute(AluOp::Div, 100, 0, 0, false),
        0xFFFF_FFFF_FFFF_FFFF
    );
}

/// Tests unsigned division operations.
#[test]
fn test_alu_divu() {
    assert_eq!(Alu::execute(AluOp::Divu, 20, 4, 0, false), 5);
    assert_eq!(
        Alu::execute(AluOp::Divu, 0xFFFF_FFFF_FFFF_FFFF, 2, 0, false),
        0x7FFF_FFFF_FFFF_FFFF
    );
    assert_eq!(
        Alu::execute(AluOp::Divu, 100, 0, 0, false),
        0xFFFF_FFFF_FFFF_FFFF
    );
}

/// Tests signed remainder operations.
#[test]
fn test_alu_rem() {
    assert_eq!(Alu::execute(AluOp::Rem, 23, 5, 0, false), 3);
    assert_eq!(
        Alu::execute(AluOp::Rem, -23i64 as u64, 5, 0, false),
        (-3i64) as u64
    );
    assert_eq!(Alu::execute(AluOp::Rem, 100, 0, 0, false), 100);
}

/// Tests unsigned remainder operations.
#[test]
fn test_alu_remu() {
    assert_eq!(Alu::execute(AluOp::Remu, 23, 5, 0, false), 3);
    assert_eq!(
        Alu::execute(AluOp::Remu, 0xFFFF_FFFF_FFFF_FFFF, 2, 0, false),
        1
    );
    assert_eq!(Alu::execute(AluOp::Remu, 100, 0, 0, false), 100);
}

/// Tests shift amount masking for 64-bit operations.
#[test]
fn test_alu_shift_amount_masking() {
    assert_eq!(Alu::execute(AluOp::Sll, 1, 64, 0, false), 1);
    assert_eq!(
        Alu::execute(AluOp::Sll, 1, 127, 0, false),
        0x8000_0000_0000_0000
    );
}

/// Tests signed division by zero.
///
/// RISC-V specifies that division by zero should return -1 (all bits set),
/// and does not trap.
#[test]
fn test_alu_div_by_zero() {
    let result = Alu::execute(AluOp::Div, 100, 0, 0, false);
    assert_eq!(result, 0xFFFF_FFFF_FFFF_FFFF);
}

/// Tests signed division overflow.
///
/// The only overflow case in division is `MIN_INT / -1`.
/// RISC-V specifies the result should be `MIN_INT`.
#[test]
fn test_alu_div_overflow() {
    let min_int = 0x8000_0000_0000_0000u64;
    let minus_one = 0xFFFF_FFFF_FFFF_FFFFu64;

    let result = Alu::execute(AluOp::Div, min_int, minus_one, 0, false);
    assert_eq!(result, min_int);
}

/// Tests unsigned division by zero.
///
/// Should return maximum unsigned value (all bits set).
#[test]
fn test_alu_divu_by_zero() {
    let result = Alu::execute(AluOp::Divu, 100, 0, 0, false);
    assert_eq!(result, 0xFFFF_FFFF_FFFF_FFFF);
}

/// Tests signed remainder by zero.
///
/// RISC-V specifies `x % 0 = x`.
#[test]
fn test_alu_rem_by_zero() {
    let dividend = 12345;
    let result = Alu::execute(AluOp::Rem, dividend, 0, 0, false);
    assert_eq!(result, dividend);
}

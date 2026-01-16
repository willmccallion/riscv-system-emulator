//! Stress tests and edge case tests.

use riscv_emulator::common::*;
use riscv_emulator::core::pipeline::signals::AluOp;
use riscv_emulator::core::units::alu::Alu;
use riscv_emulator::core::units::fpu::Fpu;

/// Stress test for register file operations across all registers.
#[test]
fn test_register_file_stress() {
    let mut regs = RegisterFile::new();

    for i in 0..32 {
        let val = (i as u64) * 0x1111_1111_1111_1111;
        regs.write(i, val);
    }

    for i in 0..32 {
        if i == 0 {
            assert_eq!(regs.read(i), 0);
        } else {
            let expected = (i as u64) * 0x1111_1111_1111_1111;
            assert_eq!(regs.read(i), expected);
        }
    }
}

/// Stress test for ALU overflow handling.
#[test]
fn test_alu_overflow_stress() {
    let max_u64 = 0xFFFF_FFFF_FFFF_FFFFu64;

    assert_eq!(Alu::execute(AluOp::Add, max_u64, 1, 0, false), 0);
    assert_eq!(
        Alu::execute(AluOp::Add, max_u64, max_u64, 0, false),
        0xFFFF_FFFF_FFFF_FFFE
    );

    assert_eq!(Alu::execute(AluOp::Sub, 0, 1, 0, false), max_u64);
    assert_eq!(Alu::execute(AluOp::Sub, 0, max_u64, 0, false), 1);
}

/// Stress test for ALU shift operations with all shift amounts.
#[test]
fn test_alu_shift_stress() {
    for shift in 0..64 {
        let result = Alu::execute(AluOp::Sll, 1, shift, 0, false);
        if shift < 64 {
            assert_eq!(result, 1u64 << shift);
        }
    }

    assert_eq!(Alu::execute(AluOp::Sll, 1, 64, 0, false), 1);
    assert_eq!(Alu::execute(AluOp::Sll, 1, 128, 0, false), 1);
}

/// Stress test for FPU precision across many operations.
#[test]
fn test_fpu_precision_stress() {
    for i in 1..100 {
        let a = (i as f64) * 0.1;
        let b = (i as f64) * 0.2;

        let a_bits = a.to_bits();
        let b_bits = b.to_bits();

        let result = Fpu::execute(AluOp::FAdd, a_bits, b_bits, 0, false);
        let result_f = f64::from_bits(result);
        let expected = a + b;

        assert!(
            (result_f - expected).abs() < 0.0001,
            "Failed for i={}: got {}, expected {}",
            i,
            result_f,
            expected
        );
    }
}

/// Tests FPU operations with extreme values.
#[test]
fn test_fpu_extreme_values() {
    let large = 1e308f64.to_bits();
    let result = Fpu::execute(AluOp::FMul, large, 2.0f64.to_bits(), 0, false);
    let result_f = f64::from_bits(result);
    assert!(result_f.is_infinite() || result_f > 1e300);

    let small = 1e-308f64.to_bits();
    let result2 = Fpu::execute(AluOp::FMul, small, 0.5f64.to_bits(), 0, false);
    let result2_f = f64::from_bits(result2);
    assert!(result2_f < 1e-300 || result2_f == 0.0);
}

/// Tests virtual address edge cases and boundary conditions.
#[test]
fn test_virt_addr_edge_cases() {
    let addr1 = VirtAddr::new(0);
    assert_eq!(addr1.val(), 0);
    assert_eq!(addr1.page_offset(), 0);

    let addr2 = VirtAddr::new(0xFFF);
    assert_eq!(addr2.page_offset(), 0xFFF);

    let addr3 = VirtAddr::new(0x1000);
    assert_eq!(addr3.page_offset(), 0);

    let addr4 = VirtAddr::new(0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(addr4.page_offset(), 0xFFF);
}

/// Stress test for CSR write and read operations.
#[test]
fn test_csr_write_read_stress() {
    use riscv_emulator::core::arch::csr::*;
    let mut csrs = Csrs::default();

    let test_values = vec![
        (MSTATUS, 0x8000_0000_0000_0000),
        (MTVEC, 0x1000_0000),
        (MEPC, 0x2000_0000),
        (MCAUSE, 0x1234_5678),
        (MSCRATCH, 0xDEAD_BEEF),
        (SSTATUS, 0x4000_0000),
        (STVEC, 0x3000_0000),
        (SEPC, 0x4000_0000),
    ];

    for (addr, val) in test_values {
        csrs.write(addr, val);
        assert_eq!(csrs.read(addr), val);
    }
}

/// Tests privilege mode ordering and comparison.
#[test]
fn test_privilege_mode_ordering() {
    use riscv_emulator::core::arch::mode::PrivilegeMode;

    assert!(PrivilegeMode::User < PrivilegeMode::Supervisor);
    assert!(PrivilegeMode::Supervisor < PrivilegeMode::Machine);
    assert!(PrivilegeMode::User < PrivilegeMode::Machine);

    assert_eq!(PrivilegeMode::User, PrivilegeMode::User);
    assert_ne!(PrivilegeMode::User, PrivilegeMode::Machine);
}

/// Tests trap cloning and equality operations.
#[test]
fn test_trap_clone_and_equality() {
    let trap1 = Trap::IllegalInstruction(0x1234);
    let trap2 = trap1.clone();
    assert_eq!(trap1, trap2);

    let trap3 = Trap::IllegalInstruction(0x5678);
    assert_ne!(trap1, trap3);

    let traps = vec![
        Trap::InstructionAddressMisaligned(0x1001),
        Trap::LoadPageFault(0x2000),
        Trap::EnvironmentCallFromUMode,
        Trap::MachineTimerInterrupt,
    ];

    for trap in traps {
        let _clone = trap.clone();
    }
}

/// Tests all translation result variants.
#[test]
fn test_translation_result_all_variants() {
    let paddr = PhysAddr::new(0x1000_0000);

    let success = TranslationResult::success(paddr, 5);
    assert_eq!(success.paddr.val(), 0x1000_0000);
    assert_eq!(success.cycles, 5);
    assert!(success.trap.is_none());

    let trap = Trap::LoadPageFault(0x2000);
    let fault = TranslationResult::fault(trap.clone(), 3);
    assert_eq!(fault.paddr.val(), 0);
    assert_eq!(fault.cycles, 3);
    assert_eq!(fault.trap, Some(trap));
}

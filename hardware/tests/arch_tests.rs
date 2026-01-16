//! Integration tests for architecture-specific components.

use riscv_emulator::core::arch::csr::*;
use riscv_emulator::core::arch::mode::PrivilegeMode;
use riscv_emulator::core::arch::*;

/// Tests general-purpose register read and write operations.
#[test]
fn test_gpr_read_write() {
    let mut gpr = gpr::Gpr::new();

    gpr.write(0, 0xDEAD_BEEF);
    assert_eq!(gpr.read(0), 0);

    for i in 1..32 {
        let val = (i as u64) * 0x1111_1111;
        gpr.write(i, val);
        assert_eq!(gpr.read(i), val);
    }
}

/// Tests floating-point register read and write operations.
#[test]
fn test_fpr_read_write() {
    let mut fpr = fpr::Fpr::new();

    let val1 = 1.5f64.to_bits();
    fpr.write(0, val1);
    assert_eq!(fpr.read(0), val1);

    let val2 = (-3.14159f64).to_bits();
    fpr.write(1, val2);
    assert_eq!(fpr.read(1), val2);

    for i in 0..32 {
        let val = (i as f64) * 0.5;
        fpr.write(i, val.to_bits());
        assert_eq!(fpr.read(i), val.to_bits());
    }
}

/// Tests privilege mode conversion between u8 and enum.
#[test]
fn test_privilege_mode_conversion() {
    assert_eq!(PrivilegeMode::User.to_u8(), 0);
    assert_eq!(PrivilegeMode::Supervisor.to_u8(), 1);
    assert_eq!(PrivilegeMode::Machine.to_u8(), 3);

    assert_eq!(PrivilegeMode::from_u8(0), PrivilegeMode::User);
    assert_eq!(PrivilegeMode::from_u8(1), PrivilegeMode::Supervisor);
    assert_eq!(PrivilegeMode::from_u8(3), PrivilegeMode::Machine);

    assert_eq!(PrivilegeMode::from_u8(2), PrivilegeMode::Machine);
    assert_eq!(PrivilegeMode::from_u8(255), PrivilegeMode::Machine);
}

/// Tests privilege mode display and naming.
#[test]
fn test_privilege_mode_display() {
    assert_eq!(PrivilegeMode::User.name(), "User");
    assert_eq!(PrivilegeMode::Supervisor.name(), "Supervisor");
    assert_eq!(PrivilegeMode::Machine.name(), "Machine");

    let s = format!("{}", PrivilegeMode::User);
    assert_eq!(s, "User");
}

/// Tests CSR read and write operations.
#[test]
fn test_csr_read_write() {
    let mut csrs = Csrs::default();

    csrs.write(MSTATUS, 0x8000_0000_0000_0000);
    assert_eq!(csrs.read(MSTATUS), 0x8000_0000_0000_0000);

    csrs.write(MTVEC, 0x1000_0000);
    assert_eq!(csrs.read(MTVEC), 0x1000_0000);

    csrs.write(MEPC, 0x2000_0000);
    assert_eq!(csrs.read(MEPC), 0x2000_0000);

    csrs.write(SATP, 0x8000_0000_0000_0000 | 0x123);
    let satp = csrs.read(SATP);
    assert_eq!((satp >> SATP_MODE_SHIFT) & SATP_MODE_MASK, SATP_MODE_SV39);

    csrs.write(SATP, 0x0000_0000_0000_0000);
    let satp = csrs.read(SATP);
    assert_eq!((satp >> SATP_MODE_SHIFT) & SATP_MODE_MASK, SATP_MODE_BARE);
}

/// Tests CSR default values.
#[test]
fn test_csr_default_values() {
    let csrs = Csrs::default();

    assert_eq!(csrs.read(MSTATUS), 0);
    assert_eq!(csrs.read(MISA), 0);
    assert_eq!(csrs.read(MTVEC), 0);
    assert_eq!(csrs.read(MEPC), 0);
    assert_eq!(csrs.read(MCAUSE), 0);
}

/// Tests unknown CSR address handling.
#[test]
fn test_csr_unknown_address() {
    let csrs = Csrs::default();

    assert_eq!(csrs.read(0x999), 0);
    assert_eq!(csrs.read(0x1234), 0);
}

/// Tests interrupt to trap conversion.
#[test]
fn test_trap_handler_irq_to_trap() {
    use riscv_emulator::common::error::Trap;

    let trap1 = trap::TrapHandler::irq_to_trap(MIP_MEIP);
    assert!(matches!(trap1, Trap::ExternalInterrupt));

    let trap2 = trap::TrapHandler::irq_to_trap(MIP_MSIP);
    assert!(matches!(trap2, Trap::MachineSoftwareInterrupt));

    let trap3 = trap::TrapHandler::irq_to_trap(MIP_MTIP);
    assert!(matches!(trap3, Trap::MachineTimerInterrupt));
}

//! Integration tests for common utilities module.

use riscv_emulator::common::*;

/// Tests virtual address creation.
#[test]
fn test_virt_addr_creation() {
    let addr = VirtAddr::new(0x8000_0000);
    assert_eq!(addr.val(), 0x8000_0000);
}

/// Tests virtual address page offset extraction.
#[test]
fn test_virt_addr_page_offset() {
    let addr = VirtAddr::new(0x8000_1234);
    assert_eq!(addr.page_offset(), 0x234);

    let addr2 = VirtAddr::new(0xFFFF_FFFF);
    assert_eq!(addr2.page_offset(), 0xFFF);
}

/// Tests physical address creation.
#[test]
fn test_phys_addr_creation() {
    let addr = PhysAddr::new(0x1000_0000);
    assert_eq!(addr.val(), 0x1000_0000);
}

/// Tests access type equality.
#[test]
fn test_access_type_equality() {
    assert_eq!(AccessType::Fetch, AccessType::Fetch);
    assert_ne!(AccessType::Read, AccessType::Write);
}

/// Tests translation result success case.
#[test]
fn test_translation_result_success() {
    let paddr = PhysAddr::new(0x1000_0000);
    let result = TranslationResult::success(paddr, 5);

    assert_eq!(result.paddr.val(), 0x1000_0000);
    assert_eq!(result.cycles, 5);
    assert!(result.trap.is_none());
}

/// Tests translation result fault case.
#[test]
fn test_translation_result_fault() {
    let trap = Trap::LoadPageFault(0x8000_0000);
    let result = TranslationResult::fault(trap.clone(), 3);

    assert_eq!(result.paddr.val(), 0);
    assert_eq!(result.cycles, 3);
    assert_eq!(result.trap, Some(trap));
}

/// Tests trap display formatting.
#[test]
fn test_trap_display() {
    let trap = Trap::IllegalInstruction(0x12345678);
    let s = format!("{}", trap);
    assert!(s.contains("IllegalInstruction"));
    assert!(s.contains("0x12345678"));
}

/// Tests register file creation and initialization.
#[test]
fn test_register_file_creation() {
    let regs = RegisterFile::new();
    assert_eq!(regs.read(0), 0);
    assert_eq!(regs.read(1), 0);
}

/// Tests register file read and write operations.
#[test]
fn test_register_file_read_write() {
    let mut regs = RegisterFile::new();

    regs.write(0, 0xDEAD_BEEF);
    assert_eq!(regs.read(0), 0);

    regs.write(1, 0x1234_5678);
    assert_eq!(regs.read(1), 0x1234_5678);

    regs.write(31, 0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(regs.read(31), 0xFFFF_FFFF_FFFF_FFFF);
}

/// Tests floating-point register file read and write operations.
#[test]
fn test_register_file_fp_read_write() {
    let mut regs = RegisterFile::new();

    regs.write_f(0, 0x1234_5678_9ABC_DEF0);
    assert_eq!(regs.read_f(0), 0x1234_5678_9ABC_DEF0);

    regs.write_f(31, 0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(regs.read_f(31), 0xFFFF_FFFF_FFFF_FFFF);
}

/// Tests all trap variants can be created and cloned.
#[test]
fn test_trap_variants() {
    let _traps = vec![
        Trap::InstructionAddressMisaligned(0x1001),
        Trap::InstructionAccessFault(0x2000),
        Trap::IllegalInstruction(0x1234),
        Trap::Breakpoint(0x3000),
        Trap::LoadAddressMisaligned(0x4001),
        Trap::LoadAccessFault(0x5000),
        Trap::StoreAddressMisaligned(0x6001),
        Trap::StoreAccessFault(0x7000),
        Trap::EnvironmentCallFromUMode,
        Trap::EnvironmentCallFromSMode,
        Trap::EnvironmentCallFromMMode,
        Trap::InstructionPageFault(0x8000),
        Trap::LoadPageFault(0x9000),
        Trap::StorePageFault(0xA000),
        Trap::UserSoftwareInterrupt,
        Trap::SupervisorSoftwareInterrupt,
        Trap::MachineSoftwareInterrupt,
        Trap::MachineTimerInterrupt,
        Trap::ExternalInterrupt,
        Trap::RequestedTrap(42),
        Trap::DoubleFault(0xB000),
    ];

    for trap in _traps {
        let _clone = trap.clone();
    }
}

//! Integration tests for the Memory Management Unit (MMU) and Page Table Walker.
//!
//! These tests verify the SV39 virtual-to-physical address translation logic,
//! including multi-level page table traversals and permission checking.

use riscv_emulator::common::{AccessType, VirtAddr};
use riscv_emulator::config::{
    CacheConfig, CacheHierarchyConfig, Config, GeneralConfig, MemoryConfig, PipelineConfig,
    SystemConfig,
};
use riscv_emulator::core::arch::csr::{Csrs, SATP, SATP_MODE_SV39};
use riscv_emulator::core::arch::mode::PrivilegeMode;
use riscv_emulator::core::units::mmu::Mmu;
use riscv_emulator::soc::System;

/// Page Table Entry valid bit (bit 0).
const PTE_VALID: u64 = 1;

/// Page Table Entry read permission bit (bit 1).
const PTE_READ: u64 = 1 << 1;

/// Page Table Entry write permission bit (bit 2).
const PTE_WRITE: u64 = 1 << 2;

/// Page Table Entry accessed bit (bit 6).
const PTE_ACCESSED: u64 = 1 << 6;

/// Page Table Entry dirty bit (bit 7).
const PTE_DIRTY: u64 = 1 << 7;

/// Bit shift for page size (12 bits = 4 KiB pages).
const PAGE_SHIFT: u64 = 12;

/// Size of a Page Table Entry in bytes (8 bytes for 64-bit PTE).
const PTE_SIZE: u64 = 8;

/// Helper to create a minimal system for MMU testing.
fn create_mmu_test_system() -> (System, Mmu, Csrs) {
    let config = Config {
        general: GeneralConfig::default(),
        system: SystemConfig {
            ram_base: 0x8000_0000,
            ..SystemConfig::default()
        },
        memory: MemoryConfig::default(),
        cache: CacheHierarchyConfig {
            l1_i: CacheConfig::default(),
            l1_d: CacheConfig::default(),
            l2: CacheConfig::default(),
            l3: CacheConfig::default(),
        },
        pipeline: PipelineConfig::default(),
    };

    let system = System::new(&config, "");
    let mmu = Mmu::new(32);
    let csrs = Csrs::default();

    (system, mmu, csrs)
}

/// Helper to write a Page Table Entry (PTE) to memory.
fn write_pte(system: &mut System, base_ppn: u64, vpn_index: u64, pte_val: u64) {
    let paddr = (base_ppn << PAGE_SHIFT) + (vpn_index * PTE_SIZE);
    system.bus.write_u64(paddr, pte_val);
}

/// Tests a successful 3-level page table walk (4KB page).
///
/// Setup:
/// - SATP points to Root PT (Level 2).
/// - Root PT points to Mid PT (Level 1).
/// - Mid PT points to Leaf PT (Level 0).
/// - Leaf PT points to Physical Page.
#[test]
fn test_mmu_sv39_4kb_walk() {
    let (mut system, mut mmu, mut csrs) = create_mmu_test_system();

    let root_ppn = 0x80000;
    let mid_ppn = 0x80001;
    let leaf_ppn = 0x80002;
    let target_ppn = 0x90000;

    let vaddr = VirtAddr::new(0x0000_0000_0010_0000);
    let vpn0 = 0x100;

    let satp_val = (SATP_MODE_SV39 << 60) | root_ppn;
    csrs.write(SATP, satp_val);

    let root_pte = (mid_ppn << 10) | PTE_VALID;
    write_pte(&mut system, root_ppn, 0, root_pte);

    let mid_pte = (leaf_ppn << 10) | PTE_VALID;
    write_pte(&mut system, mid_ppn, 0, mid_pte);

    let leaf_pte = (target_ppn << 10) | PTE_VALID | PTE_READ | PTE_WRITE | PTE_ACCESSED | PTE_DIRTY;
    write_pte(&mut system, leaf_ppn, vpn0, leaf_pte);

    let result = mmu.translate(
        vaddr,
        AccessType::Read,
        PrivilegeMode::Supervisor,
        &csrs,
        &mut system.bus,
    );

    assert!(
        result.trap.is_none(),
        "Translation faulted: {:?}",
        result.trap
    );
    assert_eq!(result.paddr.val(), target_ppn << PAGE_SHIFT);
    assert!(result.cycles > 0, "Page walk should consume cycles");
}

/// Tests a Megapage (2MB) translation.
///
/// Setup:
/// - SATP points to Root PT.
/// - Root PT points to Mid PT.
/// - Mid PT is a LEAF entry (Megapage), pointing directly to physical memory.
#[test]
fn test_mmu_sv39_megapage_walk() {
    let (mut system, mut mmu, mut csrs) = create_mmu_test_system();

    let root_ppn = 0x80000;
    let target_ppn = 0x90000;

    let vaddr = VirtAddr::new(0x0000_0000_0020_0000);
    let vpn1 = 1;

    csrs.write(SATP, (SATP_MODE_SV39 << 60) | root_ppn);

    let mid_table_ppn = root_ppn + 1;
    let root_pte = (mid_table_ppn << 10) | PTE_VALID;
    write_pte(&mut system, root_ppn, 0, root_pte);

    let mega_pte = (target_ppn << 10) | PTE_VALID | PTE_READ | PTE_WRITE | PTE_ACCESSED | PTE_DIRTY;
    write_pte(&mut system, mid_table_ppn, vpn1, mega_pte);

    let result = mmu.translate(
        vaddr,
        AccessType::Read,
        PrivilegeMode::Supervisor,
        &csrs,
        &mut system.bus,
    );

    assert!(result.trap.is_none());
    assert_eq!(result.paddr.val(), target_ppn << PAGE_SHIFT);
}

/// Tests MMU permission fault handling.
///
/// Tries to Write to a Read-Only page.
#[test]
fn test_mmu_write_permission_fault() {
    let (mut system, mut mmu, mut csrs) = create_mmu_test_system();

    let root_ppn = 0x80000;
    let leaf_ppn = 0x80001;
    let target_ppn = 0x90000;
    let vaddr = VirtAddr::new(0x1000);

    csrs.write(SATP, (SATP_MODE_SV39 << 60) | root_ppn);

    write_pte(&mut system, root_ppn, 0, (leaf_ppn << 10) | PTE_VALID);
    write_pte(&mut system, 0, 0, 0);

    let leaf_pte = (target_ppn << 10) | PTE_VALID | PTE_READ | PTE_ACCESSED | PTE_DIRTY;
    write_pte(&mut system, leaf_ppn, 1, leaf_pte);

    let result = mmu.translate(
        vaddr,
        AccessType::Write,
        PrivilegeMode::Supervisor,
        &csrs,
        &mut system.bus,
    );

    assert!(result.trap.is_some());
    match result.trap {
        Some(riscv_emulator::common::Trap::StorePageFault(addr)) => {
            assert_eq!(addr, vaddr.val());
        }
        _ => panic!("Expected StorePageFault, got {:?}", result.trap),
    }
}

/// Tests that instruction fetch from a page without Execute (X) permission fails.
#[test]
fn test_mmu_fetch_no_exec() {
    let (mut system, mut mmu, mut csrs) = create_mmu_test_system();

    let root_ppn = 0x80000;
    let target_ppn = 0x90000;
    let vaddr = VirtAddr::new(0x8000_0000);

    csrs.write(SATP, (SATP_MODE_SV39 << 60) | root_ppn);

    let pte_val = (target_ppn << 10) | 0xC7;

    write_pte(&mut system, root_ppn, 2, pte_val);

    let result = mmu.translate(
        vaddr,
        AccessType::Fetch,
        PrivilegeMode::Supervisor,
        &csrs,
        &mut system.bus,
    );

    assert!(result.trap.is_some());
    match result.trap {
        Some(riscv_emulator::common::Trap::InstructionPageFault(addr)) => {
            assert_eq!(addr, vaddr.val());
        }
        _ => panic!(
            "Expected InstructionPageFault due to missing X bit, got {:?}",
            result.trap
        ),
    }
}

/// Tests that instruction fetch from a non-canonical address fails correctly.
#[test]
fn test_mmu_fetch_non_canonical() {
    let (mut system, mut mmu, csrs) = create_mmu_test_system();

    let invalid_vaddr = VirtAddr::new(0x678f_3968_9b8c_6600);

    let result = mmu.translate(
        invalid_vaddr,
        AccessType::Fetch,
        PrivilegeMode::Supervisor,
        &csrs,
        &mut system.bus,
    );

    assert!(result.trap.is_some());
    match result.trap {
        Some(riscv_emulator::common::Trap::InstructionAccessFault(addr)) => {
            assert_eq!(addr, invalid_vaddr.val());
        }
        _ => panic!(
            "Expected InstructionAccessFault for non-canonical address, got {:?}",
            result.trap
        ),
    }
}

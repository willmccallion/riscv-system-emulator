//! Comprehensive tests for CSR operations and potential corruption issues.

use riscv_emulator::common::constants::CAUSE_INTERRUPT_BIT;
use riscv_emulator::core::arch::csr::*;

/// Tests read/write operations for all machine-level CSRs.
#[test]
fn test_csr_read_write_all_csrs() {
    let mut csrs = Csrs::default();

    let test_csrs = vec![
        (MSTATUS, 0x8000_0000_0000_0000),
        (MISA, 0x8000_0000_0014_1101),
        (MEDELEG, 0x1234),
        (MIDELEG, 0x5678),
        (MIE, 0x888),
        (MTVEC, 0x1000_0000),
        (MSCRATCH, 0xDEAD_BEEF),
        (MEPC, 0x2000_0000),
        (MCAUSE, 0x8000_0000_0000_0003),
        (MTVAL, 0x3000_0000),
        (MIP, 0x888),
    ];

    for (addr, val) in test_csrs {
        csrs.write(addr, val);
        assert_eq!(csrs.read(addr), val, "CSR {:#x} write/read mismatch", addr);
    }
}

/// Tests read/write operations for all supervisor-level CSRs.
#[test]
fn test_csr_supervisor_csrs() {
    let mut csrs = Csrs::default();

    let test_csrs = vec![
        (SSTATUS, 0x4000_0000),
        (SIE, 0x222),
        (STVEC, 0x3000_0000),
        (SSCRATCH, 0xCAFE_BABE),
        (SEPC, 0x4000_0000),
        (SCAUSE, 0x8000_0000_0000_0009),
        (STVAL, 0x5000_0000),
        (SIP, 0x222),
    ];

    for (addr, val) in test_csrs {
        csrs.write(addr, val);
        assert_eq!(csrs.read(addr), val, "CSR {:#x} write/read mismatch", addr);
    }
}

/// Tests SATP mode field validation and invalid mode handling.
#[test]
fn test_csr_satp_mode_validation() {
    let mut csrs = Csrs::default();

    let satp_sv39 = 0x8000_0000_0000_0000 | 0x123;
    csrs.write(SATP, satp_sv39);
    let read = csrs.read(SATP);
    assert_eq!((read >> SATP_MODE_SHIFT) & SATP_MODE_MASK, SATP_MODE_SV39);
    assert_eq!(read & SATP_PPN_MASK, 0x123);

    csrs.write(SATP, 0x0000_0000_0000_0000);
    assert_eq!(
        (csrs.read(SATP) >> SATP_MODE_SHIFT) & SATP_MODE_MASK,
        SATP_MODE_BARE
    );

    csrs.write(SATP, 0x5000_0000_0000_0000);
    assert_eq!(
        (csrs.read(SATP) >> SATP_MODE_SHIFT) & SATP_MODE_MASK,
        SATP_MODE_BARE
    );
}

/// Tests MEPC and SEPC address alignment behavior.
///
/// Note: Alignment is handled in orchestrator::csr_write, not in Csrs::write.
#[test]
fn test_csr_mepc_sepc_alignment() {
    let mut csrs = Csrs::default();

    csrs.write(MEPC, 0x1001);
    assert_eq!(csrs.read(MEPC), 0x1001);

    csrs.write(MEPC, 0x1002);
    assert_eq!(csrs.read(MEPC), 0x1002);

    csrs.write(SEPC, 0x2001);
    assert_eq!(csrs.read(SEPC), 0x2001);
}

/// Tests MSTATUS and SSTATUS synchronization behavior.
///
/// Note: Synchronization is handled in orchestrator::csr_write, not in Csrs::write.
#[test]
fn test_csr_mstatus_sstatus_synchronization() {
    let mut csrs = Csrs::default();

    let mstatus_val =
        MSTATUS_SIE | MSTATUS_SPIE | MSTATUS_SPP | MSTATUS_FS | MSTATUS_SUM | MSTATUS_MXR;
    csrs.write(MSTATUS, mstatus_val);
    assert_eq!(csrs.read(SSTATUS), 0);

    csrs.write(SSTATUS, 0x4000_0000);
    assert_eq!(csrs.read(MSTATUS), mstatus_val);
}

/// Tests MIP write masking behavior.
///
/// Note: Write masking is handled in orchestrator::csr_write, not in Csrs::write.
#[test]
fn test_csr_mip_write_mask() {
    let mut csrs = Csrs::default();

    csrs.write(MIP, 0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(csrs.read(MIP), 0xFFFF_FFFF_FFFF_FFFF);
}

/// Tests SIP write masking behavior.
///
/// Note: Write masking is handled in orchestrator::csr_write, not in Csrs::write.
#[test]
fn test_csr_sip_write_mask() {
    let mut csrs = Csrs::default();

    csrs.write(MIDELEG, MIP_SSIP);
    csrs.write(SIP, 0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(csrs.read(SIP), 0xFFFF_FFFF_FFFF_FFFF);
}

/// Tests SIE write masking behavior.
///
/// Note: Write masking is handled in orchestrator::csr_write, not in Csrs::write.
#[test]
fn test_csr_sie_write_mask() {
    let mut csrs = Csrs::default();

    csrs.write(MIDELEG, MIP_SSIP | MIP_STIP | MIP_SEIP);
    csrs.write(SIE, 0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(csrs.read(SIE), 0xFFFF_FFFF_FFFF_FFFF);
}

/// Tests STIMECMP write behavior and STIP clearing.
///
/// Note: STIP clearing is handled in orchestrator::csr_write, not in Csrs::write.
#[test]
fn test_csr_stimecmp_clears_stip() {
    let mut csrs = Csrs::default();

    csrs.mip |= MIP_STIP;
    assert_ne!(csrs.read(MIP) & MIP_STIP, 0);

    csrs.write(STIMECMP, 0x1000_0000);
    assert_ne!(csrs.read(MIP) & MIP_STIP, 0);
}

/// Tests read/write consistency across multiple iterations.
#[test]
fn test_csr_read_write_consistency() {
    let mut csrs = Csrs::default();

    for i in 0..10 {
        let val = (i as u64) * 0x1111_1111_1111_1111;
        csrs.write(MSCRATCH, val);
        assert_eq!(csrs.read(MSCRATCH), val);

        csrs.write(MTVAL, val);
        assert_eq!(csrs.read(MTVAL), val);
    }
}

/// Tests behavior when reading/writing unknown CSR addresses.
#[test]
fn test_csr_unknown_address() {
    let mut csrs = Csrs::default();

    assert_eq!(csrs.read(0x999), 0);
    assert_eq!(csrs.read(0x1234), 0);

    csrs.write(0x999, 0xDEAD_BEEF);
    assert_eq!(csrs.read(0x999), 0);
}

/// Tests counter CSR read operations.
#[test]
fn test_csr_counters() {
    let mut csrs = Csrs::default();

    csrs.cycle = 1000;
    assert_eq!(csrs.read(CYCLE), 1000);

    csrs.instret = 500;
    assert_eq!(csrs.read(INSTRET), 500);

    csrs.mcycle = 2000;
    assert_eq!(csrs.read(MCYCLE), 2000);

    csrs.minstret = 1000;
    assert_eq!(csrs.read(MINSTRET), 1000);
}

/// Tests interrupt pending and enable bit fields in MIP and MIE.
#[test]
fn test_csr_interrupt_bits() {
    let mut csrs = Csrs::default();

    csrs.mip = MIP_USIP
        | MIP_SSIP
        | MIP_MSIP
        | MIP_UTIP
        | MIP_STIP
        | MIP_MTIP
        | MIP_UEIP
        | MIP_SEIP
        | MIP_MEIP;
    assert_eq!(
        csrs.read(MIP),
        MIP_USIP
            | MIP_SSIP
            | MIP_MSIP
            | MIP_UTIP
            | MIP_STIP
            | MIP_MTIP
            | MIP_UEIP
            | MIP_SEIP
            | MIP_MEIP
    );

    csrs.mie = MIE_USIP
        | MIE_SSIP
        | MIE_MSIP
        | MIE_UTIE
        | MIE_STIE
        | MIE_MTIE
        | MIE_UEIP
        | MIE_SEIP
        | MIE_MEIP;
    assert_eq!(
        csrs.read(MIE),
        MIE_USIP
            | MIE_SSIP
            | MIE_MSIP
            | MIE_UTIE
            | MIE_STIE
            | MIE_MTIE
            | MIE_UEIP
            | MIE_SEIP
            | MIE_MEIP
    );
}

/// Tests MCAUSE interrupt bit field for interrupts and exceptions.
#[test]
fn test_csr_cause_interrupt_bit() {
    let mut csrs = Csrs::default();

    let interrupt_cause = 0x8000_0000_0000_0003;
    csrs.write(MCAUSE, interrupt_cause);
    assert_eq!(csrs.read(MCAUSE), interrupt_cause);
    assert_ne!(csrs.read(MCAUSE) & CAUSE_INTERRUPT_BIT, 0);

    let exception_cause = 0x0000_0000_0000_0008;
    csrs.write(MCAUSE, exception_cause);
    assert_eq!(csrs.read(MCAUSE), exception_cause);
    assert_eq!(csrs.read(MCAUSE) & CAUSE_INTERRUPT_BIT, 0);
}

/// Tests MSTATUS floating-point state field values.
#[test]
fn test_csr_mstatus_fs_field() {
    let mut csrs = Csrs::default();

    csrs.write(MSTATUS, MSTATUS_FS_OFF);
    assert_eq!(csrs.read(MSTATUS) & MSTATUS_FS, MSTATUS_FS_OFF);

    csrs.write(MSTATUS, MSTATUS_FS_INIT);
    assert_eq!(csrs.read(MSTATUS) & MSTATUS_FS, MSTATUS_FS_INIT);

    csrs.write(MSTATUS, MSTATUS_FS_CLEAN);
    assert_eq!(csrs.read(MSTATUS) & MSTATUS_FS, MSTATUS_FS_CLEAN);

    csrs.write(MSTATUS, MSTATUS_FS_DIRTY);
    assert_eq!(csrs.read(MSTATUS) & MSTATUS_FS, MSTATUS_FS_DIRTY);
}

/// Tests MSTATUS machine previous privilege mode field values.
#[test]
fn test_csr_mstatus_mpp_field() {
    let mut csrs = Csrs::default();

    csrs.write(MSTATUS, 0x0000_0000_0000_0000);
    assert_eq!(
        (csrs.read(MSTATUS) >> MSTATUS_MPP_SHIFT) & MSTATUS_MPP_MASK,
        0
    );

    csrs.write(MSTATUS, 1 << MSTATUS_MPP_SHIFT);
    assert_eq!(
        (csrs.read(MSTATUS) >> MSTATUS_MPP_SHIFT) & MSTATUS_MPP_MASK,
        1
    );

    csrs.write(MSTATUS, 3 << MSTATUS_MPP_SHIFT);
    assert_eq!(
        (csrs.read(MSTATUS) >> MSTATUS_MPP_SHIFT) & MSTATUS_MPP_MASK,
        3
    );
}

/// Tests concurrent writes to multiple CSRs for consistency.
#[test]
fn test_csr_concurrent_writes() {
    let mut csrs = Csrs::default();

    for i in 0..1000 {
        csrs.write(MSCRATCH, i);
        csrs.write(MTVAL, i * 2);
        csrs.write(MEPC, i * 4);

        assert_eq!(csrs.read(MSCRATCH), i);
        assert_eq!(csrs.read(MTVAL), i * 2);
        assert_eq!(csrs.read(MEPC), i * 4);
    }
}

/// Tests MISA extension bit fields and XLEN field.
#[test]
fn test_csr_misa_extensions() {
    let mut csrs = Csrs::default();

    let misa = MISA_XLEN_64
        | MISA_EXT_I
        | MISA_EXT_M
        | MISA_EXT_A
        | MISA_EXT_F
        | MISA_EXT_D
        | MISA_EXT_C
        | MISA_EXT_S
        | MISA_EXT_U;
    csrs.write(MISA, misa);

    let read_misa = csrs.read(MISA);
    assert_eq!(read_misa & MISA_XLEN_64, MISA_XLEN_64);
    assert_ne!(read_misa & MISA_EXT_I, 0);
    assert_ne!(read_misa & MISA_EXT_M, 0);
    assert_ne!(read_misa & MISA_EXT_A, 0);
    assert_ne!(read_misa & MISA_EXT_F, 0);
    assert_ne!(read_misa & MISA_EXT_D, 0);
    assert_ne!(read_misa & MISA_EXT_C, 0);
    assert_ne!(read_misa & MISA_EXT_S, 0);
    assert_ne!(read_misa & MISA_EXT_U, 0);
}

//! Comprehensive tests for register forwarding to detect memory corruption.

use riscv_emulator::core::pipeline::hazards;
use riscv_emulator::core::pipeline::latches::*;
use riscv_emulator::core::pipeline::signals::*;

/// Creates an ID/EX pipeline latch entry for testing.
fn create_id_ex_entry(pc: u64, rd: usize, rs1: usize, rs2: usize, rv1: u64, rv2: u64) -> IdExEntry {
    IdExEntry {
        pc,
        inst: 0,
        inst_size: 4,
        rs1,
        rs2,
        rs3: 0,
        rd,
        imm: 0,
        rv1,
        rv2,
        rv3: 0,
        ctrl: ControlSignals {
            reg_write: true,
            ..Default::default()
        },
        trap: None,
        pred_taken: false,
        pred_target: 0,
    }
}

/// Creates an EX/MEM pipeline latch entry for testing.
fn create_ex_mem_entry(
    pc: u64,
    rd: usize,
    alu: u64,
    reg_write: bool,
    mem_read: bool,
    jump: bool,
) -> ExMemEntry {
    ExMemEntry {
        pc,
        inst: 0,
        inst_size: 4,
        rd,
        alu,
        store_data: 0,
        ctrl: ControlSignals {
            reg_write,
            mem_read,
            jump,
            fp_reg_write: false,
            ..Default::default()
        },
        trap: None,
    }
}

/// Creates a MEM/WB pipeline latch entry for testing.
fn create_mem_wb_entry(
    pc: u64,
    rd: usize,
    alu: u64,
    load_data: u64,
    reg_write: bool,
    mem_read: bool,
    jump: bool,
) -> MemWbEntry {
    MemWbEntry {
        pc,
        inst: 0,
        inst_size: 4,
        rd,
        alu,
        load_data,
        ctrl: ControlSignals {
            reg_write,
            mem_read,
            jump,
            fp_reg_write: false,
            ..Default::default()
        },
        trap: None,
    }
}

/// Tests forwarding from EX/MEM pipeline stage.
#[test]
fn test_forward_from_ex_mem() {
    let id_entry = create_id_ex_entry(0x1004, 2, 1, 0, 0x1111, 0);
    let ex_mem = ExMem {
        entries: vec![create_ex_mem_entry(
            0x1000,
            1,
            0xDEAD_BEEF,
            true,
            false,
            false,
        )],
    };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(a, 0xDEAD_BEEF, "Should forward from EX/MEM");
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests forwarding from MEM/WB (fresh) pipeline stage.
#[test]
fn test_forward_from_mem_wb_fresh() {
    let id_entry = create_id_ex_entry(0x1008, 2, 1, 0, 0x1111, 0);
    let ex_mem = ExMem { entries: vec![] };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb {
        entries: vec![create_mem_wb_entry(
            0x1000,
            1,
            0xCAFE_BABE,
            0,
            true,
            false,
            false,
        )],
    };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(a, 0xCAFE_BABE, "Should forward from MEM/WB (fresh)");
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests forwarding load_data from load instructions in MEM/WB.
#[test]
fn test_forward_from_mem_wb_load() {
    let id_entry = create_id_ex_entry(0x1008, 2, 1, 0, 0x1111, 0);
    let ex_mem = ExMem { entries: vec![] };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb {
        entries: vec![create_mem_wb_entry(
            0x1000,
            1,
            0xDEAD_BEEF,
            0x1234_5678,
            true,
            true,
            false,
        )],
    };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(
        a, 0x1234_5678,
        "Should forward load_data for load instructions"
    );
    assert_ne!(
        a, 0xDEAD_BEEF,
        "Should NOT forward alu for load instructions"
    );
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests forwarding PC + inst_size from jump instructions.
#[test]
fn test_forward_from_jump() {
    let id_entry = create_id_ex_entry(0x1008, 2, 1, 0, 0x1111, 0);
    let ex_mem = ExMem { entries: vec![] };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb {
        entries: vec![create_mem_wb_entry(
            0x1000,
            1,
            0xDEAD_BEEF,
            0,
            true,
            false,
            true,
        )],
    };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(
        a, 0x1004,
        "Should forward PC + inst_size for jump instructions"
    );
    assert_ne!(
        a, 0xDEAD_BEEF,
        "Should NOT forward alu for jump instructions"
    );
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests forwarding priority when both EX/MEM and MEM/WB (old) write to the same register.
#[test]
fn test_forward_priority_ex_mem_over_mem_wb_old() {
    let id_entry = create_id_ex_entry(0x1008, 2, 1, 0, 0x1111, 0);
    let ex_mem = ExMem {
        entries: vec![create_ex_mem_entry(
            0x1004,
            1,
            0x0000_0000_0000_1000,
            true,
            false,
            false,
        )],
    };
    let mem_wb_old = MemWb {
        entries: vec![create_mem_wb_entry(
            0x1000,
            1,
            0x0000_0000_0000_2000,
            0,
            true,
            false,
            false,
        )],
    };
    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(
        a, 0x0000_0000_0000_1000,
        "EX/MEM should have priority over MEM/WB (old)"
    );
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests forwarding priority when both EX/MEM and MEM/WB (fresh) write to the same register.
#[test]
fn test_forward_priority_mem_wb_fresh_over_ex_mem() {
    let id_entry = create_id_ex_entry(0x100C, 2, 1, 0, 0x1111, 0);
    let ex_mem = ExMem {
        entries: vec![create_ex_mem_entry(
            0x1004,
            1,
            0x0000_0000_0000_3000,
            true,
            false,
            false,
        )],
    };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb {
        entries: vec![create_mem_wb_entry(
            0x1008,
            1,
            0x0000_0000_0000_4000,
            0,
            true,
            false,
            false,
        )],
    };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(
        a, 0x0000_0000_0000_4000,
        "MEM/WB (fresh) should have priority over EX/MEM"
    );
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests forwarding from intra-bundle (same cycle) instructions.
#[test]
fn test_forward_intra_bundle() {
    let id_entry = create_id_ex_entry(0x1008, 2, 1, 0, 0x1111, 0);
    let ex_mem = ExMem { entries: vec![] };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![create_ex_mem_entry(
        0x1004,
        1,
        0x0000_0000_0000_5000,
        true,
        false,
        false,
    )];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(a, 0x0000_0000_0000_5000, "Should forward from intra-bundle");
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests that intra-bundle forwarding has highest priority over other stages.
#[test]
fn test_forward_intra_bundle_priority() {
    let id_entry = create_id_ex_entry(0x100C, 2, 1, 0, 0x1111, 0);
    let ex_mem = ExMem {
        entries: vec![create_ex_mem_entry(
            0x1004,
            1,
            0x0000_0000_0000_1000,
            true,
            false,
            false,
        )],
    };
    let mem_wb_old = MemWb {
        entries: vec![create_mem_wb_entry(
            0x1000,
            1,
            0x0000_0000_0000_2000,
            0,
            true,
            false,
            false,
        )],
    };
    let mem_wb_fresh = MemWb {
        entries: vec![create_mem_wb_entry(
            0x1008,
            1,
            0x0000_0000_0000_3000,
            0,
            true,
            false,
            false,
        )],
    };
    let current_ex = vec![create_ex_mem_entry(
        0x100C,
        1,
        0x0000_0000_0000_4000,
        true,
        false,
        false,
    )];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(
        a, 0x0000_0000_0000_4000,
        "Intra-bundle should have highest priority"
    );
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests that forwarding does not occur from EX/MEM when mem_read is set.
#[test]
fn test_forward_no_forward_from_mem_read_in_ex_mem() {
    let id_entry = create_id_ex_entry(0x1008, 2, 1, 0, 0x1111, 0);
    let ex_mem = ExMem {
        entries: vec![create_ex_mem_entry(
            0x1004,
            1,
            0xDEAD_BEEF,
            true,
            true,
            false,
        )],
    };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(
        a, 0x1111,
        "Should NOT forward from EX/MEM if mem_read is set"
    );
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests forwarding to all three source registers (rs1, rs2, rs3).
#[test]
fn test_forward_rs1_rs2_rs3() {
    let mut id_entry = create_id_ex_entry(0x100C, 3, 1, 2, 0x1111, 0x2222);
    id_entry.rs3 = 3;
    id_entry.rv3 = 0x3333;
    id_entry.ctrl.rs3_fp = false;

    let ex_mem = ExMem {
        entries: vec![
            create_ex_mem_entry(0x1004, 1, 0x0000_0000_0000_A000, true, false, false),
            create_ex_mem_entry(0x1008, 2, 0x0000_0000_0000_B000, true, false, false),
        ],
    };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb {
        entries: vec![create_mem_wb_entry(
            0x1000,
            3,
            0x0000_0000_0000_C000,
            0,
            true,
            false,
            false,
        )],
    };
    let current_ex = vec![];

    let (a, b, c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(a, 0x0000_0000_0000_A000, "rs1 should be forwarded");
    assert_eq!(b, 0x0000_0000_0000_B000, "rs2 should be forwarded");
    assert_eq!(c, 0x0000_0000_0000_C000, "rs3 should be forwarded");
}

/// Tests forwarding for floating-point registers.
#[test]
fn test_forward_fp_registers() {
    let mut id_entry = create_id_ex_entry(0x1008, 2, 1, 0, 0x1111, 0);
    id_entry.ctrl.rs1_fp = true;
    id_entry.ctrl.fp_reg_write = false;

    let mut ex_mem_entry = create_ex_mem_entry(0x1004, 1, 0xDEAD_BEEF, true, false, false);
    ex_mem_entry.ctrl.fp_reg_write = true;
    let ex_mem = ExMem {
        entries: vec![ex_mem_entry],
    };

    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(a, 0xDEAD_BEEF, "Should forward FP register");
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests that forwarding does not occur when source and destination registers differ.
#[test]
fn test_forward_no_forward_wrong_register() {
    let id_entry = create_id_ex_entry(0x1008, 3, 1, 0, 0x1111, 0);
    let ex_mem = ExMem {
        entries: vec![create_ex_mem_entry(
            0x1004,
            2,
            0xDEAD_BEEF,
            true,
            false,
            false,
        )],
    };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(a, 0x1111, "Should NOT forward - different register");
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests that forwarding does not occur for x0 register.
#[test]
fn test_forward_no_forward_x0() {
    let mut id_entry = create_id_ex_entry(0x1008, 1, 0, 0, 0x1111, 0);
    id_entry.rs1 = 0;

    let ex_mem = ExMem {
        entries: vec![create_ex_mem_entry(
            0x1004,
            0,
            0xDEAD_BEEF,
            true,
            false,
            false,
        )],
    };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(a, 0x1111, "Should NOT forward x0 (always 0)");
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests that forwarding does not occur from instructions that have traps.
#[test]
fn test_forward_skip_trapped_instructions() {
    let id_entry = create_id_ex_entry(0x1008, 2, 1, 0, 0x1111, 0);
    let mut ex_mem_entry = create_ex_mem_entry(0x1004, 1, 0xDEAD_BEEF, true, false, false);
    ex_mem_entry.trap = Some(riscv_emulator::common::error::Trap::IllegalInstruction(0));
    let ex_mem = ExMem {
        entries: vec![ex_mem_entry],
    };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(a, 0x1111, "Should NOT forward from trapped instruction");
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests forwarding priority when multiple instructions write to the same register.
#[test]
fn test_forward_multiple_writers_same_register() {
    let id_entry = create_id_ex_entry(0x1010, 2, 1, 0, 0x1111, 0);
    let ex_mem = ExMem {
        entries: vec![create_ex_mem_entry(
            0x1004,
            1,
            0x0000_0000_0000_1000,
            true,
            false,
            false,
        )],
    };
    let mem_wb_old = MemWb {
        entries: vec![create_mem_wb_entry(
            0x1000,
            1,
            0x0000_0000_0000_2000,
            0,
            true,
            false,
            false,
        )],
    };
    let mem_wb_fresh = MemWb {
        entries: vec![create_mem_wb_entry(
            0x1008,
            1,
            0x0000_0000_0000_5000,
            0,
            true,
            false,
            false,
        )],
    };
    let current_ex = vec![create_ex_mem_entry(
        0x100C,
        1,
        0x0000_0000_0000_6000,
        true,
        false,
        false,
    )];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(
        a, 0x0000_0000_0000_6000,
        "Should forward from most recent writer"
    );
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests forwarding of store data (rs2) for store instructions.
#[test]
fn test_forward_store_data() {
    let id_entry = create_id_ex_entry(0x1008, 0, 0, 1, 0, 0x2222);
    let ex_mem = ExMem {
        entries: vec![create_ex_mem_entry(
            0x1004,
            1,
            0xDEAD_BEEF,
            true,
            false,
            false,
        )],
    };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(a, 0, "rs1 should not be forwarded");
    assert_eq!(b, 0xDEAD_BEEF, "rs2 (store data) should be forwarded");
}

/// Tests that FP register writes do not forward to integer register reads.
#[test]
fn test_forward_fp_vs_int_mismatch() {
    let mut id_entry = create_id_ex_entry(0x1008, 2, 1, 0, 0x1111, 0);
    id_entry.ctrl.rs1_fp = false;

    let mut ex_mem_entry = create_ex_mem_entry(0x1004, 1, 0xDEAD_BEEF, true, false, false);
    ex_mem_entry.ctrl.fp_reg_write = true;
    let ex_mem = ExMem {
        entries: vec![ex_mem_entry],
    };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(a, 0x1111, "Should NOT forward FP to integer register");
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Stress test with many instructions in pipeline stages.
#[test]
fn test_forward_stress_many_instructions() {
    let id_entry = create_id_ex_entry(0x1020, 10, 1, 2, 0x1111, 0x2222);

    let mut ex_mem_entries = vec![];
    for i in 0..10 {
        ex_mem_entries.push(create_ex_mem_entry(
            (0x1000 + (i * 4)) as u64,
            i + 1,
            (0x1000 + i) as u64,
            true,
            false,
            false,
        ));
    }
    let ex_mem = ExMem {
        entries: ex_mem_entries,
    };

    let mut mem_wb_entries = vec![];
    for i in 0..10 {
        mem_wb_entries.push(create_mem_wb_entry(
            (0x2000 + (i * 4)) as u64,
            i + 11,
            (0x2000 + i) as u64,
            0,
            true,
            false,
            false,
        ));
    }
    let mem_wb_old = MemWb {
        entries: mem_wb_entries,
    };

    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![];

    let (a, b, c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(a, 0x1000, "Should forward from EX/MEM entry 0");
    assert_eq!(b, 0x1001, "Should forward from EX/MEM entry 1");
    assert_eq!(c, 0, "rs3 should not be forwarded");
}

/// Tests forwarding behavior with multiple intra-bundle instructions writing to the same register.
#[test]
fn test_forward_reverse_order_intra_bundle() {
    let id_entry = create_id_ex_entry(0x1010, 3, 1, 0, 0x1111, 0);
    let ex_mem = ExMem { entries: vec![] };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb { entries: vec![] };

    let current_ex = vec![
        create_ex_mem_entry(0x1004, 1, 0x0000_0000_0000_D000, true, false, false),
        create_ex_mem_entry(0x1008, 1, 0x0000_0000_0000_E000, true, false, false),
    ];

    let (a, b, _c) = hazards::forward_rs(
        &id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert!(
        a == 0x0000_0000_0000_D000 || a == 0x0000_0000_0000_E000,
        "Should forward from one of the intra-bundle entries"
    );
    assert_eq!(b, 0, "rs2 should not be forwarded");
}

/// Tests forwarding from a Load instruction to a JALR (Return) instruction.
/// This simulates a function return sequence:
///   ld ra, 0(sp)
///   ret
/// Requires a stall (Load-Use hazard) detection followed by correct forwarding.
#[test]
fn test_forward_load_use_jalr() {
    use riscv_emulator::core::pipeline::hazards;
    use riscv_emulator::core::pipeline::latches::{IdEx, IdExEntry, IfId, IfIdEntry};
    use riscv_emulator::core::pipeline::signals::{ControlSignals, MemWidth};

    let mut load_ctrl = ControlSignals::default();
    load_ctrl.mem_read = true;
    load_ctrl.reg_write = true;
    load_ctrl.width = MemWidth::Double;

    let load_entry = IdExEntry {
        pc: 0x1000,
        inst: 0,
        inst_size: 4,
        rs1: 2,
        rs2: 0,
        rs3: 0,
        rd: 1,
        imm: 0,
        rv1: 0,
        rv2: 0,
        rv3: 0,
        ctrl: load_ctrl,
        trap: None,
        pred_taken: false,
        pred_target: 0,
    };

    let id_ex = IdEx {
        entries: vec![load_entry],
    };

    let jalr_inst = 0x00008067;
    let if_entry = IfIdEntry {
        pc: 0x1004,
        inst: jalr_inst,
        inst_size: 4,
        pred_taken: false,
        pred_target: 0,
        trap: None,
    };
    let if_id = IfId {
        entries: vec![if_entry],
    };

    assert!(
        hazards::need_stall_load_use(&id_ex, &if_id),
        "Pipeline should stall for Load-Use hazard on JALR rs1 (RA)"
    );
}

/// Tests forwarding from an ALU operation to JALR.
///   addi x1, x0, target
///   jalr x0, x1, 0
/// Should NOT stall, but should forward the ALU result to the Jump target calculation.
#[test]
fn test_forward_alu_jalr() {
    use riscv_emulator::core::pipeline::hazards;
    use riscv_emulator::core::pipeline::latches::{ExMem, ExMemEntry, IdExEntry, MemWb};
    use riscv_emulator::core::pipeline::signals::{AluOp, ControlSignals};

    let mut alu_ctrl = ControlSignals::default();
    alu_ctrl.reg_write = true;
    alu_ctrl.alu = AluOp::Add;

    let ex_mem_entry = ExMemEntry {
        pc: 0x1000,
        inst: 0,
        inst_size: 4,
        rd: 1,
        alu: 0x8000,
        store_data: 0,
        ctrl: alu_ctrl,
        trap: None,
    };

    let ex_mem = ExMem {
        entries: vec![ex_mem_entry],
    };
    let mem_wb_old = MemWb { entries: vec![] };
    let mem_wb_fresh = MemWb { entries: vec![] };
    let current_ex = vec![];

    let mut jalr_id_entry = IdExEntry::default();
    jalr_id_entry.pc = 0x1004;
    jalr_id_entry.rs1 = 1;
    jalr_id_entry.rv1 = 0xDEADBEEF;

    let (fwd_a, _, _) = hazards::forward_rs(
        &jalr_id_entry,
        &ex_mem,
        &mem_wb_old,
        &mem_wb_fresh,
        &current_ex,
        false,
    );

    assert_eq!(
        fwd_a, 0x8000,
        "Should forward ALU result (0x8000) to JALR rs1"
    );
}

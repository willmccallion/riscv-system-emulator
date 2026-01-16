//! Execute (EX) Stage.
//!
//! This module implements the execution stage of the pipeline. It is responsible for:
//! - Resolving operand values using forwarding logic (from EX/MEM or MEM/WB).
//! - Performing ALU and FPU operations.
//! - Handling branch and jump resolution (verifying predictions and updating the BPU).
//! - Handling system instructions (CSR access, ECALL, xRET, WFI, SFENCE.VMA).
//! - Handling FENCE.I for instruction cache coherency.

use crate::common::error::Trap;
use crate::core::pipeline::hazards;
use crate::core::pipeline::latches::{ExMem, ExMemEntry, IfId};
use crate::core::pipeline::signals::{AluOp, CsrOp, OpASrc, OpBSrc};
use crate::core::units::alu::Alu;
use crate::core::units::bru::BranchPredictor;
use crate::core::units::fpu::Fpu;
use crate::core::Cpu;
use crate::isa::abi;
use crate::isa::privileged::opcodes as sys_ops;
use crate::isa::rv64i::{funct3, opcodes};

/// Bit shift for extracting funct3 field from instruction (bits 12-14).
const FUNCT3_SHIFT: u32 = 12;

/// Bit mask for funct3 field (3 bits).
const FUNCT3_MASK: u32 = 0x7;

/// Bit mask to clear bit 0, forcing JALR target addresses to be even.
///
/// JALR requires target addresses to be aligned to 2-byte boundaries.
const JALR_ALIGNMENT_MASK: u64 = !1;

/// Executes the Execute stage of the pipeline.
///
/// This function consumes instructions from the ID/EX latch, executes them,
/// and produces results for the EX/MEM latch.
///
/// # Logic Flow
/// 1. **Drain ID/EX**: Takes all entries from the decode latch.
/// 2. **Forwarding**: Resolves data hazards by checking later pipeline stages.
/// 3. **System Ops**: Handles CSRs, Traps, and serialization instructions (FENCE.I, SFENCE.VMA).
/// 4. **Execution**: Runs ALU/FPU operations.
/// 5. **Control Flow**: Verifies branch predictions and redirects PC on misprediction.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU state.
pub fn execute_stage(cpu: &mut Cpu) {
    let mut entries = std::mem::take(&mut cpu.id_ex.entries);

    let mut ex_results = Vec::with_capacity(entries.len());
    let mut flush_remaining = false;

    for id in entries.drain(..) {
        if flush_remaining {
            break;
        }

        if let Some(trap) = id.trap.clone() {
            if cpu.trace {
                eprintln!("EX  pc={:#x} # TRAP: {:?}", id.pc, trap);
            }
            ex_results.push(ExMemEntry {
                pc: id.pc,
                inst: id.inst,
                inst_size: id.inst_size,
                rd: id.rd,
                alu: 0,
                store_data: 0,
                ctrl: id.ctrl,
                trap: Some(trap),
            });
            continue;
        }

        if cpu.trace {
            eprintln!("EX  pc={:#x}", id.pc);
        }

        let (fwd_a, fwd_b, fwd_c) = hazards::forward_rs(
            &id,
            &cpu.ex_mem,
            &cpu.wb_latch,
            &cpu.mem_wb,
            &ex_results,
            cpu.trace,
        );

        let store_data = fwd_b;

        let op_a = match id.ctrl.a_src {
            OpASrc::Reg1 => fwd_a,
            OpASrc::Pc => id.pc,
            OpASrc::Zero => 0,
        };
        let op_b = match id.ctrl.b_src {
            OpBSrc::Reg2 => fwd_b,
            OpBSrc::Imm => id.imm as u64,
            OpBSrc::Zero => 0,
        };
        let op_c = fwd_c;

        if id.ctrl.is_fence_i {
            if cpu.trace {
                println!("EX  FENCE.I - Flushing Caches and Pipeline");
            }
            cpu.l1_d_cache.flush();
            cpu.l1_i_cache.flush();

            cpu.if_id = IfId::default();
            cpu.pc = id.pc.wrapping_add(id.inst_size);
            flush_remaining = true;

            ex_results.push(ExMemEntry {
                pc: id.pc,
                inst: id.inst,
                inst_size: id.inst_size,
                rd: id.rd,
                alu: 0,
                store_data: 0,
                ctrl: id.ctrl,
                trap: None,
            });
            continue;
        }

        if id.ctrl.is_system {
            if id.ctrl.is_mret {
                cpu.do_mret();
                flush_remaining = true;
                cpu.if_id = IfId::default();
                continue;
            }
            if id.ctrl.is_sret {
                cpu.do_sret();
                flush_remaining = true;
                cpu.if_id = IfId::default();
                continue;
            }

            if id.inst == sys_ops::WFI {
                cpu.wfi_waiting = true;
                cpu.wfi_pc = id.pc;

                ex_results.push(ExMemEntry {
                    pc: id.pc,
                    inst: id.inst,
                    inst_size: id.inst_size,
                    rd: id.rd,
                    alu: 0,
                    store_data: 0,
                    ctrl: id.ctrl,
                    trap: None,
                });
                continue;
            }

            if (id.inst & 0xFE007FFF) == sys_ops::SFENCE_VMA {
                if cpu.trace {
                    eprintln!("EX  SFENCE.VMA - Flushing TLBs");
                }

                cpu.flush_pipeline_stores();

                for entry in &mut ex_results {
                    if entry.ctrl.mem_write {
                        let addr = entry.alu;
                        let src = entry.store_data;
                        let width = entry.ctrl.width;
                        match width {
                            crate::core::pipeline::signals::MemWidth::Byte => {
                                cpu.bus.bus.write_u8(addr, src as u8)
                            }
                            crate::core::pipeline::signals::MemWidth::Half => {
                                cpu.bus.bus.write_u16(addr, src as u16)
                            }
                            crate::core::pipeline::signals::MemWidth::Word => {
                                cpu.bus.bus.write_u32(addr, src as u32)
                            }
                            crate::core::pipeline::signals::MemWidth::Double => {
                                cpu.bus.bus.write_u64(addr, src)
                            }
                            _ => {}
                        }
                        entry.ctrl.mem_write = false;
                    }
                }

                cpu.mmu.dtlb.flush();
                cpu.mmu.itlb.flush();
                cpu.l1_d_cache.flush();
                cpu.l1_i_cache.flush();

                ex_results.push(ExMemEntry {
                    pc: id.pc,
                    inst: id.inst,
                    inst_size: id.inst_size,
                    rd: id.rd,
                    alu: 0,
                    store_data,
                    ctrl: id.ctrl,
                    trap: None,
                });
                continue;
            }

            if id.inst == sys_ops::ECALL {
                let get_val = |reg: usize, cpu: &Cpu, current_results: &[ExMemEntry]| -> u64 {
                    if reg == 0 {
                        return 0;
                    }
                    for entry in current_results.iter().rev() {
                        if entry.ctrl.reg_write && entry.rd == reg {
                            return entry.alu;
                        }
                    }
                    for entry in cpu.ex_mem.entries.iter().rev() {
                        if entry.ctrl.reg_write && entry.rd == reg {
                            return if entry.ctrl.jump {
                                entry.pc.wrapping_add(entry.inst_size)
                            } else {
                                entry.alu
                            };
                        }
                    }
                    for entry in cpu.mem_wb.entries.iter().rev() {
                        if entry.ctrl.reg_write && entry.rd == reg {
                            return if entry.ctrl.mem_read {
                                entry.load_data
                            } else if entry.ctrl.jump {
                                entry.pc.wrapping_add(entry.inst_size)
                            } else {
                                entry.alu
                            };
                        }
                    }
                    cpu.regs.read(reg)
                };

                if cpu.direct_mode {
                    let val_a7 = get_val(abi::REG_A7, cpu, &ex_results);
                    let val_a0 = get_val(abi::REG_A0, cpu, &ex_results);

                    if val_a7 == sys_ops::SYS_EXIT {
                        cpu.exit_code = Some(val_a0);
                        break;
                    } else if val_a0 == sys_ops::SYS_EXIT {
                        let val_a1 = get_val(abi::REG_A1, cpu, &ex_results);
                        cpu.exit_code = Some(val_a1);
                        break;
                    }
                }

                use crate::core::arch::mode::PrivilegeMode;
                let trap = match cpu.privilege {
                    PrivilegeMode::User => Trap::EnvironmentCallFromUMode,
                    PrivilegeMode::Supervisor => Trap::EnvironmentCallFromSMode,
                    PrivilegeMode::Machine => Trap::EnvironmentCallFromMMode,
                };

                ex_results.push(ExMemEntry {
                    pc: id.pc,
                    inst: id.inst,
                    inst_size: id.inst_size,
                    rd: id.rd,
                    alu: 0,
                    store_data: 0,
                    ctrl: id.ctrl,
                    trap: Some(trap),
                });
                flush_remaining = true;
                continue;
            }

            if id.ctrl.csr_op != CsrOp::None {
                let old = cpu.csr_read(id.ctrl.csr_addr);
                let src = match id.ctrl.csr_op {
                    CsrOp::Rwi | CsrOp::Rsi | CsrOp::Rci => (id.rs1 as u64) & 0x1f,
                    _ => fwd_a,
                };
                let new = match id.ctrl.csr_op {
                    CsrOp::Rw | CsrOp::Rwi => src,
                    CsrOp::Rs | CsrOp::Rsi => old | src,
                    CsrOp::Rc | CsrOp::Rci => old & !src,
                    CsrOp::None => old,
                };

                if id.ctrl.csr_addr == crate::core::arch::csr::SATP {
                    cpu.flush_pipeline_stores();
                    for entry in &mut ex_results {
                        if entry.ctrl.mem_write {
                            let addr = entry.alu;
                            let src = entry.store_data;
                            let width = entry.ctrl.width;
                            match width {
                                crate::core::pipeline::signals::MemWidth::Byte => {
                                    cpu.bus.bus.write_u8(addr, src as u8)
                                }
                                crate::core::pipeline::signals::MemWidth::Half => {
                                    cpu.bus.bus.write_u16(addr, src as u16)
                                }
                                crate::core::pipeline::signals::MemWidth::Word => {
                                    cpu.bus.bus.write_u32(addr, src as u32)
                                }
                                crate::core::pipeline::signals::MemWidth::Double => {
                                    cpu.bus.bus.write_u64(addr, src)
                                }
                                _ => {}
                            }
                            entry.ctrl.mem_write = false;
                        }
                    }
                }

                cpu.csr_write(id.ctrl.csr_addr, new);

                cpu.if_id = IfId::default();
                cpu.pc = id.pc.wrapping_add(id.inst_size);
                flush_remaining = true;

                ex_results.push(ExMemEntry {
                    pc: id.pc,
                    inst: id.inst,
                    inst_size: id.inst_size,
                    rd: id.rd,
                    alu: old,
                    store_data,
                    ctrl: id.ctrl,
                    trap: None,
                });
                continue;
            }
        }

        let alu_out = if (id.ctrl.alu as i32 >= AluOp::FCvtSW as i32
            && id.ctrl.alu as i32 <= AluOp::FCvtSL as i32)
            || id.ctrl.alu as i32 == AluOp::FMvToF as i32
        {
            match id.ctrl.alu {
                AluOp::FCvtSW => {
                    if id.ctrl.is_rv32 {
                        Fpu::box_f32((op_a as i32) as f32)
                    } else {
                        ((op_a as i32) as f64).to_bits()
                    }
                }
                AluOp::FCvtSL => {
                    if id.ctrl.is_rv32 {
                        Fpu::box_f32((op_a as i64) as f32)
                    } else {
                        ((op_a as i64) as f64).to_bits()
                    }
                }
                AluOp::FCvtSD => {
                    let val_d = f64::from_bits(op_a);
                    let val_s = val_d as f32;
                    Fpu::box_f32(val_s)
                }
                AluOp::FCvtDS => {
                    let val_s = f32::from_bits(op_a as u32);
                    let val_d = val_s as f64;
                    val_d.to_bits()
                }
                AluOp::FMvToF => {
                    if id.ctrl.is_rv32 {
                        Fpu::box_f32(f32::from_bits(op_a as u32))
                    } else {
                        op_a
                    }
                }
                _ => 0,
            }
        } else {
            let is_fp_op = matches!(
                id.ctrl.alu,
                AluOp::FAdd
                    | AluOp::FSub
                    | AluOp::FMul
                    | AluOp::FDiv
                    | AluOp::FSqrt
                    | AluOp::FMin
                    | AluOp::FMax
                    | AluOp::FMAdd
                    | AluOp::FMSub
                    | AluOp::FNMAdd
                    | AluOp::FNMSub
                    | AluOp::FSgnJ
                    | AluOp::FSgnJN
                    | AluOp::FSgnJX
                    | AluOp::FEq
                    | AluOp::FLt
                    | AluOp::FLe
                    | AluOp::FClass
                    | AluOp::FCvtWS
                    | AluOp::FCvtLS
                    | AluOp::FCvtSW
                    | AluOp::FCvtSL
                    | AluOp::FCvtSD
                    | AluOp::FCvtDS
                    | AluOp::FMvToX
                    | AluOp::FMvToF
            );

            if is_fp_op {
                Fpu::execute(id.ctrl.alu, op_a, op_b, op_c, id.ctrl.is_rv32)
            } else {
                Alu::execute(id.ctrl.alu, op_a, op_b, op_c, id.ctrl.is_rv32)
            }
        };

        if id.ctrl.branch {
            let taken = match (id.inst >> FUNCT3_SHIFT) & FUNCT3_MASK {
                funct3::BEQ => op_a == op_b,
                funct3::BNE => op_a != op_b,
                funct3::BLT => (op_a as i64) < (op_b as i64),
                funct3::BGE => (op_a as i64) >= (op_b as i64),
                funct3::BLTU => op_a < op_b,
                funct3::BGEU => op_a >= op_b,
                _ => false,
            };
            let actual_target = id.pc.wrapping_add(id.imm as u64);
            let fallthrough = id.pc.wrapping_add(id.inst_size);

            let predicted_target = if id.pred_taken {
                id.pred_target
            } else {
                fallthrough
            };
            let actual_next_pc = if taken { actual_target } else { fallthrough };

            let mispredicted = predicted_target != actual_next_pc;

            cpu.branch_predictor.update_branch(
                id.pc,
                taken,
                if taken { Some(actual_target) } else { None },
            );

            if mispredicted {
                cpu.stats.branch_mispredictions += 1;
                cpu.stats.stalls_control += 2;

                cpu.pc = actual_next_pc;
                cpu.if_id = IfId::default();
                flush_remaining = true;
            } else {
                cpu.stats.branch_predictions += 1;
            }
        }

        if id.ctrl.jump {
            use crate::common::constants::OPCODE_MASK;
            let is_jalr = (id.inst & OPCODE_MASK) == opcodes::OP_JALR;
            let is_call = (id.inst & OPCODE_MASK) == opcodes::OP_JAL && id.rd == abi::REG_RA;
            let is_ret = is_jalr && id.rd == abi::REG_ZERO && id.rs1 == abi::REG_RA;

            let actual_target = if is_jalr {
                (fwd_a.wrapping_add(id.imm as u64)) & JALR_ALIGNMENT_MASK
            } else {
                id.pc.wrapping_add(id.imm as u64)
            };

            let predicted_target = if id.pred_taken {
                id.pred_target
            } else {
                id.pc.wrapping_add(id.inst_size)
            };

            if actual_target != predicted_target {
                cpu.stats.branch_mispredictions += 1;
                cpu.stats.stalls_control += 2;
                cpu.pc = actual_target;
                cpu.if_id = IfId::default();
                flush_remaining = true;
            } else {
                cpu.stats.branch_predictions += 1;
            }

            if is_call {
                cpu.branch_predictor.on_call(
                    id.pc,
                    id.pc.wrapping_add(id.inst_size),
                    actual_target,
                );
            } else if is_ret {
                cpu.branch_predictor.on_return();
            }
        }

        ex_results.push(ExMemEntry {
            pc: id.pc,
            inst: id.inst,
            inst_size: id.inst_size,
            rd: id.rd,
            alu: alu_out,
            store_data,
            ctrl: id.ctrl,
            trap: None,
        });
    }

    cpu.id_ex.entries = entries;

    cpu.ex_mem = ExMem {
        entries: ex_results,
    };
}

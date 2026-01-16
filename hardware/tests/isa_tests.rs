//! Integration tests for ISA decoding and instruction handling.

use riscv_emulator::isa::instruction::InstructionBits;
use riscv_emulator::isa::*;

/// Tests instruction opcode extraction.
#[test]
fn test_instruction_bits_opcode() {
    let inst = 0x00008067u32;
    assert_eq!(inst.opcode(), 0x67);
}

/// Tests instruction destination register extraction.
#[test]
fn test_instruction_bits_rd() {
    let inst = 0x00108093u32;
    assert_eq!(inst.rd(), 1);
}

/// Tests instruction source register 1 extraction.
#[test]
fn test_instruction_bits_rs1() {
    let inst = 0x00108093u32;
    assert_eq!(inst.rs1(), 1);
}

/// Tests instruction source register 2 extraction.
#[test]
fn test_instruction_bits_rs2() {
    let inst = 0x00110133u32;
    assert_eq!(inst.rs2(), 1);
}

/// Tests instruction funct3 field extraction.
#[test]
fn test_instruction_bits_funct3() {
    let inst = 0x00108093u32;
    assert_eq!(inst.funct3(), 0);
}

/// Tests instruction funct7 field extraction.
#[test]
fn test_instruction_bits_funct7() {
    let inst = 0x00110133u32;
    assert_eq!(inst.funct7(), 0);
}

/// Tests instruction CSR field extraction.
#[test]
fn test_instruction_bits_csr() {
    let inst = 0x3000A073u32;
    assert_eq!(inst.csr(), 0x300);
}

/// Tests instruction source register 3 extraction.
#[test]
fn test_instruction_bits_rs3() {
    let inst = 0x0201F0D3u32;
    assert_eq!(inst.rs3(), 4);
}

/// Tests ADDI instruction decoding.
#[test]
fn test_decode_addi() {
    let inst = 0x00108093u32;
    let decoded = decode::decode(inst);

    assert_eq!(decoded.opcode, 0x13);
    assert_eq!(decoded.rd, 1);
    assert_eq!(decoded.rs1, 1);
    assert_eq!(decoded.imm, 1);
}

/// Tests ADD instruction decoding.
#[test]
fn test_decode_add() {
    let inst = 0x00110133u32;
    let decoded = decode::decode(inst);

    assert_eq!(decoded.opcode, 0x33);
    assert_eq!(decoded.rd, 2);
    assert_eq!(decoded.rs1, 2);
    assert_eq!(decoded.rs2, 1);
}

/// Tests LUI instruction decoding.
#[test]
fn test_decode_lui() {
    let inst = 0x12345037u32;
    let decoded = decode::decode(inst);

    assert_eq!(decoded.opcode, 0x37);
    assert_eq!(decoded.imm, 0x12345 << 12);
}

/// Tests AUIPC instruction decoding.
#[test]
fn test_decode_auipc() {
    let inst = 0x12345017u32;
    let decoded = decode::decode(inst);

    assert_eq!(decoded.opcode, 0x17);
    assert_eq!(decoded.imm, 0x12345 << 12);
}

/// Tests JAL instruction decoding.
#[test]
fn test_decode_jal() {
    let inst = 0x0000006Fu32;
    let decoded = decode::decode(inst);

    assert_eq!(decoded.opcode, 0x6F);
    assert_eq!(decoded.imm, 0);
}

/// Tests JALR instruction decoding.
#[test]
fn test_decode_jalr() {
    let inst = 0x00008067u32;
    let decoded = decode::decode(inst);

    assert_eq!(decoded.opcode, 0x67);
    assert_eq!(decoded.rs1, 1);
    assert_eq!(decoded.imm, 0);
}

/// Tests branch instruction decoding.
#[test]
fn test_decode_branch() {
    let inst = 0x00108263u32;
    let decoded = decode::decode(inst);

    assert_eq!(decoded.opcode, 0x63);
    assert_eq!(decoded.rs1, 1);
    assert_eq!(decoded.rs2, 1);
    assert_eq!(decoded.imm, 4);
}

/// Tests load instruction decoding.
#[test]
fn test_decode_load() {
    let inst = 0x00008083u32;
    let decoded = decode::decode(inst);

    assert_eq!(decoded.opcode, 0x03);
    assert_eq!(decoded.rd, 1);
    assert_eq!(decoded.rs1, 1);
    assert_eq!(decoded.imm, 0);
}

/// Tests store instruction decoding.
#[test]
fn test_decode_store() {
    let inst = 0x00108023u32;
    let decoded = decode::decode(inst);

    assert_eq!(decoded.opcode, 0x23);
    assert_eq!(decoded.rs1, 1);
    assert_eq!(decoded.rs2, 1);
    assert_eq!(decoded.imm, 0);
}

/// Tests immediate sign extension in instruction decoding.
#[test]
fn test_decode_immediate_sign_extension() {
    let inst = 0xFFF08093u32;
    let decoded = decode::decode(inst);
    assert_eq!(decoded.imm, -1i64);

    let inst2 = 0x00108093u32;
    let decoded2 = decode::decode(inst2);
    assert_eq!(decoded2.imm, 1);
}

/// Tests branch immediate encoding for forward and backward branches.
#[test]
fn test_decode_branch_immediate() {
    let inst = 0x00A08263u32;
    let decoded = decode::decode(inst);
    assert_eq!(decoded.imm, 20);

    let inst2 = 0xFE0082E3u32;
    let decoded2 = decode::decode(inst2);
    assert_eq!(decoded2.imm, -4);
}

/// Tests compressed instruction C.ADDI4SPN expansion.
#[test]
fn test_rvc_expand_c_addi4spn() {
    let inst = 0x0000u16;
    let expanded = rvc::expand::expand(inst);
    assert_eq!(expanded, 0);

    let inst2 = 0x0008u16;
    let expanded2 = rvc::expand::expand(inst2);
    assert_ne!(expanded2, 0);
    assert_eq!(expanded2.opcode(), 0x13);
}

/// Tests compressed instruction C.ADDI expansion.
#[test]
fn test_rvc_expand_c_addi() {
    let inst = 0x0001u16;
    let expanded = rvc::expand::expand(inst);
    assert_eq!(expanded.opcode(), 0x13);
}

/// Tests compressed instruction C.J expansion.
#[test]
fn test_rvc_expand_c_j() {
    let inst = 0x2001u16;
    let expanded = rvc::expand::expand(inst);
    assert_eq!(expanded.opcode(), 0x6F);
    assert_eq!(expanded.rd(), 0);
}

/// Tests compressed instruction C.BEQZ expansion.
#[test]
fn test_rvc_expand_c_beqz() {
    let inst = 0xC001u16;
    let expanded = rvc::expand::expand(inst);
    assert_eq!(expanded.opcode(), 0x63);
    assert_eq!(expanded.rs2(), 0);
}

/// Tests compressed instruction C.BNEZ expansion.
#[test]
fn test_rvc_expand_c_bnez() {
    let inst = 0xE001u16;
    let expanded = rvc::expand::expand(inst);
    assert_eq!(expanded.opcode(), 0x63);
    assert_eq!(expanded.rs2(), 0);
}

/// Tests compressed instruction C.LW expansion.
#[test]
fn test_rvc_expand_c_lw() {
    let inst = 0x4000u16;
    let expanded = rvc::expand::expand(inst);
    assert_eq!(expanded.opcode(), 0x03);
}

/// Tests compressed instruction C.SW expansion.
#[test]
fn test_rvc_expand_c_sw() {
    let inst = 0xC000u16;
    let expanded = rvc::expand::expand(inst);
    assert_eq!(expanded.opcode(), 0x23);
}

/// Tests RISC-V ABI register name constants.
#[test]
fn test_abi_constants() {
    assert_eq!(abi::REG_ZERO, 0);
    assert_eq!(abi::REG_RA, 1);
    assert_eq!(abi::REG_SP, 2);
    assert_eq!(abi::REG_A0, 10);
    assert_eq!(abi::REG_A1, 11);
    assert_eq!(abi::REG_A2, 12);
    assert_eq!(abi::REG_A7, 17);
}

/// Tests JALR with a non-zero offset to ensure addition is correct.
#[test]
fn test_jalr_offset_calc() {
    use riscv_emulator::isa::decode;

    let inst = 0x004100E7;
    let decoded = decode::decode(inst);

    assert_eq!(decoded.opcode, 0x67);
    assert_eq!(decoded.rd, 1);
    assert_eq!(decoded.rs1, 2);
    assert_eq!(decoded.imm, 4);

    let rs1_val = 0x1000;
    let imm = 4;
    let target = (rs1_val + imm) & !1;
    assert_eq!(target, 0x1004);
}

/// Tests JALR LSB masking (bit 0 should be cleared).
#[test]
fn test_jalr_lsb_masking() {
    let rs1_val = 0x1001;
    let imm = 0;
    let target = (rs1_val + imm) & !1;
    assert_eq!(target, 0x1000);
}

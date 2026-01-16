//! Pipeline control signals and operation types.
//!
//! This module defines the control signals that flow through the pipeline
//! to control instruction execution, including ALU operations, memory
//! operations, CSR operations, and operand source selection.

/// ALU operation types for integer and floating-point instructions.
///
/// Specifies the operation to be performed by the ALU or FPU unit.
/// Includes all RISC-V integer operations (I, M extensions) and
/// floating-point operations (F, D extensions).
#[derive(Clone, Copy, Debug, Default)]
pub enum AluOp {
    /// Default value (no operation).
    #[default]
    Add,
    /// Integer addition.
    Sub,
    /// Integer subtraction.
    Sll,
    /// Shift left logical.
    Slt,
    /// Set less than (signed).
    Sltu,
    /// Set less than unsigned.
    Xor,
    /// Bitwise XOR.
    Srl,
    /// Shift right logical.
    Sra,
    /// Shift right arithmetic.
    Or,
    /// Bitwise OR.
    And,
    /// Bitwise AND.
    Mul,
    /// Integer multiply (low bits).
    Mulh,
    /// Integer multiply (high bits, signed × signed).
    Mulhsu,
    /// Integer multiply (high bits, signed × unsigned).
    Mulhu,
    /// Integer multiply (high bits, unsigned × unsigned).
    Div,
    /// Integer divide (signed).
    Divu,
    /// Integer divide (unsigned).
    Rem,
    /// Integer remainder (signed).
    Remu,
    /// Integer remainder (unsigned).
    FAdd,
    /// Floating-point addition.
    FSub,
    /// Floating-point subtraction.
    FMul,
    /// Floating-point multiplication.
    FDiv,
    /// Floating-point division.
    FSqrt,
    /// Floating-point square root.
    FMin,
    /// Floating-point minimum.
    FMax,
    /// Floating-point maximum.
    FMAdd,
    /// Floating-point multiply-add (fused).
    FMSub,
    /// Floating-point multiply-subtract (fused).
    FNMAdd,
    /// Floating-point negated multiply-add (fused).
    FNMSub,
    /// Floating-point negated multiply-subtract (fused).
    FCvtWS,
    /// Convert word to single-precision float (signed).
    FCvtLS,
    /// Convert long to single-precision float (signed).
    FCvtSW,
    /// Convert single-precision float to word (signed).
    FCvtSL,
    /// Convert single-precision float to long (signed).
    FCvtSD,
    /// Convert single-precision to double-precision float.
    FCvtDS,
    /// Convert double-precision to single-precision float.
    FSgnJ,
    /// Floating-point sign injection (copy sign).
    FSgnJN,
    /// Floating-point sign injection (negate sign).
    FSgnJX,
    /// Floating-point sign injection (XOR sign).
    FEq,
    /// Floating-point equality comparison.
    FLt,
    /// Floating-point less-than comparison.
    FLe,
    /// Floating-point less-than-or-equal comparison.
    FClass,
    /// Floating-point classify.
    /// Move floating-point register to integer register.
    FMvToX,
    /// Move integer register to floating-point register.
    FMvToF,
}

/// Atomic memory operation types (RISC-V A extension).
///
/// Specifies the type of atomic operation to perform on memory,
/// including load-reserved (LR), store-conditional (SC), and
/// various atomic read-modify-write operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AtomicOp {
    /// No atomic operation.
    #[default]
    None,
    /// Load-reserved (atomic load with reservation).
    Lr,
    /// Store-conditional (atomic store if reservation valid).
    Sc,
    /// Atomic swap.
    Swap,
    /// Atomic add.
    Add,
    /// Atomic XOR.
    Xor,
    /// Atomic AND.
    And,
    /// Atomic OR.
    Or,
    /// Atomic minimum (signed).
    Min,
    /// Atomic maximum (signed).
    Max,
    /// Atomic minimum (unsigned).
    Minu,
    /// Atomic maximum (unsigned).
    Maxu,
}

/// Memory access width for load and store operations.
///
/// Specifies the size of data to be loaded from or stored to memory.
#[derive(Clone, Copy, Debug, Default)]
pub enum MemWidth {
    /// No memory operation.
    #[default]
    Nop,
    /// 8-bit byte access.
    Byte,
    /// 16-bit half-word access.
    Half,
    /// 32-bit word access.
    Word,
    /// 64-bit double-word access.
    Double,
}

/// Source for ALU operand A.
///
/// Selects the source of the first ALU operand from register file,
/// program counter, or zero.
#[derive(Clone, Copy, Debug, Default)]
pub enum OpASrc {
    /// Use rs1 register value.
    #[default]
    Reg1,
    /// Use program counter value (for AUIPC, JAL).
    Pc,
    /// Use zero (for LUI).
    Zero,
}

/// Source for ALU operand B.
///
/// Selects the source of the second ALU operand from immediate value,
/// register file, or zero.
#[derive(Clone, Copy, Debug, Default)]
pub enum OpBSrc {
    /// Use sign-extended immediate value.
    #[default]
    Imm,
    /// Use rs2 register value.
    Reg2,
    /// Use zero.
    Zero,
}

/// CSR (Control and Status Register) operation type.
///
/// Specifies the type of CSR operation: read-write, read-set, read-clear,
/// or their immediate variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CsrOp {
    /// No CSR operation.
    #[default]
    None,
    /// CSR read-write (CSRRW).
    Rw,
    /// CSR read-set (CSRRS).
    Rs,
    /// CSR read-clear (CSRRC).
    Rc,
    /// CSR read-write immediate (CSRRWI).
    Rwi,
    /// CSR read-set immediate (CSRRSI).
    Rsi,
    /// CSR read-clear immediate (CSRRCI).
    Rci,
}

/// Control signals for pipeline stage execution.
///
/// Contains all control signals generated during instruction decode
/// that control execution, memory access, register writes, and
/// system operations throughout the pipeline stages.
#[derive(Clone, Copy, Debug, Default)]
pub struct ControlSignals {
    /// Enable write to integer destination register.
    pub reg_write: bool,
    /// Enable write to floating-point destination register.
    pub fp_reg_write: bool,
    /// Enable memory read operation (load).
    pub mem_read: bool,
    /// Enable memory write operation (store).
    pub mem_write: bool,
    /// Instruction is a conditional branch.
    pub branch: bool,
    /// Instruction is an unconditional jump (JAL/JALR).
    pub jump: bool,
    /// Instruction uses 32-bit operands (RV32 mode).
    pub is_rv32: bool,
    /// Width of memory access for load/store operations.
    pub width: MemWidth,
    /// Load should be sign-extended (vs zero-extended).
    pub signed_load: bool,
    /// ALU operation to perform.
    pub alu: AluOp,
    /// Source selection for ALU operand A.
    pub a_src: OpASrc,
    /// Source selection for ALU operand B.
    pub b_src: OpBSrc,
    /// Instruction is a system instruction (CSR, ECALL, etc.).
    pub is_system: bool,
    /// CSR address for CSR operations.
    pub csr_addr: u32,
    /// Instruction is MRET (return from machine mode).
    pub is_mret: bool,
    /// Instruction is SRET (return from supervisor mode).
    pub is_sret: bool,
    /// CSR operation type.
    pub csr_op: CsrOp,
    /// rs1 is a floating-point register.
    pub rs1_fp: bool,
    /// rs2 is a floating-point register.
    pub rs2_fp: bool,
    /// rs3 is a floating-point register (for FMA instructions).
    pub rs3_fp: bool,
    /// Atomic memory operation type.
    pub atomic_op: AtomicOp,
    /// Instruction is FENCE.I (instruction fence).
    pub is_fence_i: bool,
}

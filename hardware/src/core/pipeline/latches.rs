//! Pipeline latch structures for inter-stage communication.
//!
//! Pipeline latches store instruction state as it flows through the
//! five pipeline stages. Each latch contains entries for multiple
//! instructions to support superscalar execution.

use crate::common::error::Trap;
use crate::core::pipeline::signals::ControlSignals;

/// Entry in the IF/ID pipeline latch (Fetch to Decode stage).
///
/// Contains instruction information fetched from memory, including
/// the instruction encoding, program counter, branch prediction
/// information, and any fetch-time traps.
#[derive(Clone, Default, Debug)]
pub struct IfIdEntry {
    /// Program counter of the instruction.
    pub pc: u64,
    /// 32-bit instruction encoding (expanded from compressed if needed).
    pub inst: u32,
    /// Size of the instruction in bytes (2 for compressed, 4 for standard).
    pub inst_size: u64,
    /// Whether the branch predictor predicted this instruction as taken.
    pub pred_taken: bool,
    /// Predicted target address for branch/jump instructions.
    pub pred_target: u64,
    /// Trap that occurred during fetch, if any.
    pub trap: Option<Trap>,
}

/// Entry in the ID/EX pipeline latch (Decode to Execute stage).
///
/// Contains decoded instruction information including register indices,
/// immediate values, register values read from the register file,
/// control signals, and branch prediction information.
#[derive(Clone, Default, Debug)]
pub struct IdExEntry {
    /// Program counter of the instruction.
    pub pc: u64,
    /// 32-bit instruction encoding.
    pub inst: u32,
    /// Size of the instruction in bytes.
    pub inst_size: u64,
    /// First source register index (rs1).
    pub rs1: usize,
    /// Second source register index (rs2).
    pub rs2: usize,
    /// Third source register index (rs3, for FMA instructions).
    pub rs3: usize,
    /// Destination register index (rd).
    pub rd: usize,
    /// Sign-extended immediate value.
    pub imm: i64,
    /// Value read from rs1 register.
    pub rv1: u64,
    /// Value read from rs2 register.
    pub rv2: u64,
    /// Value read from rs3 register (for FMA instructions).
    pub rv3: u64,
    /// Control signals for downstream pipeline stages.
    pub ctrl: ControlSignals,
    /// Trap that occurred during decode, if any.
    pub trap: Option<Trap>,
    /// Whether the branch predictor predicted this instruction as taken.
    pub pred_taken: bool,
    /// Predicted target address for branch/jump instructions.
    pub pred_target: u64,
}

/// Entry in the EX/MEM pipeline latch (Execute to Memory stage).
///
/// Contains execution results including ALU output, store data,
/// and control signals needed for the memory stage.
#[derive(Clone, Default, Debug)]
pub struct ExMemEntry {
    /// Program counter of the instruction.
    pub pc: u64,
    /// 32-bit instruction encoding.
    pub inst: u32,
    /// Size of the instruction in bytes.
    pub inst_size: u64,
    /// Destination register index (rd).
    pub rd: usize,
    /// ALU computation result or address for memory operations.
    pub alu: u64,
    /// Data to be stored (for store instructions).
    pub store_data: u64,
    /// Control signals for downstream pipeline stages.
    pub ctrl: ControlSignals,
    /// Trap that occurred during execute, if any.
    pub trap: Option<Trap>,
}

/// Entry in the MEM/WB pipeline latch (Memory to Writeback stage).
///
/// Contains memory stage results including loaded data, ALU results,
/// and control signals needed for register writeback.
#[derive(Clone, Default, Debug)]
pub struct MemWbEntry {
    /// Program counter of the instruction.
    pub pc: u64,
    /// 32-bit instruction encoding.
    pub inst: u32,
    /// Size of the instruction in bytes.
    pub inst_size: u64,
    /// Destination register index (rd).
    pub rd: usize,
    /// ALU computation result (for non-load instructions).
    pub alu: u64,
    /// Data loaded from memory (for load instructions).
    pub load_data: u64,
    /// Control signals for the writeback stage.
    pub ctrl: ControlSignals,
    /// Trap that occurred during memory access, if any.
    pub trap: Option<Trap>,
}

/// IF/ID pipeline latch (Fetch to Decode stage).
///
/// Contains a vector of instructions fetched from memory, ready
/// to be decoded. Supports multiple instructions per cycle for
/// superscalar execution.
#[derive(Clone, Debug)]
pub struct IfId {
    /// Vector of fetched instruction entries.
    pub entries: Vec<IfIdEntry>,
}

impl Default for IfId {
    /// Creates an empty IF/ID latch.
    ///
    /// # Returns
    ///
    /// A new `IfId` instance with an empty entries vector.
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

/// ID/EX pipeline latch (Decode to Execute stage).
///
/// Contains a vector of decoded instructions with register values
/// and control signals, ready for execution. Supports multiple
/// instructions per cycle for superscalar execution.
#[derive(Clone, Default, Debug)]
pub struct IdEx {
    /// Vector of decoded instruction entries.
    pub entries: Vec<IdExEntry>,
}

/// EX/MEM pipeline latch (Execute to Memory stage).
///
/// Contains a vector of execution results ready for memory access.
/// Supports multiple instructions per cycle for superscalar execution.
#[derive(Clone, Default, Debug)]
pub struct ExMem {
    /// Vector of execution result entries.
    pub entries: Vec<ExMemEntry>,
}

/// MEM/WB pipeline latch (Memory to Writeback stage).
///
/// Contains a vector of memory stage results ready for register
/// writeback. Supports multiple instructions per cycle for superscalar execution.
#[derive(Clone, Default, Debug)]
pub struct MemWb {
    /// Vector of memory stage result entries.
    pub entries: Vec<MemWbEntry>,
}

//! Pipeline stage implementations.
//!
//! Contains the five stages of the instruction pipeline:
//! - Fetch: Retrieves instructions from memory
//! - Decode: Decodes instructions and reads register values
//! - Execute: Performs ALU operations and branch resolution
//! - Memory: Handles load/store operations
//! - Writeback: Writes results back to registers and handles traps

/// Instruction decode stage implementation.
pub mod decode;

/// Instruction execute stage implementation.
pub mod execute;

/// Instruction fetch stage implementation.
pub mod fetch;

/// Memory access stage implementation.
pub mod memory;

/// Writeback stage implementation.
pub mod writeback;

pub use decode::decode_stage;
pub use execute::execute_stage;
pub use fetch::fetch_stage;
pub use memory::mem_stage;
pub use writeback::wb_stage;

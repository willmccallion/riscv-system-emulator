//! Test module organization.
//!
//! This module organizes all integration tests for the RISC-V simulator.

/// ALU (Arithmetic Logic Unit) instruction tests.
mod alu_tests;

/// Architecture-specific register and CSR tests.
mod arch_tests;

/// Branch predictor algorithm tests.
mod branch_predictor_tests;

/// Cache hierarchy and replacement policy tests.
mod cache_tests;

/// Common utility and data structure tests.
mod common_tests;

/// Floating-Point Unit instruction tests.
mod fpu_tests;

/// End-to-end system integration tests.
mod integration_tests;

/// Instruction Set Architecture decoding and encoding tests.
mod isa_tests;

/// Load/Store Unit memory access tests.
mod lsu_tests;

/// Memory Management Unit and page table walk tests.
mod mmu_test;

/// Stress tests and performance benchmarks.
mod stress_tests;

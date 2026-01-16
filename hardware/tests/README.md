# Test Suite for RISC-V System Simulator

This directory contains comprehensive test suites for the RISC-V system simulator.

## Test Structure

### Unit Tests
Unit tests are located in `#[cfg(test)]` modules within each source file, testing individual components in isolation.

### Integration Tests
Integration tests are in the `tests/` directory and test multiple components working together:

- **common_tests.rs**: Tests for common utilities (addresses, errors, registers)
- **arch_tests.rs**: Tests for architecture components (CSRs, registers, privilege modes)
- **alu_tests.rs**: Tests for ALU operations (arithmetic, logical, shifts)
- **fpu_tests.rs**: Tests for FPU operations (floating-point arithmetic)
- **isa_tests.rs**: Tests for instruction decoding and expansion
- **cache_tests.rs**: Tests for cache system (hits, misses, replacement policies)
- **branch_predictor_tests.rs**: Tests for branch prediction algorithms
- **lsu_tests.rs**: Tests for load/store unit atomic operations
- **csr_tests.rs**: Comprehensive CSR operation tests (potential corruption detection)
- **forwarding_tests.rs**: Comprehensive register forwarding tests (memory corruption detection)
- **integration_tests.rs**: Full system integration tests
- **stress_tests.rs**: Stress tests and edge cases

## Running Tests

Run all tests:
```bash
cargo test
```

Run specific test file:
```bash
cargo test --test csr_tests
cargo test --test forwarding_tests
```

Run with output:
```bash
cargo test -- --nocapture
```

## Test Coverage

The test suite covers:
- All ALU operations (add, sub, mul, div, shifts, logical)
- All FPU operations (add, sub, mul, div, conversions, comparisons)
- CSR read/write operations and synchronization
- Register forwarding from all pipeline stages
- Cache hit/miss behavior and replacement policies
- Branch prediction algorithms
- Instruction decoding and expansion
- Address translation and page handling
- Atomic memory operations
- Edge cases and stress scenarios

## Known Test Issues

Some predictor tests (gshare, tage) may fail due to the complex nature of global history-based prediction. These are being refined to match actual predictor behavior.

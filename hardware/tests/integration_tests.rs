//! Integration tests for the full system.

use riscv_emulator::config::*;
use riscv_emulator::core::arch::csr;
use riscv_emulator::core::Cpu;
use riscv_emulator::soc::System;

/// Creates a minimal configuration for testing.
fn create_minimal_config() -> Config {
    Config {
        general: GeneralConfig {
            trace_instructions: false,
            start_pc: 0x8000_0000,
        },
        system: SystemConfig {
            uart_base: 0x1000_0000,
            disk_base: 0x9000_0000,
            ram_base: 0x8000_0000,
            clint_base: 0x0200_0000,
            syscon_base: 0x0010_0000,
            kernel_offset: 0x0020_0000,
            bus_width: 8,
            bus_latency: 4,
            clint_divider: 10,
        },
        memory: MemoryConfig {
            ram_size: 128 * 1024 * 1024,
            controller: MemoryController::Simple,
            t_cas: 14,
            t_ras: 14,
            t_pre: 14,
            row_miss_latency: 120,
            tlb_size: 32,
        },
        cache: CacheHierarchyConfig {
            l1_i: CacheConfig {
                enabled: false,
                size_bytes: 4096,
                ways: 1,
                line_bytes: 64,
                latency: 1,
                policy: ReplacementPolicy::Lru,
                prefetcher: Prefetcher::None,
                prefetch_table_size: 64,
                prefetch_degree: 1,
            },
            l1_d: CacheConfig {
                enabled: false,
                size_bytes: 4096,
                ways: 1,
                line_bytes: 64,
                latency: 1,
                policy: ReplacementPolicy::Lru,
                prefetcher: Prefetcher::None,
                prefetch_table_size: 64,
                prefetch_degree: 1,
            },
            l2: CacheConfig {
                enabled: false,
                size_bytes: 65536,
                ways: 8,
                line_bytes: 64,
                latency: 10,
                policy: ReplacementPolicy::Lru,
                prefetcher: Prefetcher::None,
                prefetch_table_size: 64,
                prefetch_degree: 1,
            },
            l3: CacheConfig {
                enabled: false,
                size_bytes: 1048576,
                ways: 16,
                line_bytes: 64,
                latency: 50,
                policy: ReplacementPolicy::Lru,
                prefetcher: Prefetcher::None,
                prefetch_table_size: 64,
                prefetch_degree: 1,
            },
        },
        pipeline: PipelineConfig {
            width: 1,
            branch_predictor: BranchPredictor::Static,
            btb_size: 256,
            ras_size: 8,
            misa_override: None,
            tage: TageConfig::default(),
            perceptron: PerceptronConfig::default(),
            tournament: TournamentConfig::default(),
        },
    }
}

/// Tests CPU creation and initialization.
#[test]
fn test_cpu_creation() {
    let config = create_minimal_config();
    let system = System::new(&config, "");
    let cpu = Cpu::new(system, &config);

    assert_eq!(cpu.pc, config.general.start_pc);
    assert_eq!(cpu.privilege.to_u8(), 3);
}

/// Tests system creation and initialization.
#[test]
fn test_system_creation() {
    let config = create_minimal_config();
    let system = System::new(&config, "");

    assert_eq!(system.bus.width_bytes, 8);
    assert_eq!(system.bus.latency_cycles, 4);
}

/// Tests CPU register initialization.
#[test]
fn test_cpu_registers_initialized() {
    let config = create_minimal_config();
    let system = System::new(&config, "");
    let cpu = Cpu::new(system, &config);

    for i in 0..32 {
        assert_eq!(cpu.regs.read(i), 0);
    }
}

/// Tests CPU CSR initialization.
#[test]
fn test_cpu_csrs_initialized() {
    let config = create_minimal_config();
    let system = System::new(&config, "");
    let cpu = Cpu::new(system, &config);

    assert_ne!(cpu.csrs.read(csr::MISA), 0);
}

/// Tests pipeline latch initialization.
#[test]
fn test_pipeline_latches_initialized() {
    let config = create_minimal_config();
    let system = System::new(&config, "");
    let cpu = Cpu::new(system, &config);

    assert_eq!(cpu.if_id.entries.len(), 0);
    assert_eq!(cpu.id_ex.entries.len(), 0);
    assert_eq!(cpu.ex_mem.entries.len(), 0);
    assert_eq!(cpu.mem_wb.entries.len(), 0);
}

//! Integration tests for cache system.

use riscv_emulator::config::{CacheConfig, Prefetcher, ReplacementPolicy};
use riscv_emulator::core::units::cache::CacheSim;

/// Creates a test cache configuration.
fn create_test_cache_config() -> CacheConfig {
    CacheConfig {
        enabled: true,
        size_bytes: 4096,
        ways: 4,
        line_bytes: 64,
        latency: 1,
        policy: ReplacementPolicy::Lru,
        prefetcher: Prefetcher::None,
        prefetch_table_size: 64,
        prefetch_degree: 1,
    }
}

/// Tests cache creation and initialization.
#[test]
fn test_cache_creation() {
    let config = create_test_cache_config();
    let cache = CacheSim::new(&config);

    assert!(cache.enabled);
    assert_eq!(cache.latency, 1);
}

/// Tests cache line presence checking.
#[test]
fn test_cache_contains() {
    let config = create_test_cache_config();
    let cache = CacheSim::new(&config);

    assert!(!cache.contains(0x1000));
}

/// Tests cache hit behavior after initial miss.
#[test]
fn test_cache_access_hit() {
    let config = create_test_cache_config();
    let mut cache = CacheSim::new(&config);

    let (hit, penalty) = cache.access(0x1000, false, 10);
    assert!(!hit);
    assert!(penalty > 0);

    let (hit2, penalty2) = cache.access(0x1000, false, 10);
    assert!(hit2);
    assert_eq!(penalty2, 0);
}

/// Tests cache miss behavior.
#[test]
fn test_cache_access_miss() {
    let config = create_test_cache_config();
    let mut cache = CacheSim::new(&config);

    let (hit, _penalty) = cache.access(0x1000, false, 10);
    assert!(!hit);
}

/// Tests cache write operations.
#[test]
fn test_cache_write() {
    let config = create_test_cache_config();
    let mut cache = CacheSim::new(&config);

    let (hit, _) = cache.access(0x1000, true, 10);
    assert!(!hit);

    let (hit2, _) = cache.access(0x1000, false, 10);
    assert!(hit2);
}

/// Tests cache flush operation.
#[test]
fn test_cache_flush() {
    let config = create_test_cache_config();
    let mut cache = CacheSim::new(&config);

    cache.access(0x1000, false, 10);
    cache.access(0x2000, false, 10);

    assert!(cache.contains(0x1000));
    assert!(cache.contains(0x2000));

    cache.flush();
}

/// Tests LRU replacement policy.
#[test]
fn test_cache_replacement_lru() {
    let mut config = create_test_cache_config();
    config.size_bytes = 256;
    config.ways = 4;
    config.line_bytes = 16;
    config.policy = ReplacementPolicy::Lru;

    let mut cache = CacheSim::new(&config);

    for i in 0..4 {
        cache.access(0x1000 + (i * 16), false, 10);
    }

    for i in 0..4 {
        assert!(cache.contains(0x1000 + (i * 16)));
    }

    cache.access(0x1000, false, 10);

    cache.access(0x2000, false, 10);

    assert!(cache.contains(0x1000));
}

/// Tests FIFO replacement policy.
#[test]
fn test_cache_replacement_fifo() {
    let mut config = create_test_cache_config();
    config.size_bytes = 256;
    config.ways = 4;
    config.line_bytes = 16;
    config.policy = ReplacementPolicy::Fifo;

    let mut cache = CacheSim::new(&config);

    for i in 0..4 {
        cache.access(0x1000 + (i * 16), false, 10);
    }

    cache.access(0x2000, false, 10);

    assert!(!cache.contains(0x1000));
}

/// Tests disabled cache behavior.
#[test]
fn test_cache_disabled() {
    let mut config = create_test_cache_config();
    config.enabled = false;

    let mut cache = CacheSim::new(&config);

    assert!(!cache.enabled);
    assert!(!cache.contains(0x1000));

    let (hit, penalty) = cache.access(0x1000, false, 10);
    assert!(!hit);
    assert_eq!(penalty, 0);
}

/// Tests cache line alignment behavior.
#[test]
fn test_cache_line_alignment() {
    let config = create_test_cache_config();
    let mut cache = CacheSim::new(&config);

    cache.access(0x1000, false, 10);
    cache.access(0x1001, false, 10);
    cache.access(0x103F, false, 10);

    assert!(cache.contains(0x1000));
    assert!(cache.contains(0x1001));
    assert!(cache.contains(0x103F));
}

/// Tests multiple cache sets.
#[test]
fn test_cache_multiple_sets() {
    let config = create_test_cache_config();
    let mut cache = CacheSim::new(&config);

    cache.access(0x1000, false, 10);
    cache.access(0x2000, false, 10);
    cache.access(0x3000, false, 10);

    assert!(cache.contains(0x1000));
    assert!(cache.contains(0x2000));
    assert!(cache.contains(0x3000));
}

/// Tests write-back penalty for dirty cache lines.
#[test]
fn test_cache_write_back_penalty() {
    let config = create_test_cache_config();
    let mut cache = CacheSim::new(&config);

    cache.access(0x1000, true, 10);

    for i in 1..5 {
        cache.access(0x1000 + (i * 64), false, 10);
    }

    let (hit, penalty) = cache.access(0x2000, false, 10);
    assert!(!hit);
    assert!(penalty >= 10);
}

/// Tests MRU replacement policy.
#[test]
fn test_cache_replacement_mru() {
    let mut config = create_test_cache_config();
    config.size_bytes = 256;
    config.ways = 4;
    config.line_bytes = 16;
    config.policy = ReplacementPolicy::Mru;

    let mut cache = CacheSim::new(&config);

    for i in 0..4 {
        cache.access(0x1000 + (i * 16), false, 10);
    }

    cache.access(0x1010, false, 10);

    cache.access(0x2000, false, 10);

    assert!(!cache.contains(0x1010), "MRU should have evicted 0x1010");
    assert!(cache.contains(0x1000));
    assert!(cache.contains(0x1020));
    assert!(cache.contains(0x1030));
    assert!(cache.contains(0x2000));
}

/// Tests Stream prefetcher (Ascending).
#[test]
fn test_prefetch_stream_ascending() {
    let mut config = create_test_cache_config();
    config.prefetcher = Prefetcher::Stream;
    config.prefetch_degree = 1;
    config.line_bytes = 64;

    let mut cache = CacheSim::new(&config);

    cache.access(0x1000, false, 10);
    cache.access(0x1040, false, 10);
    cache.access(0x1080, false, 10);

    assert!(
        cache.contains(0x10C0),
        "Stream prefetcher failed to fetch next line"
    );
}

/// Tests Stream prefetcher (Descending).
#[test]
fn test_prefetch_stream_descending() {
    let mut config = create_test_cache_config();
    config.prefetcher = Prefetcher::Stream;
    config.prefetch_degree = 1;
    config.line_bytes = 64;

    let mut cache = CacheSim::new(&config);

    cache.access(0x2000, false, 10);
    cache.access(0x1FC0, false, 10);
    cache.access(0x1F80, false, 10);

    assert!(
        cache.contains(0x1F40),
        "Stream prefetcher failed to fetch prev line"
    );
}

/// Tests Tagged prefetcher.
#[test]
fn test_prefetch_tagged() {
    let mut config = create_test_cache_config();
    config.prefetcher = Prefetcher::Tagged;
    config.prefetch_degree = 1;
    config.line_bytes = 64;

    let mut cache = CacheSim::new(&config);

    cache.access(0x1000, false, 10);
    assert!(
        cache.contains(0x1040),
        "Tagged prefetcher should fetch on miss"
    );

    cache.access(0x1040, false, 10);
    assert!(
        cache.contains(0x1080),
        "Tagged prefetcher should fetch on hit of prefetched line"
    );

    cache.access(0x2000, false, 10);
    assert!(cache.contains(0x2040));

    cache.access(0x2000, false, 10);
    assert!(
        !cache.contains(0x2080),
        "Tagged prefetcher should NOT fetch on hit of demand line"
    );
}

/// Tests Stride prefetcher confidence warmup.
///
/// The stride prefetcher requires seeing the same stride twice (confidence > 1)
/// before it begins issuing prefetches.
#[test]
fn test_prefetch_stride_confidence() {
    let mut config = create_test_cache_config();
    config.prefetcher = Prefetcher::Stride;
    config.prefetch_degree = 1;
    config.line_bytes = 64;

    let mut cache = CacheSim::new(&config);

    cache.access(0x1000, false, 10);
    assert!(!cache.contains(0x1040));

    cache.access(0x1040, false, 10);
    assert!(!cache.contains(0x1080));

    cache.access(0x1080, false, 10);

    assert!(
        cache.contains(0x10C0),
        "Stride prefetcher should fire after confidence warmup"
    );
}

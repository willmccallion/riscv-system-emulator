//! Integration tests for branch prediction.

use riscv_emulator::config::{PerceptronConfig, TageConfig, TournamentConfig};
use riscv_emulator::core::units::bru::*;

/// Tests static branch predictor behavior.
#[test]
fn test_static_predictor() {
    let mut bp = static_bp::StaticPredictor::new(256, 8);

    let (taken, target) = bp.predict_branch(0x1000);
    assert!(!taken);
    assert!(target.is_none());

    bp.update_branch(0x1000, true, Some(0x2000));

    let (taken2, _) = bp.predict_branch(0x1000);
    assert!(!taken2);
}

/// Tests branch target buffer functionality.
#[test]
fn test_btb() {
    let mut bp = static_bp::StaticPredictor::new(256, 8);

    bp.on_call(0x1000, 0x1004, 0x2000);
    let target = bp.predict_btb(0x1000);
    assert_eq!(target, Some(0x2000));
}

/// Tests return address stack push and pop operations.
#[test]
fn test_ras() {
    let mut bp = static_bp::StaticPredictor::new(256, 8);

    bp.on_call(0x1000, 0x1004, 0x2000);

    let ret = bp.predict_return();
    assert_eq!(ret, Some(0x1004));

    bp.on_return();

    let ret2 = bp.predict_return();
    assert!(ret2.is_none());
}

/// Tests return address stack with multiple nested calls.
#[test]
fn test_ras_multiple_calls() {
    let mut bp = static_bp::StaticPredictor::new(256, 8);

    bp.on_call(0x1000, 0x1004, 0x2000);
    bp.on_call(0x2000, 0x2004, 0x3000);
    bp.on_call(0x3000, 0x3004, 0x4000);

    let ret = bp.predict_return();
    assert_eq!(ret, Some(0x3004));

    bp.on_return();
    let ret2 = bp.predict_return();
    assert_eq!(ret2, Some(0x2004));

    bp.on_return();
    let ret3 = bp.predict_return();
    assert_eq!(ret3, Some(0x1004));
}

/// Tests return address stack overflow behavior.
#[test]
fn test_ras_overflow() {
    let mut bp = static_bp::StaticPredictor::new(256, 2);

    bp.on_call(0x1000, 0x1004, 0x2000);
    bp.on_call(0x2000, 0x2004, 0x3000);
    bp.on_call(0x3000, 0x3004, 0x4000);

    let ret = bp.predict_return();
    assert_eq!(ret, Some(0x3004), "RAS should return most recent entry");

    bp.on_return();
    let ret2 = bp.predict_return();
    assert_eq!(
        ret2,
        Some(0x1004),
        "RAS should return first entry after pop"
    );

    bp.on_return();
    let ret3 = bp.predict_return();
    assert!(
        ret3.is_none(),
        "RAS should be empty after popping all entries"
    );
}

/// Tests GShare branch predictor with global history.
#[test]
fn test_gshare_predictor() {
    let mut bp = gshare::GSharePredictor::new(256, 8);

    bp.update_branch(0x1000, true, Some(0x2000));
    bp.update_branch(0x1000, true, Some(0x2000));
    bp.update_branch(0x1000, true, Some(0x2000));

    let target = bp.predict_btb(0x1000);
    assert_eq!(
        target,
        Some(0x2000),
        "GShare BTB should have correct target"
    );

    let (_taken, target2) = bp.predict_branch(0x1000);
    assert_eq!(
        target2,
        Some(0x2000),
        "GShare should predict correct target from BTB"
    );
}

/// Tests GShare predictor misprediction handling.
#[test]
fn test_gshare_misprediction() {
    let mut bp = gshare::GSharePredictor::new(256, 8);

    for _ in 0..5 {
        bp.update_branch(0x1000, true, Some(0x2000));
    }

    for _ in 0..10 {
        bp.update_branch(0x1000, false, None);
    }

    let (taken, _) = bp.predict_branch(0x1000);
    assert!(!taken);
}

/// Tests perceptron branch predictor.
#[test]
fn test_perceptron_predictor() {
    let config = PerceptronConfig::default();
    let mut bp = perceptron::PerceptronPredictor::new(&config, 256, 8);

    for _ in 0..20 {
        bp.update_branch(0x1000, true, Some(0x2000));
    }

    let (taken, target) = bp.predict_branch(0x1000);
    assert!(taken);
    assert_eq!(target, Some(0x2000));
}

/// Tests TAGE branch predictor.
#[test]
fn test_tage_predictor() {
    let config = TageConfig {
        num_banks: 4,
        table_size: 2048,
        loop_table_size: 256,
        reset_interval: 256_000,
        history_lengths: vec![5, 15, 44, 130],
        tag_widths: vec![9, 9, 10, 10],
    };
    let mut bp = tage::TagePredictor::new(&config, 256, 8);

    for _ in 0..3 {
        bp.update_branch(0x1000, true, Some(0x2000));
    }

    let (taken, target) = bp.predict_branch(0x1000);
    assert!(taken, "TAGE base table should predict taken after training");
    assert_eq!(target, Some(0x2000), "TAGE should predict correct target");
}

/// Tests tournament branch predictor.
#[test]
fn test_tournament_predictor() {
    let config = TournamentConfig::default();
    let mut bp = tournament::TournamentPredictor::new(&config, 256, 8);

    for _ in 0..10 {
        bp.update_branch(0x1000, true, Some(0x2000));
    }

    let (taken, target) = bp.predict_branch(0x1000);
    assert!(taken);
    assert_eq!(target, Some(0x2000));
}

/// Tests branch target prediction.
#[test]
fn test_branch_target_prediction() {
    let mut bp = static_bp::StaticPredictor::new(256, 8);

    bp.update_branch(0x1000, true, Some(0x2000));

    let target = bp.predict_btb(0x1000);
    assert_eq!(target, Some(0x2000));
}

/// Tests branch target update behavior.
#[test]
fn test_branch_target_update() {
    let mut bp = static_bp::StaticPredictor::new(256, 8);

    bp.update_branch(0x1000, true, Some(0x2000));
    assert_eq!(bp.predict_btb(0x1000), Some(0x2000));

    bp.update_branch(0x1000, true, Some(0x3000));
    assert_eq!(bp.predict_btb(0x1000), Some(0x3000));
}

/// Tests GShare counter saturation.
///
/// Verifies that the 2-bit counters do not wrap around (e.g., 3 + 1 != 0).
#[test]
fn test_gshare_saturation() {
    let mut bp = gshare::GSharePredictor::new(256, 8);
    let pc = 0x1000;

    for _ in 0..10 {
        bp.update_branch(pc, true, Some(0x2000));
    }

    bp.update_branch(pc, false, None);

    let (taken, _) = bp.predict_branch(pc);
    assert!(
        taken,
        "Predictor should have hysteresis and remain taken after one miss"
    );
}

/// Tests TAGE provider allocation logic.
///
/// Ensures that a misprediction on the base predictor allocates an entry
/// in a tagged bank.
#[test]
fn test_tage_allocation() {
    let config = TageConfig::default();
    let mut bp = tage::TagePredictor::new(&config, 256, 8);
    let pc = 0x1000;

    let (taken_initial, _) = bp.predict_branch(pc);
    assert!(!taken_initial);

    bp.update_branch(pc, true, Some(0x2000));

    let (taken_after, _) = bp.predict_branch(pc);
    assert!(taken_after);
}

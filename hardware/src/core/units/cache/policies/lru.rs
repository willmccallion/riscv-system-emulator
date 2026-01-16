//! Least Recently Used (LRU) Replacement Policy.
//!
//! This policy evicts the cache line that has not been accessed for the longest time.
//! It maintains a usage stack for each set. When a line is accessed, it is moved
//! to the top (Most Recently Used position). The bottom of the stack represents
//! the Least Recently Used line.

use super::ReplacementPolicy;

/// LRU Policy state.
pub struct LruPolicy {
    /// A vector of usage stacks (one per set).
    /// Index 0 is MRU, last index is LRU.
    usage: Vec<Vec<usize>>,
}

impl LruPolicy {
    /// Creates a new LRU policy instance.
    ///
    /// # Arguments
    ///
    /// * `sets` - The number of sets in the cache.
    /// * `ways` - The associativity (number of ways) of the cache.
    pub fn new(sets: usize, ways: usize) -> Self {
        let mut usage = Vec::with_capacity(sets);
        for _ in 0..sets {
            usage.push((0..ways).collect());
        }
        Self { usage }
    }
}

impl ReplacementPolicy for LruPolicy {
    /// Updates the policy state on access.
    ///
    /// Moves the accessed `way` to the front of the usage stack (MRU position),
    /// shifting other elements down.
    fn update(&mut self, set: usize, way: usize) {
        let stack = &mut self.usage[set];
        if let Some(pos) = stack.iter().position(|&x| x == way) {
            stack.remove(pos);
        }
        stack.insert(0, way);
    }

    /// Identifies the victim way to evict.
    ///
    /// Returns the way at the bottom of the usage stack (LRU position).
    fn get_victim(&mut self, set: usize) -> usize {
        *self.usage[set].last().unwrap()
    }
}

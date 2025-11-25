//! Parallel processing utilities for NUMA-aware computation.
//!
//! Provides thread pool initialization and workload chunking for efficient
//! parallel processing with [Rayon](https://docs.rs/rayon).
//!
//! # Example
//!
//! ```rust
//! use h3on::parallel::chunk_bounds;
//! use rayon::prelude::*;
//!
//! let data: Vec<i32> = (0..1_000_000).collect();
//! let (min_len, max_len) = chunk_bounds(data.len());
//!
//! let sum: i32 = data.par_iter()
//!     .with_min_len(min_len)
//!     .with_max_len(max_len)
//!     .sum();
//! ```

use rayon::{ThreadPoolBuilder, current_num_threads};
use std::sync::Once;

static START: Once = Once::new();

/// Initialize the global Rayon thread pool with custom spawn handler.
/// Called automatically by [`chunk_bounds`]; explicit calls are optional.
pub fn init_thread_pool() {
    START.call_once(|| {
        #[expect(clippy::let_underscore_must_use, reason = "ThreadPoolBuilder error is intentionally ignored as fallback behavior is acceptable")]
        let _ = ThreadPoolBuilder::new()
            .spawn_handler(|thread| {
                std::thread::Builder::new()
                    .name(thread.name().unwrap_or("rayon-worker").to_owned())
                    .spawn(move || thread.run())
                    .map(|_| ())
            })
            .build_global();
    });
}

/// Compute dynamic workload chunk bounds for parallel iterators.
/// Returns `(min_len, max_len)` used with `with_min_len`/`with_max_len`.
#[must_use]
pub fn chunk_bounds(total_len: usize) -> (usize, usize) {
    init_thread_pool();
    let threads = current_num_threads().max(1);
    let job_min = core::cmp::max(1024, total_len / (threads * 4));
    (job_min, job_min * 4)
}

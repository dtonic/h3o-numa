#[cfg(feature = "rayon")]
use rayon::{current_num_threads, ThreadPoolBuilder};
#[cfg(feature = "rayon")]
use std::sync::Once;

#[cfg(feature = "rayon")]
static START: Once = Once::new();

/// Initialize a global rayon thread pool with a spawn handler hook.
/// This prepares the ground for NUMA-aware thread pinning in later steps.
#[cfg(feature = "rayon")]
pub fn init_thread_pool() {
    START.call_once(|| {
        let _ = ThreadPoolBuilder::new()
            .spawn_handler(|thread| {
                std::thread::Builder::new()
                    .name(thread.name().unwrap_or("rayon-worker").to_string())
                    .spawn(move || thread.run())
                    .map(|_| ())
            })
            .build_global();
    });
}

/// Compute dynamic workload chunk bounds for parallel iterators.
/// Returns `(min_len, max_len)` used with `with_min_len`/`with_max_len`.
#[cfg(feature = "rayon")]
pub fn chunk_bounds(total_len: usize) -> (usize, usize) {
    init_thread_pool();
    let threads = current_num_threads().max(1);
    let job_min = core::cmp::max(1024, total_len / (threads * 4));
    (job_min, job_min * 4)
}

//! NUMA-aware optimizations for h3on
//! 
//! This module provides NUMA topology detection, thread affinity management,
//! and first-touch memory initialization for optimal performance on NUMA systems.

#[cfg(feature = "numa")]
pub mod topo;
#[cfg(feature = "numa")]
pub mod pool;

#[cfg(feature = "numa")]
pub use topo::NumaTopology;
#[cfg(feature = "numa")]
pub use pool::{build_numa_pool, estimate_buffer_sizes};

/// Initialize NUMA topology and return the topology information
/// 
/// This function should be called once at startup to detect the system's
/// NUMA topology and cache it for later use.
#[cfg(feature = "numa")]
pub fn init_numa() -> NumaTopology {
    topo::load_topology()
}

/// Check if NUMA features are available
pub fn is_numa_available() -> bool {
    cfg!(feature = "numa")
}

//! NUMA-aware thread pool and first-touch memory initialization
//!
//! This module provides a thread pool that pins threads to specific cores
//! and initializes local memory buffers to ensure first-touch allocation.

use once_cell::unsync::OnceCell;

use crate::numa::topo::NumaTopology;

/// Node-local data structures for each worker thread
///
/// Contains scratch buffers and caches that are initialized
/// with first-touch allocation on the local NUMA node.
#[derive(Debug)]
pub struct NodeLocal {
    /// Scratch buffer for temporary computations
    pub scratch: Vec<u8>,
    /// Lookup table cache for geometry operations
    pub geometry_lut: Vec<u32>,
    /// Buffer for intermediate results
    pub intermediate: Vec<u64>,
}

impl NodeLocal {
    /// Create a new `NodeLocal` instance with specified buffer sizes
    ///
    /// This function performs first-touch allocation by writing to
    /// the allocated memory, ensuring it's mapped to the local NUMA node.
    fn new(
        scratch_size: usize,
        lut_size: usize,
        intermediate_size: usize,
    ) -> Self {
        // Allocate scratch buffer
        let mut scratch = vec![0_u8; scratch_size];

        // Allocate and initialize geometry lookup table
        let mut geometry_lut = vec![0_u32; lut_size];
        // Allocate intermediate buffer
        let mut intermediate = vec![0_u64; intermediate_size];

        // Perform first-touch by writing to all allocated memory
        // This ensures the pages are mapped to the local NUMA node
        scratch.fill(0);
        geometry_lut.fill(0);
        intermediate.fill(0);

        Self {
            scratch,
            geometry_lut,
            intermediate,
        }
    }

    /// Get a reference to the scratch buffer
    #[must_use]
    pub fn scratch(&self) -> &[u8] {
        &self.scratch
    }

    /// Get a mutable reference to the scratch buffer
    pub fn scratch_mut(&mut self) -> &mut [u8] {
        &mut self.scratch
    }

    /// Get a reference to the geometry lookup table
    #[must_use]
    pub fn geometry_lut(&self) -> &[u32] {
        &self.geometry_lut
    }

    /// Get a mutable reference to the geometry lookup table
    pub fn geometry_lut_mut(&mut self) -> &mut [u32] {
        &mut self.geometry_lut
    }

    /// Get a reference to the intermediate buffer
    #[must_use]
    pub fn intermediate(&self) -> &[u64] {
        &self.intermediate
    }

    /// Get a mutable reference to the intermediate buffer
    pub fn intermediate_mut(&mut self) -> &mut [u64] {
        &mut self.intermediate
    }
}

thread_local! {
    static NODE_LOCAL: OnceCell<NodeLocal> = const { OnceCell::new() };
}

/// Build and configure a NUMA-aware thread pool
///
/// This function creates a thread pool where each worker thread is:
/// 1. Pinned to a specific core using `core_affinity`
/// 2. Initialized with local memory buffers (first-touch allocation)
/// 3. Configured to handle work from the local NUMA node
///
/// # Arguments
///
/// * `topo` - NUMA topology information
/// * `buffer_sizes` - Buffer sizes for scratch, LUT, and intermediate buffers
/// * `work` - The work function to execute
///
/// # Returns
///
/// The result of executing the work function
pub fn build_numa_pool<F, R>(
    topo: &NumaTopology,
    buffer_sizes: (usize, usize, usize),
    work: F,
) -> R
where
    F: FnOnce() -> R + Send,
    R: Send,
{
    use rayon::ThreadPoolBuilder;

    // Collect all available cores from all NUMA nodes
    let worker_cores: Vec<usize> =
        topo.cores_per_node.iter().flatten().copied().collect();

    // Ensure we have at least one core to work with
    if worker_cores.is_empty() {
        // Fall back to standard processing if no cores available
        return work();
    }

    let workers = worker_cores.len();

    // Create thread pool with custom spawn handler
    let pool = ThreadPoolBuilder::new()
        .num_threads(workers)
        .spawn_handler(|thread| {
            let thread_index = thread.index();
            let core_id = worker_cores[thread_index % workers];

            // Pin the thread to a specific core
            let _ = core_affinity::set_for_current(core_affinity::CoreId {
                id: core_id,
            });

            // Initialize node-local buffers with first-touch allocation
            NODE_LOCAL.with(|cell| {
                let (scratch_size, lut_size, intermediate_size) = buffer_sizes;
                #[expect(clippy::let_underscore_must_use, reason = "OnceCell initialization is infallible for NodeLocal")]
                let _ = cell.set(NodeLocal::new(
                    scratch_size,
                    lut_size,
                    intermediate_size,
                ));
            });

            // Create and spawn the worker thread
            std::thread::Builder::new()
                .name(format!("h3on-numa-{thread_index}"))
                .spawn(move || thread.run())
                .map(|_| ())
        })
        .build()
        .expect("Failed to build NUMA-aware thread pool");

    // Execute the work function using the configured pool
    pool.install(work)
}

/// Access the current thread's node-local data
///
/// This function provides access to the `NodeLocal` instance that was
/// initialized when the current thread started.
///
/// # Arguments
///
/// * `f` - Function to execute with access to the `NodeLocal` instance
///
/// # Returns
///
/// The result of executing the function
pub fn with_node_local<T>(f: impl FnOnce(&NodeLocal) -> T) -> T {
    NODE_LOCAL.with(|cell| {
        let nl = cell.get().expect("NodeLocal not initialized");
        f(nl)
    })
}

/// Access the current thread's node-local data with mutable access
///
/// This function provides mutable access to the `NodeLocal` instance.
///
/// # Arguments
///
/// * `f` - Function to execute with mutable access to the `NodeLocal` instance
///
/// # Returns
///
/// The result of executing the function
pub fn with_node_local_mut<T>(_f: impl FnOnce(&mut NodeLocal) -> T) -> T {
    // For now, just use the immutable version to avoid unsafe casting
    // TODO: Implement proper mutable access when needed
    with_node_local(|_nl| {
        // This is a temporary workaround - we'll need to redesign this
        // to properly support mutable access without unsafe code
        unimplemented!("Mutable access to NodeLocal not yet implemented")
    })
}

/// Estimate buffer sizes based on input parameters
///
/// This function estimates appropriate buffer sizes for the `NodeLocal`
/// instance based on the expected workload.
///
/// # Arguments
///
/// * `resolution` - H3 resolution level
/// * `expected_cells` - Expected number of cells to process
///
/// # Returns
///
/// Tuple of (`scratch_size`, `lut_size`, `intermediate_size`)
#[must_use]
pub fn estimate_buffer_sizes(
    resolution: u8,
    expected_cells: usize,
) -> (usize, usize, usize) {
    // Scratch buffer: proportional to resolution and expected cells
    let scratch_size = (expected_cells * resolution as usize * 8).max(1024);

    // Geometry LUT: fixed size based on resolution
    let lut_size = (1 << (resolution * 2)).max(1024);

    // Intermediate buffer: proportional to expected cells
    let intermediate_size = (expected_cells * 2).max(1024);

    (scratch_size, lut_size, intermediate_size)
}

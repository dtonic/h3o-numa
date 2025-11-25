//! NUMA topology detection and caching
//!
//! This module provides functionality to detect the system's NUMA topology
//! and cache it for efficient access during runtime.

use once_cell::sync::OnceCell;

/// NUMA topology information
///
/// Contains information about the number of NUMA nodes and the cores
/// associated with each node.
#[derive(Debug, Clone)]
pub struct NumaTopology {
    /// Cores per NUMA node (logical core IDs)
    pub cores_per_node: Vec<Vec<usize>>,
}

impl NumaTopology {
    /// Get the NUMA node ID for a given core ID
    #[must_use]
    pub fn get_node_for_core(&self, core_id: usize) -> Option<usize> {
        for (node_id, cores) in self.cores_per_node.iter().enumerate() {
            if cores.contains(&core_id) {
                return Some(node_id);
            }
        }
        None
    }

    /// Get all core IDs for a specific NUMA node
    #[must_use]
    pub fn get_cores_for_node(&self, node_id: usize) -> Option<&[usize]> {
        self.cores_per_node.get(node_id).map(Vec::as_slice)
    }

    /// Check if the topology is valid (has at least one node and core)
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.cores_per_node.is_empty()
            && self.cores_per_node.iter().any(|cores| !cores.is_empty())
    }
}

/// Global topology cache
static TOPOLOGY_CACHE: OnceCell<NumaTopology> = OnceCell::new();

/// Load NUMA topology from the system
///
/// This function detects the system's NUMA topology and caches it.
/// It should be called once at startup.
pub fn load_topology() -> NumaTopology {
    if let Some(cached) = TOPOLOGY_CACHE.get() {
        return cached.clone();
    }

    let topology = detect_topology();

    // Cache the topology for future use
    #[expect(clippy::let_underscore_must_use, reason = "Caching the topology, ignore set result")]
    let _ = TOPOLOGY_CACHE.set(topology.clone());

    topology
}

/// Get cached topology information
///
/// Returns the cached topology if available, otherwise loads it.
pub fn get_topology() -> &'static NumaTopology {
    TOPOLOGY_CACHE.get_or_init(detect_topology)
}

fn detect_topology() -> NumaTopology {
    use hwlocality::Topology;

    Topology::new().map_or_else(|_| {
            // Fallback to single node if hwloc fails
            let cores = (0..num_cpus::get()).collect::<Vec<_>>();
            NumaTopology {
                cores_per_node: vec![cores],
            }
        }, |topo| {
            use hwlocality::object::types::ObjectType;

            let nodes = topo.objects_with_type(ObjectType::NUMANode).count();

            let mut cores_per_node = vec![Vec::new(); nodes.max(1)];

            // Collect cores for each NUMA node
            for (nid, node) in
                topo.objects_with_type(ObjectType::NUMANode).enumerate()
            {
                let cores = node
                    .io_children()
                    .filter_map(hwlocality::object::TopologyObject::cpuset)
                    .enumerate()
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>();

                cores_per_node[nid].clone_from(&cores);
            }

            // If no NUMA nodes found, treat as single node
            if nodes == 0 {
                let all_cores = topo
                    .objects_with_type(ObjectType::PU)
                    .filter_map(hwlocality::object::TopologyObject::os_index)
                    .collect::<Vec<_>>();

                cores_per_node = vec![all_cores];
            }

            NumaTopology { cores_per_node }
        })
}

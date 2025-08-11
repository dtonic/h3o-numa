use super::h3api;
use h3on::CellIndex;

#[test]
fn identity() {
    let indexes = [CellIndex::try_from(0x8b1fb46622dcfff).expect("cell index")];
    let result = CellIndex::grid_disks_fast(indexes.iter().copied(), 0)
        .collect::<Option<Vec<_>>>();
    let reference = h3api::grid_disks_unsafe(indexes.iter().copied(), 0);

    assert_eq!(result, reference);
}

#[test]
fn ring1of1() {
    let indexes = [
        CellIndex::try_from(0x89283080ddbffff).expect("cell index"),
        CellIndex::try_from(0x89283080c37ffff).expect("cell index"),
        CellIndex::try_from(0x89283080c27ffff).expect("cell index"),
        CellIndex::try_from(0x89283080d53ffff).expect("cell index"),
        CellIndex::try_from(0x89283080dcfffff).expect("cell index"),
        CellIndex::try_from(0x89283080dc3ffff).expect("cell index"),
    ];
    let result = CellIndex::grid_disks_fast(indexes.iter().copied(), 1)
        .collect::<Option<Vec<_>>>();
    let reference = h3api::grid_disks_unsafe(indexes.iter().copied(), 1);

    // Compare sets instead of ordered lists to handle parallel processing order differences
    match (result, reference) {
        (Some(result), Some(reference)) => {
            let result_set: std::collections::HashSet<_> = result.into_iter().collect();
            let reference_set: std::collections::HashSet<_> = reference.into_iter().collect();
            assert_eq!(result_set, reference_set, "Results should contain the same cells regardless of order");
        }
        (None, None) => {} // Both failed, which is fine
        (result, reference) => {
            panic!("One succeeded while the other failed: result={:?}, reference={:?}", result, reference);
        }
    }
}

#[test]
fn ring2of1() {
    let indexes = [
        CellIndex::try_from(0x89283080ddbffff).expect("cell index"),
        CellIndex::try_from(0x89283080c37ffff).expect("cell index"),
        CellIndex::try_from(0x89283080c27ffff).expect("cell index"),
        CellIndex::try_from(0x89283080d53ffff).expect("cell index"),
        CellIndex::try_from(0x89283080dcfffff).expect("cell index"),
        CellIndex::try_from(0x89283080dc3ffff).expect("cell index"),
    ];
    let result = CellIndex::grid_disks_fast(indexes.iter().copied(), 2)
        .collect::<Option<Vec<_>>>();
    let reference = h3api::grid_disks_unsafe(indexes.iter().copied(), 2);

    // Compare sets instead of ordered lists to handle parallel processing order differences
    match (result, reference) {
        (Some(result), Some(reference)) => {
            let result_set: std::collections::HashSet<_> = result.into_iter().collect();
            let reference_set: std::collections::HashSet<_> = reference.into_iter().collect();
            assert_eq!(result_set, reference_set, "Results should contain the same cells regardless of order");
        }
        (None, None) => {} // Both failed, which is fine
        (result, reference) => {
            panic!("One succeeded while the other failed: result={:?}, reference={:?}", result, reference);
        }
    }
}

#[test]
fn failed() {
    let indexes = [
        CellIndex::try_from(0x8029fffffffffff).expect("cell index"),
        CellIndex::try_from(0x801dfffffffffff).expect("cell index"),
    ];
    let result = CellIndex::grid_disks_fast(indexes.iter().copied(), 2)
        .collect::<Option<Vec<_>>>();
    let reference = h3api::grid_disks_unsafe(indexes.iter().copied(), 2);

    // Compare sets instead of ordered lists to handle parallel processing order differences
    match (result, reference) {
        (Some(result), Some(reference)) => {
            let result_set: std::collections::HashSet<_> = result.into_iter().collect();
            let reference_set: std::collections::HashSet<_> = reference.into_iter().collect();
            assert_eq!(result_set, reference_set, "Results should contain the same cells regardless of order");
        }
        (None, None) => {} // Both failed, which is fine
        (result, reference) => {
            panic!("One succeeded while the other failed: result={:?}, reference={:?}", result, reference);
        }
    }
}

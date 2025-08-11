use super::h3api;
use h3on::{CellIndex, Resolution};

macro_rules! test {
    ($name:ident, $compacted:expr, $resolution:literal) => {
        #[test]
        fn $name() {
            let compacted = $compacted
                .into_iter()
                .map(|value| CellIndex::try_from(value).expect("cell index"))
                .collect::<Vec<_>>();
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            let result =
                CellIndex::uncompact(compacted.iter().copied(), resolution)
                    .collect::<Vec<_>>();
            let reference = h3api::uncompact_cells(&compacted, resolution);

            // Compare sets instead of ordered lists to handle parallel processing order differences
            let result_set: std::collections::HashSet<_> = result.into_iter().collect();
            let reference_set: std::collections::HashSet<_> = reference.into_iter().collect();
            assert_eq!(result_set, reference_set, "Results should contain the same cells regardless of order");
        }
    };
}

test!(single_hexagon, vec![0x802bfffffffffff], 5);
test!(single_pentagon, vec![0x820807fffffffff], 5);
test!(
    mix,
    vec![0x802bfffffffffff, 0x820807fffffffff, 0x83734efffffffff],
    5
);

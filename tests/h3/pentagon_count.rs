use super::h3api;
use h3on::Resolution;

#[test]
fn value() {
    let result = Resolution::pentagon_count();
    let reference = h3api::pentagon_count();

    assert_eq!(result, reference);
}

use stain::{Store, create_stain, stain};

trait SuperMinimal {}

mod inner {
    use super::*;

    // Branch: pub(super) mod, NO ordering, NO prefix
    create_stain! {
        trait SuperMinimal;
        store: pub(super) mod sm_store;
    }
}

use inner::sm_store;

#[derive(Default)]
struct SmImpl;
impl SuperMinimal for SmImpl {}

stain! {
    store: sm_store;
    item: SmImpl;
    ordering: 0; // Default u64
}

#[test]
fn test_minimal_pub_super() {
    let store = inner::sm_store::Store::collect();
    assert_eq!(store.iter().count(), 1);
}

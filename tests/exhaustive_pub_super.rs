use stain::{create_stain, stain, Store};

trait SuperWorker {}

mod inner {
    use super::*;

    // Branch: pub(super) mod, explicit ordering, explicit prefix
    create_stain! {
        trait SuperWorker;
        ordering: u64;

        prefix: super_sys;
        store: pub(super) mod super_store;
    }
}

use inner::super_store;

#[derive(Default)]
struct SuperImpl;
impl SuperWorker for SuperImpl {}

stain! {
    store: super_store;
    item: SuperImpl;
    ordering: 0;
}

#[test]
fn test_exhaustive_pub_super() {
    let store = inner::super_store::Store::collect();
    assert_eq!(store.iter().count(), 1);
}

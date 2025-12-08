use stain::{create_stain, stain, Store};

trait CrateWorker {}

// Branch: pub(crate) mod, explicit ordering, explicit prefix
create_stain! {
    trait CrateWorker;
    ordering: u64;

    prefix: crate_sys;
    store: pub(crate) mod crate_store;
}

#[derive(Default)]
struct CrateImpl;
impl CrateWorker for CrateImpl {}

stain! {
    store: crate_store;
    item: CrateImpl;
    ordering: 0;
}

#[test]
fn test_exhaustive_pub_crate() {
    let store = crate_store::Store::collect();
    assert_eq!(store.iter().count(), 1);
}

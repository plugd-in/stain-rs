use stain::{create_stain, stain, Store};

trait MinimalPubCrate {}

// Branch: pub mod, NO ordering, NO prefix
create_stain! {
    trait MinimalPubCrate;
    store: pub(crate) mod min_pub_crate_store;
}

#[derive(Default)]
struct MinPubCrateImpl;
impl MinimalPubCrate for MinPubCrateImpl {}

stain! {
    store: min_pub_crate_store;
    item: MinPubCrateImpl;
    ordering: 0;
}

#[test]
fn test_minimal_pub() {
    let store = min_pub_crate_store::Store::collect();
    assert_eq!(store.iter().count(), 1);
}

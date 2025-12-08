use stain::{Store, create_stain, stain};

trait NoPrefixPubCrate {}

// Branch: pub(crate) mod, explicit ordering, NO prefix
create_stain! {
    trait NoPrefixPubCrate;
    ordering: u64;
    store: pub(crate) mod np_pub_crate_store;
}

#[derive(Default)]
struct NpPubCrateImpl;
impl NoPrefixPubCrate for NpPubCrateImpl {}

stain! {
    store: np_pub_crate_store;
    item: NpPubCrateImpl;
    ordering: 0;
}

#[test]
fn test_implicit_prefix_pub() {
    let store = np_pub_crate_store::Store::collect();
    assert_eq!(store.iter().count(), 1);
}

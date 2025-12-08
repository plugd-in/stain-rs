use stain::{create_stain, stain, Store};

pub trait NoPrefixPub {}

// Branch: pub mod, explicit ordering, NO prefix
create_stain! {
    trait NoPrefixPub;
    ordering: u64;
    store: pub mod np_pub_store;
}

#[derive(Default)]
struct NpPubImpl;
impl NoPrefixPub for NpPubImpl {}

stain! {
    store: np_pub_store;
    item: NpPubImpl;
    ordering: 0;
}

#[test]
fn test_implicit_prefix_pub() {
    let store = np_pub_store::Store::collect();
    assert_eq!(store.iter().count(), 1);
}

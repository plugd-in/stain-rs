use stain::{create_stain, stain, Store};

pub trait MinimalPub {}

// Branch: pub mod, NO ordering, NO prefix
create_stain! {
    trait MinimalPub;
    store: pub mod min_pub_store;
}

#[derive(Default)]
struct MinPubImpl;
impl MinimalPub for MinPubImpl {}

stain! {
    store: min_pub_store;
    item: MinPubImpl;
    ordering: 0;
}

#[test]
fn test_minimal_pub() {
    let store = min_pub_store::Store::collect();
    assert_eq!(store.iter().count(), 1);
}

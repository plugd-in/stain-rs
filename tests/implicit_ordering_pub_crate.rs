use stain::{create_stain, stain, Store};

trait DefOrderCrate {}

// Branch: pub(crate) mod, NO ordering, explicit prefix
create_stain! {
    trait DefOrderCrate;
    prefix: doc_sys;
    store: pub(crate) mod doc_store;
}

#[derive(Default)]
struct DocImpl;
impl DefOrderCrate for DocImpl {}

stain! {
    store: doc_store;
    item: DocImpl;
    ordering: 100;
}

#[test]
fn test_default_ordering_pub_crate() {
    let store = doc_store::Store::collect();
    assert_eq!(store.iter().count(), 1);
}

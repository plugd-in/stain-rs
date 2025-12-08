use stain::{create_stain, stain, Store};

trait SuperImpOrder {}

mod inner {
    use super::*;

    // Branch: pub(super) mod, NO ordering (default u64), explicit prefix
    create_stain! {
        trait SuperImpOrder;
        prefix: sio_pre;
        store: pub(super) mod sio_store;
    }
}

use inner::sio_store;

#[derive(Default)]
struct SioImpl;
impl SuperImpOrder for SioImpl {}

// Verify we can access the store from the parent module
stain! {
    store: sio_store;
    item: SioImpl;
    ordering: 100; // Must be u64
}

#[test]
fn test_implicit_ordering_pub_super() {
    let store = inner::sio_store::Store::collect();
    assert_eq!(store.iter().count(), 1);

    let item = store.iter().next().unwrap();
    assert_eq!(*item.ordering(), 100u64);
}

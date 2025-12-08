use stain::{create_stain, stain, Store};

trait SuperImpPrefix {}

mod inner {
    use super::*;

    // Branch: pub(super) mod, explicit ordering, NO prefix
    create_stain! {
        trait SuperImpPrefix;
        ordering: i32;
        store: pub(super) mod sip_store;
    }
}

use inner::sip_store;

#[derive(Default)]
struct SipImpl;
impl SuperImpPrefix for SipImpl {}

stain! {
    store: sip_store;
    item: SipImpl;
    ordering: -5;
}

#[test]
fn test_implicit_prefix_pub_super() {
    let store = inner::sip_store::Store::collect();
    assert_eq!(store.iter().count(), 1);

    let item = store.iter().next().unwrap();
    assert_eq!(*item.ordering(), -5);
}

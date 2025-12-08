use stain::{create_stain, stain, Store};

trait NoPrefixTrait {}

// Branch: mod (private), explicit ordering, NO prefix
create_stain! {
    trait NoPrefixTrait;
    ordering: u64;
    store: mod np_store;
}

#[derive(Default)]
struct NpImpl;
impl NoPrefixTrait for NpImpl {}

stain! {
    store: np_store;
    item: NpImpl;
    ordering: 0;
}

#[test]
fn test_implicit_prefix() {
    let store = np_store::Store::collect();
    assert_eq!(store.iter().count(), 1);
}

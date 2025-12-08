use stain::{Store, create_stain, stain};

trait MinimalSelf {}

// Branch: pub(self) mod, NO ordering, NO prefix
create_stain! {
    trait MinimalSelf;
    store: pub(self) mod min_self_store;
}

#[derive(Default)]
struct MinSelfImpl;
impl MinimalSelf for MinSelfImpl {}

stain! {
    store: min_self_store;
    item: MinSelfImpl;
    ordering: 0;
}

#[test]
fn test_minimal_pub_self() {
    let store = min_self_store::Store::collect();
    assert_eq!(store.iter().count(), 1);
}

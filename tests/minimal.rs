use stain::{create_stain, stain, Store};

trait MinimalTrait {
    fn hello(&self) -> &'static str;
}

// Branch: mod (private), NO ordering, NO prefix
create_stain! {
    trait MinimalTrait;
    store: mod min_store;
}

#[derive(Default)]
struct MinImpl;
impl MinimalTrait for MinImpl {
    fn hello(&self) -> &'static str { "min" }
}

stain! {
    store: min_store;
    item: MinImpl;
    ordering: 0; // Default u64
}

#[test]
fn test_minimal() {
    let store = min_store::Store::collect();

    assert_eq!(store.iter().count(), 1);
    assert_eq!(store.iter().next().unwrap().hello(), "min");
}

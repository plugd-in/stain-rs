use stain::{create_stain, stain, Store};

trait DefOrderTrait {}

// Branch: mod (private), NO ordering (default u64), explicit prefix
create_stain! {
    trait DefOrderTrait;
    prefix: do_sys;
    store: mod do_store;
}

#[derive(Default)]
struct DoImpl;
impl DefOrderTrait for DoImpl {}

stain! {
    store: do_store;
    item: DoImpl;
    ordering: 100; // Must be u64
}

#[test]
fn test_default_ordering() {
    let store = do_store::Store::collect();
    assert_eq!(store.iter().count(), 1);

    let item = store.iter().next().unwrap();
    assert_eq!(*item.ordering(), 100u64);
}

use stain::{create_stain, stain, Store};

pub trait PubWorker {
    fn work(&self);
}

// Branch: pub mod, explicit ordering, explicit prefix
create_stain! {
    trait PubWorker;
    ordering: i32;

    prefix: pub_sys;
    store: pub mod pub_store;
}

#[derive(Default)]
struct PubImpl;
impl PubWorker for PubImpl { fn work(&self) {} }

stain! {
    store: pub_store;
    item: PubImpl;
    ordering: -1;
}

#[test]
fn test_exhaustive_pub() {
    let store = pub_store::Store::collect();
    assert_eq!(store.iter().count(), 1);
}

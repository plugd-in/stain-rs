use stain::{create_stain, stain, Store};

trait Worker<T> {
    type Output;

    fn do_work(&self, input: T) -> Self::Output;
}

// Branch: mod (private/default), explicit ordering, explicit prefix, generics + GATs
create_stain! {
    trait Worker;
    ordering: u8;

    type i32;
    trait type Output = i32;

    prefix: priv_worker;
    store: mod worker_store;
}

#[derive(Default)]
struct WorkerImpl;

impl Worker<i32> for WorkerImpl {
    type Output = i32;

    fn do_work(&self, input: i32) -> i32 {
        input * 2
    }
}

stain! {
    store: worker_store;
    item: WorkerImpl;
    ordering: 1;
}

#[test]
fn test_exhaustive_private() {
    let store = worker_store::Store::collect();
    assert_eq!(store.iter().count(), 1);

    let worker = store.iter().next().unwrap();
    assert_eq!(worker.do_work(10), 20);
}

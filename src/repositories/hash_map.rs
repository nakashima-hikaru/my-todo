#[cfg(test)]
pub(crate) mod test_utils {
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use std::sync::{RwLockReadGuard, RwLockWriteGuard};

    use crate::repositories::todos::Todo;

    type TodoData = HashMap<i32, Todo>;

    #[derive(Debug, Clone)]
    pub(crate) struct HashMapRepository {
        store: Arc<RwLock<TodoData>>,
    }

    impl HashMapRepository {
        pub(crate) fn new() -> Self {
            HashMapRepository {
                store: Arc::default(),
            }
        }

        pub(crate) fn write_store_ref(&self) -> RwLockWriteGuard<TodoData> {
            self.store.write().unwrap()
        }

        pub(crate) fn read_store_ref(&self) -> RwLockReadGuard<TodoData> {
            self.store.read().unwrap()
        }
    }
}

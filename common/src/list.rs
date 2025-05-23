use scrypto::prelude::*;

pub type ListIndex = u64;
pub type ListIndexOffset = i64;

/**
 * A list data structure that can be used to store a list of items.
 */
#[derive(ScryptoSbor)]
pub struct List<T: ScryptoSbor + Clone> {
    pointer: ListIndex,
    kvs: KeyValueStore<ListIndex, T>,
}

impl<T: ScryptoSbor + Clone> List<T> {
    pub fn new<F>(create_fn: F) -> Self 
    where
        F: Fn() -> KeyValueStore<ListIndex, T>,
    {
        Self { 
            pointer: 0,
            kvs: create_fn(),
        }
    }
    
    pub fn push(&mut self, item: T) {
        self.kvs.insert(self.pointer, item);
        self.pointer += 1;
    }

    pub fn get(&self, index: ListIndex) -> Option<KeyValueEntryRef<T>> {
        self.kvs.get(&index)
    }

    pub fn get_mut(&mut self, index: ListIndex) -> Option<KeyValueEntryRefMut<T>> {
        self.kvs.get_mut(&index)
    }

    pub fn update(&mut self, index: ListIndex, item: T) {
        assert!(index < self.pointer, "Index out of bounds");
        self.kvs.insert(index, item);
    }

    pub fn clear(&mut self) {
        // Clear the list by removing all items
        // from the KeyValueStore
        for i in 0..self.pointer {
            self.kvs.remove(&i);
        }
        self.pointer = 0;
    }

    pub fn range(&self, start: ListIndex, end: ListIndex) -> Vec<T> {
        let mut result = Vec::new();
        for i in start..end {
            if let Some(item) = self.get(i) {
                result.push(item.to_owned());
            } else {
                break;
            }
        }
        result
    }

    pub fn len(&self) -> ListIndex {
        self.pointer
    }
}
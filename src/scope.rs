use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Debug)]
pub struct ScopeMap<K: Eq + Hash, V> {
    maps: Vec<HashMap<K, V>>,
}

impl<K: Eq + Hash, V> ScopeMap<K, V> {
    pub fn new() -> Self {
        Self { maps: Vec::new() }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        for m in self.maps.iter().rev() {
            if m.contains_key(key) {
                return m.get(key);
            }
        }

        None
    }

    pub fn insert(&mut self, key: K, value: V) {
        let m = self.maps.last_mut().unwrap();
        m.insert(key, value);
    }

    pub fn new_stack(&mut self) {
        self.maps.push(HashMap::new());
    }

    pub fn drop_stack(&mut self) {
        self.maps.pop();
    }
}

use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
pub struct Counter<K, V> {
    items: HashMap<K, HashMap<V, u32>>,
}

impl<K, V> Default for Counter<K, V> {
    fn default() -> Self {
        Self {
            items: Default::default(),
        }
    }
}

impl<K: Eq + Hash, V: Clone + Eq + Hash> Counter<K, V> {
    pub fn insert(&mut self, key: K, value: V) {
        let key_map = self.items.entry(key).or_insert_with(HashMap::new);
        *key_map.entry(value).or_insert(0) += 1;
    }

    pub fn most_common(&self, key: K) -> Option<V> {
        match self.items.get(&key) {
            Some(key_map) => {
                let mut values: Vec<(V, u32)> = key_map.iter().map(|(value, count)| (value.clone(), *count)).collect();
                values.sort_by_key(|&(_, count)| count);
                values.last().map(|(value, _count)| value.clone())
            }
            None => None,
        }
    }
}

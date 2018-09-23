use std::hash::{Hash, Hasher, BuildHasher};
use std::borrow::BorrowMut;
use std::mem;
use std::slice;
use std::iter::FromIterator;
use std::fmt::{Debug, Formatter, Result};
use std::ops::Index;

const MIN_DEFAULT_CAPACITY: usize = 32;

struct HashItem<K, V> {
    key: K,
    value: V,
    hash: u64,
    del: bool,
}

pub struct SimpleHashMap<K, V> {
    table: Vec<Option<HashItem<K, V>>>,
    capacity: usize,
    len: usize,
}

pub struct Iter<'a, K: 'a, V: 'a> {
    iter: slice::Iter<'a, Option<HashItem<K, V>>>,
    consumed: usize,
    current_len: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        loop {
            let iter_item_opt = self.iter.next();
            if iter_item_opt.is_none() {
                return None;
            }
            match iter_item_opt.unwrap() {
                Some(item) => {
                    if !item.del {
                        self.consumed += 1;
                        return Some((&item.key, &item.value))
                    }
                },
                None => continue
            }
        }
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {
    fn len(&self) -> usize {
        self.current_len - self.consumed
    }
}

pub struct Keys<'a, K: 'a, V: 'a> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;
    fn next(&mut self) -> Option<&'a K> {
        self.inner.next().map(|(k, _)| k)
    }
}

pub struct Values<'a, K: 'a, V: 'a> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<&'a V> {
        self.inner.next().map(|(_, v)| v)
    }
}

impl<K: Hash + Eq, V> SimpleHashMap<K, V> {

    pub fn new() -> SimpleHashMap<K, V> {
        SimpleHashMap {
            table: Vec::new(),
            capacity: 0,
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> SimpleHashMap<K, V> {
        let mut tab = Vec::with_capacity(capacity);
        for i in 0..capacity {
            tab.push(None);
        }
        assert_eq!(tab.capacity(), capacity);
        assert_eq!(tab.len(), capacity);
        SimpleHashMap {
            table: tab,
            capacity: capacity,
            len: 0,
        }
    }
}

///
/// Subset implementation of original HashMap functions
///
/// `TODO:`
/// support retain
/// support reserve
/// support expand
/// support resize
/// support get_mut
/// support entry
/// support clone
/// support shrink
/// support size_hint
impl<K, V> SimpleHashMap<K, V> where K: Hash + Eq {

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn keys(&self) -> Keys<K, V> {
        Keys { inner: self.iter() }
    }

    pub fn values(&self) -> Values<K, V> {
        Values { inner: self.iter() }
    }

    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            iter: self.table.iter(),
            consumed: 0,
            current_len: self.len,
        }
    }

    pub fn len(&self) -> usize { self.len }

    pub fn is_empty(&self) -> bool { self.len() == 0 }

    pub fn clear(&mut self) {
        self.len = 0;
        self.table.drain(..);
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        if self.capacity == 0 {
            return None;
        }

        match self.find_index(k) {
            Some(index) => {
                if let Some(ref item) = &self.table[index] {
                    Some(&item.value)
                } else {
                    panic!("find index but have no value")
                }
            },
            None => None,
        }
    }

    pub fn contains_key(&self, k: &K) -> bool {
        if self.capacity == 0 {
            return false;
        }
        match self.find_index(k) {
            Some(_) => true,
            _ => false
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {

        // allocate minimum capacity, lazy evaluation
        if self.capacity == 0 {
            self.capacity = MIN_DEFAULT_CAPACITY;
            self.table = Vec::with_capacity(MIN_DEFAULT_CAPACITY);
            for _ in 0..self.capacity {
                self.table.push(None);
            }
            self.len = 0;
            assert_eq!(self.len(), 0);
        }

        // make sure there's a usable place
        if self.len() >= self.capacity {
            panic!("insert failed, cache is full")
        }

        let mut state = self.build_hasher();
        k.hash(&mut state);
        let hash_value = state.finish();

        let origin = (hash_value as usize) % self.capacity;
//        println!("[insert] key hash value = {:x}, original position = {}", hash_value, origin);

        let new_item = HashItem {
            key: k,
            value: v,
            hash: hash_value,
            del: false
        };

        let mut idx = origin;
        while self.try_to_insert_at(idx, &new_item.key).is_none() {
            idx = (idx + 1) % self.capacity;
            if idx == origin {
                panic!("cache is full, but can not find a hole")
            }
        }

        let old_value = self.table.remove(idx);
        self.table.insert(idx, Some(new_item));
        self.len += 1;

        old_value.filter(|item|!item.del)
            .map_or(None::<V>, |old_item| Some(old_item.value))
    }

    fn try_to_insert_at(&self, index: usize, k: &K) -> Option<usize> {
        match &self.table[index] {
            None => {
                println!("find an empty position at {}", index);
                Some(index)
            },
            Some(item) if item.del => {
                println!("find position {} with deleted status", index);
                Some(index)
            },
            Some(item) if !item.del && &item.key == k => {
                println!("find position {} with existing key", index);
                Some(index)
            },
            _ => {
                None
            }
        }
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        if self.capacity == 0 {
            return None;
        }
        match self.find_index(k) {
            Some(index) => {
                let mut old_item_opt = self.table.remove(index);
                if let Some(mut item) = old_item_opt {
                    item.del = true;
                    let mut ret = None;
                    unsafe {
                        let old_value: V = mem::transmute_copy(&item.value);
                        ret = Some(old_value);
                    }
                    self.table.insert(index, Some(item));
                    self.len -= 1;
                    ret
                } else {
                    None
                }
            }
            None => None
        }
    }

    fn find_index(&self, k: &K) -> Option<usize> {
        let mut state = self.build_hasher();
        k.hash(&mut state);
        let hash_value = state.finish();
        let origin = (hash_value as usize) % self.capacity;
//        println!("[find_index] key hash value = {:x}, original position = {}",
//                 hash_value, origin);

        let mut idx = origin;
        loop {
            if self.exists(idx, k, hash_value) {
                return Some(idx);
            }
            idx = (idx + 1) % self.capacity;
            if idx == origin {
                break;
            }
        }
        None
    }

    fn exists(&self, index: usize, k: &K, hash: u64) -> bool {
        let item_opt = &self.table[index];
        if let Some(ref item) = item_opt {
            if &item.key == k && item.hash == hash && !item.del {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl<K, V> BuildHasher for SimpleHashMap<K, V> {
    type Hasher = SimpleRSHasher;

    fn build_hasher(&self) -> Self::Hasher {
        SimpleRSHasher(0)
    }
}

impl<'a, K, V> IntoIterator for &'a SimpleHashMap<K, V> where K: Hash + Eq {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<K, V> FromIterator<(K, V)> for SimpleHashMap<K, V> where K: Hash + Eq {

    fn from_iter<I: IntoIterator<Item=(K, V)>>(iter: I) -> SimpleHashMap<K, V> {
        let mut map = SimpleHashMap::new();
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

impl<K, V> PartialEq for SimpleHashMap<K, V>
    where K: Hash + Eq,
          V: PartialEq {

    fn eq(&self, other: &SimpleHashMap<K, V>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(
            |(key, value)| other.get(key).map_or(
                false, |v| *v == *value)
        )
    }
}

impl<K, V> Debug for SimpleHashMap<K, V>
    where K: Hash + Eq + Debug,
          V: Debug {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'a, K, V> Index<&'a K> for SimpleHashMap<K, V>
    where K: Hash + Eq + Sized {

    type Output = V;

    fn index(&self, key: &K) -> &V {
        self.get(key).expect("no entry found for key")
    }
}


///Use simple RS hash function.
///
///Refer to http://www.partow.net/programming/hashfunctions/
///
pub struct SimpleRSHasher(u64);

impl Hasher for SimpleRSHasher {

    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let b: u64 = 378551;
        let mut a: u64 = 63689;
        let mut hash: u64 = 0;

        for byte in bytes {
            hash = hash.wrapping_mul(a).wrapping_add(*byte as u64);
            a = a.wrapping_mul(b);
        }

        self.0 = hash;
    }
}

/// copy part of unit tests from HashMap
#[cfg(test)]
mod test_simple_map {
    use super::SimpleHashMap;
    use super::MIN_DEFAULT_CAPACITY;

    #[test]
    fn test_zero_capacities() {
        type HM = SimpleHashMap<i32, i32>;

        let m = HM::new();
        assert_eq!(m.capacity(), 0);

        let m = HM::with_capacity(0);
        assert_eq!(m.capacity(), 0);
    }

    #[test]
    fn test_create_capacity_zero() {
        let mut m = SimpleHashMap::with_capacity(0);

        assert!(m.insert(1, 1).is_none());

        assert!(m.contains_key(&1));
        assert!(!m.contains_key(&0));
    }

    #[test]
    fn test_insert() {
        let mut m = SimpleHashMap::new();
        assert_eq!(m.len(), 0);
        assert!(m.insert(1, 2).is_none());
        assert_eq!(m.len(), 1);
        assert!(m.insert(2, 4).is_none());
        assert_eq!(m.len(), 2);
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert_eq!(*m.get(&2).unwrap(), 4);
    }

    #[test]
    fn test_empty_remove() {
        let mut m: SimpleHashMap<i32, bool> = SimpleHashMap::new();
        assert_eq!(m.remove(&0), None);
    }

    #[test]
    fn test_empty_iter() {
        let mut m: SimpleHashMap<i32, bool> = SimpleHashMap::new();
        assert_eq!(m.keys().next(), None);
        assert_eq!(m.values().next(), None);
        assert_eq!(m.iter().next(), None);
        assert_eq!(m.len(), 0);
        assert!(m.is_empty());
    }

    #[test]
    fn test_lots_of_insertions() {
//        let mut m = SimpleHashMap::new();
        let mut m = SimpleHashMap::with_capacity(MIN_DEFAULT_CAPACITY*100);

        // Try this a few times to make sure we never screw up the hashmap's
        // internal state.
//        for _ in 0..10 {//TODO: take long time, improve later
        for _ in 0..10 {
            assert!(m.is_empty());

            for i in 1..1001 {
                assert!(m.insert(i, i).is_none());

                for j in 1..i + 1 {
                    let r = m.get(&j);
                    assert_eq!(r, Some(&j));
                }

                for j in i + 1..1001 {
                    let r = m.get(&j);
                    assert_eq!(r, None);
                }
            }

            for i in 1001..2001 {
                assert!(!m.contains_key(&i));
            }

            // remove forwards
            for i in 1..1001 {
                assert!(m.remove(&i).is_some());

                for j in 1..i + 1 {
                    assert!(!m.contains_key(&j));
                }

                for j in i + 1..1001 {
                    assert!(m.contains_key(&j));
                }
            }

            for i in 1..1001 {
                assert!(!m.contains_key(&i));
            }

            for i in 1..1001 {
                assert!(m.insert(i, i).is_none());
            }

            // remove backwards
            for i in (1..1001).rev() {
                assert!(m.remove(&i).is_some());

                for j in i..1001 {
                    assert!(!m.contains_key(&j));
                }

                for j in 1..i {
                    assert!(m.contains_key(&j));
                }
            }
        }
    }

    #[test]
    fn test_insert_overwrite() {
        let mut m = SimpleHashMap::new();
        assert!(m.insert(1, 2).is_none());
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert!(!m.insert(1, 3).is_none());
        assert_eq!(*m.get(&1).unwrap(), 3);
    }

    #[test]
    fn test_insert_conflicts() {
        let mut m = SimpleHashMap::with_capacity(4);
        assert!(m.insert(1, 2).is_none());
        assert!(m.insert(5, 3).is_none());
        assert!(m.insert(9, 4).is_none());

        assert_eq!(m.len(), 3);

        assert_eq!(*m.get(&9).unwrap(), 4);
        assert_eq!(*m.get(&5).unwrap(), 3);
        assert_eq!(*m.get(&1).unwrap(), 2);
    }

    #[test]
    fn test_conflict_remove() {
        let mut m = SimpleHashMap::with_capacity(4);
        assert!(m.insert(1, 2).is_none());
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert!(m.insert(5, 3).is_none());
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert_eq!(*m.get(&5).unwrap(), 3);
        assert!(m.insert(9, 4).is_none());
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert_eq!(*m.get(&5).unwrap(), 3);
        assert_eq!(*m.get(&9).unwrap(), 4);
        assert!(m.remove(&1).is_some());
        assert_eq!(*m.get(&9).unwrap(), 4);
        assert_eq!(*m.get(&5).unwrap(), 3);
    }

    #[test]
    fn test_is_empty() {
        let mut m = SimpleHashMap::with_capacity(4);
        assert!(m.insert(1, 2).is_none());
        assert!(!m.is_empty());
        assert!(m.remove(&1).is_some());
        assert!(m.is_empty());
    }

    #[test]
    fn test_remove() {
        let mut m = SimpleHashMap::new();
        m.insert(1, 2);
        assert_eq!(m.remove(&1), Some(2));
        assert_eq!(m.remove(&1), None);
    }

    //TODO: support auto resize
    /*
    #[test]
    fn test_iterate() {
        let mut m = SimpleHashMap::with_capacity(4);
        for i in 0..32 {
            assert!(m.insert(i, i*2).is_none());
        }
        assert_eq!(m.len(), 32);

        let mut observed: u32 = 0;

        for (k, v) in &m {
            assert_eq!(*v, *k * 2);
            observed |= 1 << *k;
        }
        assert_eq!(observed, 0xFFFF_FFFF);
    }
    */

    #[test]
    fn test_keys() {
        let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
        let map: SimpleHashMap<_, _> = vec.into_iter().collect();
        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&1));
        assert!(keys.contains(&2));
        assert!(keys.contains(&3));
    }

    #[test]
    fn test_values() {
        let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
        let map: SimpleHashMap<_, _> = vec.into_iter().collect();
        let values: Vec<_> = map.values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&'a'));
        assert!(values.contains(&'b'));
        assert!(values.contains(&'c'));
    }

    #[test]
    fn test_find() {
        let mut m = SimpleHashMap::new();
        assert!(m.get(&1).is_none());
        m.insert(1, 2);
        match m.get(&1) {
            None => panic!(),
            Some(v) => assert_eq!(*v, 2),
        }
    }

    #[test]
    fn test_eq() {
        let mut m1 = SimpleHashMap::new();
        m1.insert(1, 2);
        m1.insert(2, 3);
        m1.insert(3, 4);

        let mut m2 = SimpleHashMap::new();
        m2.insert(1, 2);
        m2.insert(2, 3);

        assert!(m1 != m2);

        m2.insert(3, 4);

        assert_eq!(m1, m2);
    }

    #[test]
    fn test_show() {
        let mut map = SimpleHashMap::new();
        let empty: SimpleHashMap<i32, i32> = SimpleHashMap::new();

        map.insert(1, 2);
        map.insert(3, 4);

        let map_str = format!("{:?}", map);

        assert!(map_str == "{1: 2, 3: 4}" ||
            map_str == "{3: 4, 1: 2}");
        assert_eq!(format!("{:?}", empty), "{}");
    }

    #[test]
    fn test_from_iter() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: SimpleHashMap<_, _> = xs.iter().cloned().collect();

        for &(k, v) in &xs {
            assert_eq!(map.get(&k), Some(&v));
        }
    }

    #[test]
    fn test_iter_len() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: SimpleHashMap<_, _> = xs.iter().cloned().collect();

        let mut iter = map.iter();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.len(), 3);
    }

    #[test]
    fn test_index() {
        let mut map = SimpleHashMap::new();

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        assert_eq!(map[&2], 1);
    }

    #[test]
    #[should_panic]
    fn test_index_nonexistent() {
        let mut map = SimpleHashMap::new();

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        map[&4];
    }

    // TODO: support auto resize
    /*
    #[test]
    fn test_capacity_not_less_than_len() {
        let mut a = SimpleHashMap::new();
        let mut item = 0;

        for _ in 0..116 {
            a.insert(item, 0);
            item += 1;
        }

        assert!(a.capacity() > a.len());

        let free = a.capacity() - a.len();
        for _ in 0..free {
            a.insert(item, 0);
            item += 1;
        }

        assert_eq!(a.len(), a.capacity());

        // Insert at capacity should cause allocation.
        a.insert(item, 0);
        assert!(a.capacity() > a.len());
    }
    */
}

//! Minimal custom HashMap for axstd

extern crate alloc;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
use core::cmp::Eq;
use axhal::misc::random;

// 新增：独立的 hash 计算函数，不借用 `self`
fn index_for_seed<Q: ?Sized + Hash>(seed: u64, key: &Q, cap: usize) -> usize {
    let mut h = core::hash::SipHasher::new_with_keys(seed, 0xdeadbeef);
    key.hash(&mut h);
    (h.finish() as usize) % cap
}

/// 一个简单的桶结构
struct Bucket<K, V> {
    key: K,
    value: V,
}

/// 自定义 HashMap，使用开放寻址
pub struct HashMap<K, V> {
    buckets: Vec<Option<Bucket<K, V>>>,
    len: usize,
    seed: u64, // 每个 HashMap 固定种子，避免每次 hash 时调用 random()
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    /// 新建一个 HashMap
    pub fn new() -> Self {
        let cap = 16; // 初始容量
        let buckets = (0..cap).map(|_| None).collect::<Vec<Option<Bucket<K, V>>>>();
        let seed = random() as u64;
        Self { buckets, len: 0, seed }
    }

    /// 返回一个不可变迭代器，遍历 (&K, &V)
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            buckets: &self.buckets,
            idx: 0,
        }
    }

    // 计算基于当前 seed 的哈希并取模容量
    fn index_for<Q: ?Sized + Hash>(&self, key: &Q, cap: usize) -> usize {
        index_for_seed(self.seed, key, cap)
    }

    fn hash<Q: ?Sized + Hash>(&self, key: &Q) -> usize {
        self.index_for(key, self.buckets.len())
    }

    // 扩容并重哈希所有元素
    fn grow(&mut self) {
        let old_cap = self.buckets.len();
        let new_cap = old_cap.checked_mul(2).unwrap_or(old_cap + 1);
        let mut new_buckets = (0..new_cap).map(|_| None).collect::<Vec<Option<Bucket<K, V>>>>();

        // 先把 seed 复制到局部变量，避免在迭代期间借用 `self`
        let seed = self.seed;

        for slot in self.buckets.iter_mut() {
            if let Some(bucket) = slot.take() {
                // 使用不借用 self 的 helper 来计算索引
                let mut idx = index_for_seed(seed, &bucket.key, new_cap);
                loop {
                    match &mut new_buckets[idx] {
                        None => {
                            new_buckets[idx] = Some(bucket);
                            break;
                        }
                        _ => {
                            idx = (idx + 1) % new_cap;
                        }
                    }
                }
            }
        }
        self.buckets = new_buckets;
    }

    /// 插入键值对（如果负载过高则扩容）
    pub fn insert(&mut self, key: K, value: V) {
        // 当负载 >= 70% 时扩容
        if self.len * 100 >= self.buckets.len() * 70 {
            self.grow();
        }

        let mut idx = self.hash(&key);
        loop {
            match &mut self.buckets[idx] {
                Some(bucket) if bucket.key == key => {
                    bucket.value = value;
                    return;
                }
                None => {
                    self.buckets[idx] = Some(Bucket { key, value });
                    self.len += 1;
                    return;
                }
                _ => {
                    idx = (idx + 1) % self.buckets.len();
                }
            }
        }
    }

    /// 获取值
    pub fn get(&self, key: &K) -> Option<&V> {
        let mut idx = self.hash(key);
        loop {
            match &self.buckets[idx] {
                Some(bucket) if bucket.key == *key => return Some(&bucket.value),
                None => return None,
                _ => {
                    idx = (idx + 1) % self.buckets.len();
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

/// 迭代器类型，遍历 HashMap 中的键值引用
pub struct Iter<'a, K, V> {
    buckets: &'a [Option<Bucket<K, V>>],
    idx: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.buckets.len() {
            if let Some(ref b) = self.buckets[self.idx] {
                self.idx += 1;
                return Some((&b.key, &b.value));
            }
            self.idx += 1;
        }
        None
    }
}
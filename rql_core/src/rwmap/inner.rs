use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

use left_right::aliasing::DropBehavior;

use super::{op::NoDrop, value::Value};

pub struct Inner<K, V, M, S, D = NoDrop>
where
    K: Eq + Hash,
    D: DropBehavior,
    S: BuildHasher,
{
    pub(super) data: HashMap<K, Value<V, D>, S>,
    pub(super) meta: M,
    pub(super) hasher: S,
    pub(super) ready: bool,
}

impl<K, V, M, S> Clone for Inner<K, V, M, S>
where
    K: Eq + Hash + Clone,
    S: BuildHasher + Clone,
    M: Clone,
{
    fn clone(&self) -> Self {
        assert!(self.data.is_empty());
        Self {
            data: HashMap::with_hasher(self.data.hasher().clone()),
            meta: self.meta.clone(),
            hasher: self.hasher.clone(),
            ready: self.ready,
        }
    }
}

impl<K, V, S> Inner<K, V, (), S>
where
    K: Eq + Hash,
    S: BuildHasher + Clone,
{
    pub(super) fn with_hasher(hasher: S) -> Self {
        Self {
            data: HashMap::with_hasher(hasher.clone()),
            meta: (),
            hasher,
            ready: false,
        }
    }
}

impl<K, V, M, S> Inner<K, V, M, S>
where
    K: Eq + Hash,
    S: BuildHasher + Clone,
{
    pub(super) fn with_meta_and_hasher(meta: M, hasher: S) -> Self {
        Self {
            data: HashMap::with_hasher(hasher.clone()),
            meta,
            hasher,
            ready: false,
        }
    }
}

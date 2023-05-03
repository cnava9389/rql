use hashbrown::HashMap;
use std::hash::{BuildHasher, Hash};

use left_right::aliasing::DropBehavior;

use super::{op::NoDrop, value::Value};

/// The underlying struct that contains the hashmap, meta, and hasher.
/// The V will be wrapped in a Value Wrapper type check [`Value`](crate::value::Value).
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
    /// default implementation that only takes a hasher and contains () as meta.
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
    /// takes meta and hasher. A more customizable option and should be considered
    /// over [`with_hasher`](crate::inner::Inner::with_hasher)
    pub(super) fn with_meta_and_hasher(meta: M, hasher: S) -> Self {
        Self {
            data: HashMap::with_hasher(hasher.clone()),
            meta,
            hasher,
            ready: false,
        }
    }
}

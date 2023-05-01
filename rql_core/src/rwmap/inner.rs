use std::{collections::HashMap, hash::BuildHasher};

use super::op::NoDropVal;

pub struct Inner<T, S>
where
    S: BuildHasher,
{
    pub(super) data: HashMap<String, T, S>,
    pub(super) meta: (),
}

impl<S> Clone for Inner<NoDropVal, S>
where
    S: BuildHasher + Clone,
{
    fn clone(&self) -> Self {
        assert!(self.data.is_empty());
        Self {
            data: HashMap::with_hasher(self.data.hasher().clone()),
            meta: self.meta.clone(),
        }
    }
}

impl<S> Inner<NoDropVal, S>
where
    S: BuildHasher,
{
    pub(super) fn with_hasher(hasher: S) -> Self {
        Self {
            data: HashMap::with_hasher(hasher),
            meta: (),
        }
    }
}

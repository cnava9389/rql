use std::{collections::hash_map::RandomState, hash::BuildHasher};

pub struct Options<M, S>
where
    S: BuildHasher,
{
    pub(super) meta: M,
    pub(super) hasher: S,
}

impl Default for Options<(), RandomState> {
    fn default() -> Self {
        Self {
            meta: (),
            hasher: RandomState::default(),
        }
    }
}

impl<M, S> Options<M, S>
where
    S: BuildHasher,
{
    pub fn new(meta: M, hasher: S) -> Self {
        Self { meta, hasher }
    }

    pub fn with_meta<M2>(self, meta: M2) -> Options<M2, S> {
        Options {
            meta,
            hasher: self.hasher,
        }
    }

    pub fn with_hasher<S2>(self, hasher: S2) -> Options<M, S2>
    where
        S2: BuildHasher + Clone,
    {
        Options {
            meta: self.meta,
            hasher,
        }
    }
}

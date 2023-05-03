use ahash::RandomState;
use hashbrown::HashMap;
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash};

use left_right::ReadGuard;

use super::inner::Inner;
use super::op::NoDrop;
use super::value::Value;

/// A live reference into the read half of an evmap.
///
/// As long as this lives, changes to the map being read cannot be published. If a writer attempts
/// to call [`WriteHandle::publish`], that call will block until this is dropped.
///
/// Since the map remains immutable while this lives, the methods on this type all give you
/// unguarded references to types contained in the map.
pub struct MapReadRef<'rh, K, V, M = (), S = RandomState>
where
    K: Hash + Eq,
    V: Eq,
    S: BuildHasher,
{
    pub(super) guard: ReadGuard<'rh, Inner<K, V, M, S>>,
}

impl<'rh, K, V, M, S> MapReadRef<'rh, K, V, M, S>
where
    K: Hash + Eq,
    V: Eq,
    S: BuildHasher,
{
    /// Iterate over all key + valuesets in the map.
    ///
    /// Be careful with this function! While the iteration is ongoing, any writer that tries to
    /// publish changes will block waiting on this reader to finish.
    pub fn iter(&self) -> ReadGuardIter<'_, K, V, S> {
        ReadGuardIter {
            iter: self.guard.data.iter(),
        }
    }

    /// Iterate over all keys in the map.
    ///
    /// Be careful with this function! While the iteration is ongoing, any writer that tries to
    /// publish changes will block waiting on this reader to finish.
    pub fn keys(&self) -> KeysIter<'_, K, V, S> {
        KeysIter {
            iter: self.guard.data.iter(),
        }
    }

    /// Iterate over all value sets in the map.
    ///
    /// Be careful with this function! While the iteration is ongoing, any writer that tries to
    /// publish changes will block waiting on this reader to finish.
    pub fn values(&self) -> ValuesIter<'_, K, V, S> {
        ValuesIter {
            iter: self.guard.data.iter(),
        }
    }

    /// Returns the number of non-empty keys present in the map.
    pub fn len(&self) -> usize {
        self.guard.data.len()
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.guard.data.is_empty()
    }

    /// Get the current meta value.
    pub fn meta(&self) -> &M {
        &self.guard.meta
    }

    /// Returns a reference to the values corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but `Hash` and `Eq` on the borrowed
    /// form *must* match those for the key type.
    ///
    /// Note that not all writes will be included with this read -- only those that have been
    /// published by the writer. If no publish has happened, or the map has been destroyed, this
    /// function returns `None`.
    pub fn get<'a, Q: ?Sized>(&'a self, key: &'_ Q) -> Option<&'a Value<V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.guard.data.get(key)
    }

    /// Returns true if the map contains any values for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but `Hash` and `Eq` on the borrowed
    /// form *must* match those for the key type.
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.guard.data.contains_key(key)
    }
}

/// An [`Iterator`] over keys and values in the evmap.
pub struct ReadGuardIter<'rg, K, V, S>
where
    K: Eq + Hash,
    V: Eq,
    S: BuildHasher,
{
    iter: <&'rg HashMap<K, Value<V, NoDrop>, S> as IntoIterator>::IntoIter,
}

impl<'rg, K, V, S> Iterator for ReadGuardIter<'rg, K, V, S>
where
    K: Eq + Hash,
    V: Eq + Hash,
    S: BuildHasher,
{
    type Item = (&'rg K, &'rg Value<V>);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, v)| (k, v))
    }
}

/// An [`Iterator`] over keys.
pub struct KeysIter<'rg, K, V, S>
where
    K: Eq + Hash,
    V: Eq,
    S: BuildHasher,
{
    iter: <&'rg HashMap<K, Value<V, NoDrop>, S> as IntoIterator>::IntoIter,
}

impl<'rg, K, V, S> Iterator for KeysIter<'rg, K, V, S>
where
    K: Eq + Hash,
    V: Eq,
    S: BuildHasher,
{
    type Item = &'rg K;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, _)| k)
    }
}

/// An [`Iterator`] over value sets.
pub struct ValuesIter<'rg, K, V, S>
where
    K: Eq + Hash,
    V: Eq,
    S: BuildHasher,
{
    iter: <&'rg HashMap<K, Value<V, NoDrop>, S> as IntoIterator>::IntoIter,
}

impl<'rg, K, V, S> Iterator for ValuesIter<'rg, K, V, S>
where
    K: Eq + Hash,
    V: Eq,
    S: BuildHasher,
{
    type Item = &'rg Value<V>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }
}

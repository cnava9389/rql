use std::{
    borrow::Borrow,
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash},
};

use left_right::{aliasing::Aliased, ReadGuard};

use super::{inner::Inner, mapguard::MapReadRef, op::Op, value::Value};

/// A handle that may be used to read from the eventually consistent map.
///
/// Note that any changes made to the map will not be made visible until the writer calls
/// [`publish`](crate::WriteHandle::publish). In other words, all operations performed on a
/// `ReadHandle` will *only* see writes to the map that preceeded the last call to `publish`.
pub struct ReadHandle<K, V, M = (), S = RandomState>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    pub(super) handle: left_right::ReadHandle<Inner<K, V, M, S>>,
}

impl<K, V, M, S> Clone for ReadHandle<K, V, M, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl<K, V, M, S> ReadHandle<K, V, M, S>
where
    K: Eq + Hash,
    V: Eq,
    M: Clone,
    S: BuildHasher,
{
    pub(super) fn new(handle: left_right::ReadHandle<Inner<K, V, M, S>>) -> Self {
        Self { handle }
    }

    /// Take out a guarded live reference to the read side of the map.
    ///
    /// This lets you perform more complex read operations on the map.
    ///
    /// While the reference lives, changes to the map cannot be published.
    ///
    /// If no publish has happened, or the map has been destroyed, this function returns `None`.
    ///
    /// See [`MapReadRef`].
    pub fn enter(&self) -> Option<MapReadRef<'_, K, V, M, S>> {
        let guard = self.handle.enter()?;
        if !guard.ready {
            return None;
        }
        Some(MapReadRef { guard })
    }

    /// Returns the number of non-empty keys present in the map.
    pub fn len(&self) -> usize {
        self.enter().map_or(0, |x| x.len())
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.enter().map_or(true, |x| x.is_empty())
    }

    /// Get the current meta value.
    pub fn meta(&self) -> Option<ReadGuard<'_, M>> {
        Some(ReadGuard::map(self.handle.enter()?, |inner| &inner.meta))
    }

    /// Internal version of `get_and`
    fn get_raw<Q: ?Sized>(&self, key: &Q) -> Option<ReadGuard<'_, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let inner = self.handle.enter()?;
        if !inner.ready {
            return None;
        }

        ReadGuard::try_map(inner, |inner| inner.data.get(key).map(AsRef::as_ref))
    }

    /// Returns a guarded reference to the values corresponding to the key.
    ///
    /// While the guard lives, changes to the map cannot be published.
    ///
    /// The key may be any borrowed form of the map's key type, but `Hash` and `Eq` on the borrowed
    /// form must match those for the key type.
    ///
    /// Note that not all writes will be included with this read -- only those that have been
    /// published by the writer. If no publish has happened, or the map has been destroyed, this
    /// function returns `None`.
    #[inline]
    pub fn get<'rh, Q: ?Sized>(&'rh self, key: &'_ Q) -> Option<ReadGuard<'rh, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        // call `borrow` here to monomorphize `get_raw` fewer times
        self.get_raw(key.borrow())
    }

    /// Returns a guarded reference to the values corresponding to the key along with the map
    /// meta.
    ///
    /// While the guard lives, changes to the map cannot be published.
    ///
    /// The key may be any borrowed form of the map's key type, but `Hash` and `Eq` on the borrowed
    /// form *must* match those for the key type.
    ///
    /// Note that not all writes will be included with this read -- only those that have been
    /// refreshed by the writer. If no refresh has happened, or the map has been destroyed, this
    /// function returns `None`.
    ///
    /// If no values exist for the given key, `Some(None, _)` is returned.
    pub fn meta_get<Q: ?Sized>(&self, key: &Q) -> Option<(Option<ReadGuard<'_, Value<V>>>, M)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let inner = self.handle.enter()?;
        if !inner.ready {
            return None;
        }
        let meta = inner.meta.clone();
        let res = ReadGuard::try_map(inner, |inner| inner.data.get(key));
        Some((res, meta))
    }

    /// Returns true if the [`WriteHandle`](crate::WriteHandle) has been dropped.
    pub fn was_dropped(&self) -> bool {
        self.handle.was_dropped()
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
        self.enter().map_or(false, |x| x.contains_key(key))
    }
}

/// A handle that may be used to modify the eventually consistent map.
///
/// Note that any changes made to the map will not be made visible to readers until
/// [`publish`](Self::publish) is called.
///
/// When the `WriteHandle` is dropped, the map is immediately (but safely) taken away from all
/// readers, causing all future lookups to return `None`.
pub struct WriteHandle<K, V, M = (), S = RandomState>
where
    K: Eq + Hash + Clone,
    S: BuildHasher + Clone,
    V: Eq,
    M: 'static + Clone,
{
    handle: left_right::WriteHandle<Inner<K, V, M, S>, Op<K, V, M>>,
    r_handle: ReadHandle<K, V, M, S>,
}

impl<K, V, M, S> WriteHandle<K, V, M, S>
where
    K: Eq + Hash + Clone,
    S: BuildHasher + Clone,
    V: Eq,
    M: 'static + Clone,
{
    pub(super) fn new(handle: left_right::WriteHandle<Inner<K, V, M, S>, Op<K, V, M>>) -> Self {
        let r_handle = ReadHandle::new(left_right::ReadHandle::clone(&*handle));
        Self { handle, r_handle }
    }

    /// Publish all changes since the last call to `publish` to make them visible to readers.
    ///
    /// This can take some time, especially if readers are executing slow operations, or if there
    /// are many of them.
    pub fn publish(&mut self) -> &mut Self {
        self.handle.publish();
        self
    }

    /// Returns true if there are changes to the map that have not yet been exposed to readers.
    pub fn has_pending(&self) -> bool {
        self.handle.has_pending_operations()
    }

    /// Set the metadata.
    ///
    /// Will only be visible to readers after the next call to [`publish`](Self::publish).
    pub fn set_meta(&mut self, meta: M) {
        self.add_op(Op::SetMeta(meta));
    }

    fn add_op(&mut self, op: Op<K, V, M>) -> &mut Self {
        self.handle.append(op);
        self
    }

    /// Add the given value to the value-bag of the given key.
    ///
    /// The updated value-bag will only be visible to readers after the next call to
    /// [`publish`](Self::publish).
    pub fn insert(&mut self, k: K, v: V) -> &mut Self {
        self.add_op(Op::Insert(k, Aliased::from(v)))
    }

    pub fn remove(&mut self, k: K) -> &mut Self {
        self.add_op(Op::Delete(k))
    }
}

// allow using write handle for reads
use std::ops::Deref;
impl<K, V, M, S> Deref for WriteHandle<K, V, M, S>
where
    K: Eq + Hash + Clone,
    S: BuildHasher + Clone,
    V: Eq + Hash,
    M: 'static + Clone,
{
    type Target = ReadHandle<K, V, M, S>;
    fn deref(&self) -> &Self::Target {
        &self.r_handle
    }
}

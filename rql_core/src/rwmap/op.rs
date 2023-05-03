use std::hash::{BuildHasher, Hash};

use super::{inner::Inner, value::Value};
use left_right::{
    aliasing::{Aliased, DropBehavior},
    Absorb,
};

/// struct that is used to let the [`Inner`](crate::inner::Inner) struct know not to drop value.
#[derive(Debug)]
pub struct NoDrop;
impl DropBehavior for NoDrop {
    const DO_DROP: bool = false;
}

/// struct that indicates value should be dropped from [`Inner`](crate::inner::Inner).
pub(super) struct DoDrop;
impl DropBehavior for DoDrop {
    const DO_DROP: bool = true;
}

type NoDropVal<T> = Aliased<T, NoDrop>;

/// enum that represents list of operations that [`Inner`](crate::inner::Inner) will apply
/// to underlying maps
pub(super) enum Op<K, V, M> {
    Insert(K, NoDropVal<V>),
    Delete(K),
    SetMeta(M),
    MarkReady,
}

impl<K, V, M, S> Absorb<Op<K, V, M>> for Inner<K, V, M, S>
where
    K: Eq + Hash + Clone,
    V: Eq,
    S: BuildHasher + Clone,
    M: Clone,
{
    fn absorb_first(&mut self, operation: &mut Op<K, V, M>, _other: &Self) {
        match operation {
            Op::Insert(k, v) => {
                self.data
                    .insert(k.to_owned(), Value::new(unsafe { v.alias() }));
            }
            Op::Delete(k) => {
                self.data.remove(k);
            }
            Op::SetMeta(m) => {
                self.meta = m.clone();
            }
            Op::MarkReady => {
                self.ready = true;
            }
        };
    }

    fn sync_with(&mut self, first: &Self) {
        assert_eq!(self.data.len(), 0);
        let inner: &mut Inner<K, V, M, S, DoDrop> = unsafe { &mut *(self as *mut _ as *mut _) };
        inner.data.extend(
            first
                .data
                .iter()
                .map(|(k, v)| (k.to_owned(), unsafe { Value::alias(v) })),
        );
        self.ready = true;
    }

    fn absorb_second(&mut self, operation: Op<K, V, M>, _other: &Self) {
        let with_drop: &mut Inner<K, V, M, S, DoDrop> = unsafe { &mut *(self as *mut _ as *mut _) };
        match operation {
            Op::Insert(k, v) => {
                with_drop
                    .data
                    .insert(k, Value::new(unsafe { v.change_drop() }));
            }
            Op::Delete(ref k) => {
                with_drop.data.remove(k);
            }
            Op::SetMeta(m) => {
                with_drop.meta = m;
            }
            Op::MarkReady => {
                with_drop.ready = true;
            }
        };
    }

    fn drop_first(self: Box<Self>) {}

    fn drop_second(self: Box<Self>) {
        // Convert self to DoDrop and drop it.
        let with_drop: Box<Inner<K, V, M, S, DoDrop>> =
            unsafe { Box::from_raw(Box::into_raw(self) as *mut _ as *mut _) };
        drop(with_drop);
    }
}

use std::hash::BuildHasher;

use super::inner::Inner;
use left_right::{
    aliasing::{Aliased, DropBehavior},
    Absorb,
};
use serde_json::Value;

pub struct NoDrop;
impl DropBehavior for NoDrop {
    const DO_DROP: bool = false;
}

pub(super) struct DoDrop;
impl DropBehavior for DoDrop {
    const DO_DROP: bool = true;
}

pub(super) type NoDropVal = Aliased<Value, NoDrop>;
type DropVal = Aliased<Value, DoDrop>;

/// change this to be private
pub enum Op {
    Insert(String, NoDropVal),
    Delete(String),
    Update(String, NoDropVal),
}

impl<S> Absorb<Op> for Inner<NoDropVal, S>
where
    S: BuildHasher,
{
    fn absorb_first(&mut self, operation: &mut Op, _other: &Self) {
        match operation {
            Op::Insert(k, v) => {
                self.data.insert(k.to_owned(), unsafe { v.alias() });
            }
            Op::Delete(k) => {
                self.data.remove(k);
            }
            Op::Update(_, _) => todo!(),
        };
    }

    fn sync_with(&mut self, first: &Self) {
        assert_eq!(self.data.len(), 0);
        self.data.extend(
            first
                .data
                .iter()
                .map(|(k, v)| (k.to_owned(), unsafe { v.alias() })),
        );
    }

    fn absorb_second(&mut self, operation: Op, _other: &Self) {
        let with_drop: &mut Inner<DropVal, S> = unsafe { &mut *(self as *mut _ as *mut _) };
        match operation {
            Op::Insert(k, v) => {
                with_drop.data.insert(k, unsafe { v.change_drop() });
            }
            Op::Delete(ref k) => {
                self.data.remove(k);
            }
            Op::Update(_, _) => todo!(),
        };
    }

    fn drop_first(self: Box<Self>) {}

    fn drop_second(self: Box<Self>) {
        // Convert self to DoDrop and drop it.
        let with_drop: Box<Inner<DropVal, S>> =
            unsafe { Box::from_raw(Box::into_raw(self) as *mut _ as *mut _) };
        drop(with_drop);
    }
}

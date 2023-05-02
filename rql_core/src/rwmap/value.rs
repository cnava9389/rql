use super::op::{DoDrop, NoDrop};
use left_right::aliasing::{Aliased, DropBehavior};

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct Value<T, D = NoDrop>(pub(super) Aliased<T, D>)
where
    D: DropBehavior;

impl<T, D> Value<T, D>
where
    D: DropBehavior,
{
    pub(super) fn new(val: Aliased<T, D>) -> Self {
        Value(val)
    }
}

impl<T> Value<T, DoDrop>
where
    T: Eq,
{
    pub(crate) unsafe fn alias(other: &Value<T, NoDrop>) -> Self {
        Value(other.0.alias().change_drop())
    }
}

impl<T> AsRef<T> for Value<T, NoDrop> {
    fn as_ref(&self) -> &T {
        // safety: Values is #[repr(transparent)]
        unsafe { &*(self as *const _ as *const T) }
    }
}

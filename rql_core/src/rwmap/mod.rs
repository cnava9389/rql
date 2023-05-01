mod inner;
mod op;
mod options;

use std::{collections::hash_map::RandomState, hash::BuildHasher};

use inner::Inner;
use left_right::{ReadHandle, WriteHandle};
use op::{NoDropVal, Op};
use options::Options;

pub struct RwMap;

impl RwMap {
    pub fn with_options<M, S>(
        options: Options<M, S>,
    ) -> (
        WriteHandle<Inner<NoDropVal, S>, Op>,
        ReadHandle<Inner<NoDropVal, S>>,
    )
    where
        S: BuildHasher + Clone,
    {
        let inner = Inner::with_hasher(options.hasher);
        left_right::new_from_empty(inner)
    }

    fn default() -> (
        WriteHandle<Inner<NoDropVal, RandomState>, Op>,
        ReadHandle<Inner<NoDropVal, RandomState>>,
    ) {
        left_right::new_from_empty(Inner::with_hasher(Options::default().hasher))
    }
}

#[cfg(test)]
mod test {
    use left_right::aliasing::Aliased;
    use serde_json::{json, Value};

    use super::*;
    #[test]
    fn is_empty() {
        let (_w, r) = RwMap::default();
        {
            assert!(r.enter().unwrap().data.is_empty());
        }
    }

    #[test]
    fn check_one_item() {
        let (mut w, r) = RwMap::default();
        {
            assert!(r.enter().unwrap().data.is_empty());
        }
        w.append(Op::Insert("test".into(), Aliased::from(json!(1))));
        w.publish();
        {
            let read_guard = r.enter().unwrap();
            assert_eq!(read_guard.data.len(), 1);
            let val: &Value = read_guard.data.get("test").unwrap().as_ref();
            assert_eq!(val.as_u64().unwrap(), 1);
        }
    }

    #[test]
    fn check_two_items() {
        let (mut w, r) = RwMap::default();
        {
            assert!(r.enter().unwrap().data.is_empty());
        }
        w.append(Op::Insert("test1".into(), Aliased::from(json!(1))));
        w.append(Op::Insert("test2".into(), Aliased::from(json!("2"))));
        w.publish();
        {
            let read_guard = r.enter().unwrap();
            assert_eq!(read_guard.data.len(), 2);
            let val: &Value = read_guard.data.get("test1").unwrap().as_ref();
            assert_eq!(val.as_u64().unwrap(), 1);
            let val: &Value = read_guard.data.get("test2").unwrap().as_ref();
            assert_eq!(val.as_str().unwrap(), "2");
        }
    }

    #[test]
    fn delete_item() {
        let (mut w, r) = RwMap::default();
        {
            assert!(r.enter().unwrap().data.is_empty());
        }
        w.append(Op::Insert("test1".into(), Aliased::from(json!(1))));
        w.append(Op::Insert("test2".into(), Aliased::from(json!("2"))));
        w.publish();
        {
            let read_guard = r.enter().unwrap();
            assert_eq!(read_guard.data.len(), 2);
            let val: &Value = read_guard.data.get("test1").unwrap().as_ref();
            assert_eq!(val.as_u64().unwrap(), 1);
            let val: &Value = read_guard.data.get("test2").unwrap().as_ref();
            assert_eq!(val.as_str().unwrap(), "2");
        }
        w.append(Op::Delete("test1".into()));
        w.publish();
        {
            let read_guard = r.enter().unwrap();
            assert_eq!(read_guard.data.len(), 1);
            let val = read_guard.data.get("test1");
            assert_eq!(val, None);
        }
    }
}

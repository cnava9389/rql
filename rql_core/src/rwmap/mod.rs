mod handles;
mod inner;
mod mapguard;
mod op;
mod value;

use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash},
};

use handles::{ReadHandle, WriteHandle};
use inner::Inner;

pub struct RwMap;

impl RwMap {
    pub fn maybe_with_meta_and_hasher<K, V, M, S>(
        meta: Option<M>,
        hasher: Option<S>,
    ) -> (WriteHandle<K, V, M, S>, ReadHandle<K, V, M, S>)
    where
        K: Eq + Hash + Clone,
        V: Eq,
        S: BuildHasher + Clone + Default,
        M: Clone + Default,
    {
        let inner =
            Inner::with_meta_and_hasher(meta.unwrap_or_default(), hasher.unwrap_or_default());

        let (mut w, r) = left_right::new_from_empty(inner);
        w.append(op::Op::MarkReady);
        (WriteHandle::new(w), ReadHandle::new(r))
    }

    fn default<K, V>() -> (
        WriteHandle<K, V, (), RandomState>,
        ReadHandle<K, V, (), RandomState>,
    )
    where
        V: Eq,
        K: Eq + Hash + Clone,
    {
        let (mut w, r) = left_right::new_from_empty(Inner::with_hasher(RandomState::new()));
        w.append(op::Op::MarkReady);
        (WriteHandle::new(w), ReadHandle::new(r))
    }
}

#[cfg(test)]
mod test {
    use serde_json::{json, Value as json_value};

    use super::*;
    #[test]
    fn is_empty() {
        let (_w, r) = RwMap::default::<String, json_value>();
        {
            assert!(r.enter().is_none());
        }
    }

    #[test]
    fn check_one_item() {
        let (mut w, r) = RwMap::default::<String, json_value>();
        {
            assert!(r.enter().is_none());
        }
        w.insert("test".into(), json!(1));
        w.publish();
        {
            let read_guard = r.enter().unwrap();
            assert_eq!(read_guard.len(), 1);
            let val = read_guard.get("test").unwrap().as_ref();
            assert_eq!(val.as_u64().unwrap(), 1);
        }
    }

    #[test]
    fn check_two_items() {
        let (mut w, r) = RwMap::default::<String, json_value>();
        {
            assert!(r.enter().is_none());
        }
        w.insert("test1".into(), json!(1));
        w.insert("test2".into(), json!("2"));
        w.publish();
        {
            let read_guard = r.enter().unwrap();
            assert_eq!(read_guard.len(), 2);
            let val = read_guard.get("test1").unwrap().as_ref();
            assert_eq!(val.as_u64().unwrap(), 1);
            let val = read_guard.get("test2").unwrap().as_ref();
            assert_eq!(val.as_str().unwrap(), "2");
        }
    }

    #[test]
    fn delete_item() {
        let (mut w, r) = RwMap::default::<String, json_value>();
        w.publish();
        {
            assert!(r.is_empty());
        }
        w.insert("test1".into(), json!(1));
        w.insert("test2".into(), json!("2"));
        w.publish();
        {
            assert_eq!(r.len(), 2);
            let val = r.get("test1").unwrap();
            assert_eq!(val.as_u64().unwrap(), 1);
            let val = r.get("test2").unwrap();
            assert_eq!(val.as_str().unwrap(), "2");
        }
        w.remove("test2".to_owned());
        w.publish();
        {
            assert_eq!(r.len(), 1);
            let val = r.get("test2");
            assert!(val.is_none());
        }
    }
}

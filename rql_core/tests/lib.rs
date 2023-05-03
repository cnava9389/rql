#![allow(unused_macros)]
extern crate rql_core;
use rql_core::rwmap::RwMap;

// tests provided by [`evmap`](crate::evmap)

macro_rules! assert_match {
    ($x:expr, $p:pat) => {
        if let $p = $x {
        } else {
            panic!(concat!(stringify!($x), " did not match ", stringify!($p)));
        }
    };
}

#[test]
#[cfg_attr(miri, ignore)]
// https://github.com/rust-lang/miri/issues/658
fn paniced_reader_doesnt_block_writer() {
    let (mut w, r) = RwMap::default::<usize, &str>();
    w.insert(1, "a");
    w.publish();

    // reader panics
    let r = std::panic::catch_unwind(move || r.get(&1).map(|_| panic!()));
    assert!(r.is_err());

    // writer should still be able to continue
    w.insert(1, "b");
    w.publish();
    w.publish();
}

#[test]
fn read_after_drop() {
    let x = ('x', 42);

    let (mut w, r) = RwMap::default::<char, (char, usize)>();
    w.insert(x.0, x);
    w.publish();
    assert!(r.get(&x.0).is_some());

    // once we drop the writer, the readers should see empty maps
    drop(w);
    assert!(r.get(&x.0).is_none());
}

#[test]
fn clone_types() {
    let x = b"xyz";

    let (mut w, r) = RwMap::default::<&[u8; 3], &[u8; 3]>();
    w.insert(&*x, x);
    w.publish();

    assert!(r.get(&*x).is_some());

    assert_eq!(r.get(&*x).map(|v| *v == x), Some(true));
}

#[test]
#[cfg_attr(miri, ignore)]
fn busybusybusy_fast() {
    busybusybusy_inner(false);
}
#[test]
#[cfg_attr(miri, ignore)]
fn busybusybusy_slow() {
    busybusybusy_inner(true);
}

fn busybusybusy_inner(slow: bool) {
    use std::thread;
    use std::time;

    let threads = 4;
    let mut n = 1000;
    if !slow {
        n *= 100;
    }
    let (mut w, r) = RwMap::default::<usize, usize>();
    w.publish();

    let rs: Vec<_> = (0..threads)
        .map(|_| {
            let r = r.clone();
            thread::spawn(move || {
                // rustfmt
                for i in 0..n {
                    let i = i.into();
                    loop {
                        let map = r.enter().unwrap();
                        let rs = map.get(&i);
                        if rs.is_some() && slow {
                            thread::sleep(time::Duration::from_millis(2));
                        }
                        match rs {
                            Some(rs) => {
                                assert_eq!(rs.as_ref(), &i);
                                break;
                            }
                            None => {
                                thread::yield_now();
                            }
                        }
                    }
                }
            })
        })
        .collect();

    for i in 0..n {
        w.insert(i, i);
        w.publish();
    }

    for r in rs {
        r.join().unwrap();
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn busybusybusy_heap() {
    use std::thread;

    let threads = 2;
    let n = 1000;
    let (mut w, r) = RwMap::default::<_, Vec<_>>();
    w.publish();

    let rs: Vec<_> = (0..threads)
        .map(|_| {
            let r = r.clone();
            thread::spawn(move || {
                for i in 0..n {
                    let i = i.into();
                    loop {
                        let map = r.enter().unwrap();
                        let rs = map.get(&i);
                        match rs {
                            Some(_rs) => {
                                break;
                            }
                            None => {
                                thread::yield_now();
                            }
                        }
                    }
                }
            })
        })
        .collect();

    for i in 0..n {
        w.insert(i, (0..i).collect());
        w.publish();
    }

    for r in rs {
        r.join().unwrap();
    }
}

#[test]
fn minimal_query() {
    let (mut w, r) = RwMap::default::<usize, &str>();
    w.insert(1, "a");
    w.publish();
    w.insert(1, "b");

    assert_eq!(r.get(&1).map(|rs| rs.len()), Some(1));
    assert!(r.get(&1).map(|rs| rs.as_ref() == &"a").unwrap());
}

#[test]
fn non_copy_values() {
    let (mut w, r) = RwMap::default::<usize, String>();
    w.insert(1, "a".to_string());
    assert_eq!(r.get(&1).map(|rs| rs.len()), None);

    w.publish();

    assert!(r.get(&1).is_some());
    assert!(r.get(&1).map(|rs| { rs.as_ref() == "a" }).unwrap());

    w.insert(1, "b".to_string());
    assert_eq!(r.get(&1).map(|rs| rs.len()), Some(1));
    assert!(r.get(&1).map(|rs| { rs.as_ref() == "a" }).unwrap());
}

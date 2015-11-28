use libc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::super::{Accessible, native_id};

pub type NativeRef = usize;
pub type NativeRefCache = HashMap<*const libc::c_void, usize>;

pub fn to_native_ref(accessible: Arc<Accessible>, cache: Arc<Mutex<NativeRefCache>>,) -> usize {
    let aid = native_id(&accessible);
    if let Some(cached) = cache.lock().unwrap().get(&aid) {
        return *cached;
    }

    let id = 0;
    cache.lock().unwrap().insert(aid, id);
    id
}
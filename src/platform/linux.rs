use dbus::{MessageItem, Message, Connection, MessageType, BusType};
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

pub fn init() {
    let c = Connection::get_private(BusType::Session).unwrap();
    let m = Message::new_method_call("org.a11y.Bus",
				     "/org/a11y/bus",
				     "org.freedesktop.DBus.Properties",
				     "Get")
      .unwrap()
      .append("org.a11y.Status")
      .append("IsEnabled");
    let r = c.send_with_reply_and_block(m, 1000).unwrap();
    let reply = r.get_items();

    assert!(r.msg_type() == MessageType::MethodReturn);
    assert!(reply.len() == 1);
    if let MessageItem::Variant(ref inner) = reply[0] {
	if let &MessageItem::Bool(b) = &**inner {
	    println!("{}", b);
	} else {
	    panic!("unexpected return type: {:?}", reply[0]);
	}
    } else {
        panic!("unexpected return type: {:?}", reply[0]);
    }
}

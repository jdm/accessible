#![feature(raw)]

#[cfg(target_os = "macos")]
extern crate cocoa;
#[cfg(target_os = "linux")]
extern crate dbus;
extern crate libc;
#[macro_use]
extern crate log;
#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

mod platform;

pub use platform::implementation::{NativeRef, NativeRefCache};
use std::mem;
use std::raw::TraitObject;
use std::sync::{Arc, Mutex};

pub enum Role {
    Group,
    StaticText,
    Image,
    Button,
    Window,
    ScrollArea,
}

pub enum Parent {
    Native(NativeRef),
    Accessible(Arc<dyn Accessible>),
}

pub struct Position {
    pub x: f32,
    pub y: f32,
}

pub struct Dimensions {
    pub w: f32,
    pub h: f32,
}

pub trait Accessible {
    fn parent(&self) -> Parent;
    fn children(&self) -> Vec<Arc<dyn Accessible>>;
    fn role(&self) -> Role;
    fn title(&self) -> Option<String>;
    fn value(&self) -> Option<String>;
    fn position(&self) -> Position;
    fn dimensions(&self) -> Dimensions;
}

fn native_id(accessible: &Arc<dyn Accessible>) -> *const libc::c_void {
    unsafe {
        mem::transmute::<&dyn Accessible, TraitObject>(&**accessible as &dyn Accessible).data
            as *const libc::c_void
    }
}

pub fn to_native_ref<T: Accessible + 'static>(
    accessible: Arc<T>,
    cache: Arc<Mutex<NativeRefCache>>,
) -> NativeRef {
    platform::implementation::to_native_ref(accessible as Arc<dyn Accessible>, cache)
}

pub fn init() {
    platform::implementation::init();
}

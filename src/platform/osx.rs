use super::super::{native_id, Accessible, Parent, Role};
use cocoa::base::{id, nil, BOOL};
use cocoa::foundation::{NSArray, NSPoint, NSSize, NSString, NSValue};
use libc;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::{Arc, Mutex, Once};

pub type NativeRef = id;

pub type NativeRefCache = HashMap<*const libc::c_void, id>;

struct AccessibleState {
    accessible: Arc<dyn Accessible>,
    cache: Arc<Mutex<NativeRefCache>>,
}

pub fn to_native_ref(accessible: Arc<dyn Accessible>, cache: Arc<Mutex<NativeRefCache>>) -> id {
    let aid = native_id(&accessible);
    if let Some(cached) = cache.lock().unwrap().get(&aid) {
        return *cached;
    }

    let state = AccessibleState {
        accessible,
        cache: cache.clone(),
    };
    unsafe {
        let id: id = msg_send![class(), new];
        (&mut *id).set_ivar(
            "native",
            Box::into_raw(Box::new(state)) as *mut libc::c_void,
        );
        let id = msg_send![id, retain];
        cache.lock().unwrap().insert(aid, id);
        debug!("returning {:?}", id);
        id
    }
}

trait ToAXRole {
    fn to_axrole(&self) -> &'static str;
}

impl ToAXRole for Role {
    fn to_axrole(&self) -> &'static str {
        match *self {
            Role::Group => "AXGroup",
            Role::StaticText => "AXStaticText",
            Role::Image => "AXImage",
            Role::Button => "AXButton",
            Role::Window => "AXWindow",
            Role::ScrollArea => "AXScrollArea",
        }
    }
}

fn cache(this: &Object) -> Arc<Mutex<NativeRefCache>> {
    unsafe {
        let state: *mut libc::c_void = *this.get_ivar("native");
        let state = state as *mut AccessibleState;
        let state = &*state;
        state.cache.clone()
    }
}

fn native(this: &Object) -> Arc<dyn Accessible> {
    unsafe {
        let state: *mut libc::c_void = *this.get_ivar("native");
        let state = state as *mut AccessibleState;
        let state = &*state;
        state.accessible.clone()
    }
}

fn class() -> *const Class {
    extern "C" fn accessibility_attribute_names(this: &Object, _: Sel) -> id {
        println!("getting a11y attribute names for {:?}", this);
        //TODO: make this static
        unsafe {
            let attributes = vec![
                NSString::alloc(nil).init_str("AXChildren"),
                NSString::alloc(nil).init_str("AXParent"),
                NSString::alloc(nil).init_str("AXRole"),
                NSString::alloc(nil).init_str("AXTitle"),
                NSString::alloc(nil).init_str("AXValue"),
                NSString::alloc(nil).init_str("AXSize"),
                NSString::alloc(nil).init_str("AXPosition"),
                /*NSString::alloc(nil).init_str("AXSubrole"),
                NSString::alloc(nil).init_str("AXRoleDescription"),
                NSString::alloc(nil).init_str("AXAccessibilityEnabled"),
                NSString::alloc(nil).init_str("AXWindow"),
                NSString::alloc(nil).init_str("AXFocused"),
                NSString::alloc(nil).init_str("AXHelp"),
                NSString::alloc(nil).init_str("AXTitleUIElement"),
                NSString::alloc(nil).init_str("AXTopLevelUIElement")*/
            ];
            NSArray::arrayWithObjects(nil, &attributes)
        }
    }

    extern "C" fn accessibility_attribute_value(this: &Object, _: Sel, attribute: id) -> id {
        unsafe {
            info!(
                "accessible {:?}: {:?}",
                this,
                CStr::from_ptr(attribute.UTF8String())
            );

            if attribute.isEqualToString("AXChildren") {
                let children: Vec<id> = native(this)
                    .children()
                    .into_iter()
                    .map(|child| to_native_ref(child, cache(this)))
                    .collect();
                debug!("returning {:?}", children);
                return NSArray::arrayWithObjects(nil, &children);
            }

            if attribute.isEqualToString("AXParent") {
                return match native(this).parent() {
                    Parent::Accessible(parent) => to_native_ref(parent, cache(this)),
                    Parent::Native(parent) => parent,
                };
            }

            if attribute.isEqualToString("AXRole") {
                return NSString::alloc(nil).init_str(&native(this).role().to_axrole());
            }

            if attribute.isEqualToString("AXTitle") {
                return match native(this).title() {
                    Some(title) => NSString::alloc(nil).init_str(&title),
                    None => nil,
                };
            }

            if attribute.isEqualToString("AXValue") {
                return match native(this).value() {
                    Some(value) => NSString::alloc(nil).init_str(&value),
                    None => nil,
                };
            }

            if attribute.isEqualToString("AXPosition") {
                let pos = native(this).position();
                let point = NSPoint {
                    x: pos.x as f64,
                    y: pos.y as f64,
                };
                return NSValue::valueWithPoint(nil, point);
            }

            if attribute.isEqualToString("AXSize") {
                let dim = native(this).dimensions();
                let size = NSSize {
                    width: dim.w as f64,
                    height: dim.h as f64,
                };
                return NSValue::valueWithSize(nil, size);
            }

            nil
        }
    }

    extern "C" fn accessibility_hit_test(_this: &Object, _: Sel, _point: NSPoint) -> id {
        nil
    }

    extern "C" fn accessibility_is_ignored(_this: &Object, _: Sel) -> BOOL {
        false as BOOL
    }

    extern "C" fn accessibility_focused_uielement(_this: &Object, _: Sel) -> id {
        nil
    }

    extern "C" fn accessibility_is_attribute_settable(
        _this: &Object,
        _: Sel,
        _attribute: id,
    ) -> BOOL {
        false as BOOL
    }

    extern "C" fn accessibility_action_names(_this: &Object, _: Sel) -> id {
        unsafe { NSArray::array(nil) }
    }

    extern "C" fn accessibility_action_description(_this: &Object, _: Sel, _action: id) -> id {
        unsafe { NSString::alloc(nil).init_str("") }
    }

    extern "C" fn accessibility_perform_action(_this: &Object, _: Sel, _action: id) {}

    static mut OBJECT_CLASS: *const Class = 0 as *const Class;
    static INIT: Once = Once::new();

    INIT.call_once(|| unsafe {
        // Create new NSObject
        let superclass = Class::get("NSObject").unwrap();
        let mut decl = ClassDecl::new("AccessibleObject", superclass).unwrap();

        decl.add_method(
            sel!(accessibilityAttributeNames),
            accessibility_attribute_names as extern "C" fn(&Object, Sel) -> id,
        );
        decl.add_method(
            sel!(accessibilityAttributeValue:),
            accessibility_attribute_value as extern "C" fn(&Object, Sel, id) -> id,
        );
        decl.add_method(
            sel!(accessibilityHitTest:),
            accessibility_hit_test as extern "C" fn(&Object, Sel, NSPoint) -> id,
        );
        decl.add_method(
            sel!(accessibilityIsIgnored),
            accessibility_is_ignored as extern "C" fn(&Object, Sel) -> BOOL,
        );
        decl.add_method(
            sel!(accessibilityFocusedUIElement),
            accessibility_focused_uielement as extern "C" fn(&Object, Sel) -> id,
        );
        decl.add_method(
            sel!(accessibilityIsAttributeSettable:),
            accessibility_is_attribute_settable as extern "C" fn(&Object, Sel, id) -> BOOL,
        );
        decl.add_method(
            sel!(accessibilityActionNames),
            accessibility_action_names as extern "C" fn(&Object, Sel) -> id,
        );
        decl.add_method(
            sel!(accessibilityActionDescription:),
            accessibility_action_description as extern "C" fn(&Object, Sel, id) -> id,
        );
        decl.add_method(
            sel!(accessibilityPerformAction:),
            accessibility_perform_action as extern "C" fn(&Object, Sel, id),
        );

        decl.add_ivar::<*mut libc::c_void>("native");

        OBJECT_CLASS = decl.register();
    });

    unsafe { OBJECT_CLASS }
}

pub fn init() {}

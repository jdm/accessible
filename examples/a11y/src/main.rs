extern crate accessible;
extern crate glutin;
extern crate gleam;

use accessible::{Accessible, Role, Parent, NativeRef, Dimensions, Position};
use std::default::Default;
use std::sync::{Arc, Mutex};

mod support;

fn resize_callback(width: u32, height: u32) {
    println!("Window resized to {}x{}", width, height);
}

struct RootAccessible(Mutex<(Vec<Arc<Accessible>>, Option<NativeRef>, i32, i32, u32, u32)>);
struct TextAccessible(Arc<RootAccessible>, String, i32, i32, u32, u32);

impl Accessible for RootAccessible {
    fn parent(&self) -> Parent {
        Parent::Native(self.0.lock().unwrap().1.unwrap())
    }

    fn children(&self) -> Vec<Arc<Accessible>> {
        println!("getting root children: {:?}", self.0.lock().unwrap().0.len());
        self.0.lock().unwrap().0.clone()
    }

    fn role(&self) -> Role {
        Role::ScrollArea
    }

    fn title(&self) -> Option<String> {
        Some("A fantastic (accessible!) window!".to_owned())
    }

    fn value(&self) -> Option<String> {
        None
    }

    fn position(&self) -> Position {
        let this = self.0.lock().unwrap();
        Position {
            x: this.2 as f32,
            y: this.3 as f32,
        }
    }

    fn dimensions(&self) -> Dimensions {
        let this = self.0.lock().unwrap();
        Dimensions {
            w: this.4 as f32,
            h: this.5 as f32,
        }
    }
}

impl Accessible for TextAccessible {
    fn parent(&self) -> Parent {
        Parent::Accessible(self.0.clone() as Arc<Accessible>)
    }

    fn children(&self) -> Vec<Arc<Accessible>> {
        vec![]
    }

    fn role(&self) -> Role {
        Role::StaticText
    }

    fn title(&self) -> Option<String> {
        None
    }

    fn value(&self) -> Option<String> {
        Some(self.1.clone())
    }

    fn position(&self) -> Position {
        Position {
            x: self.2 as f32,
            y: self.3 as f32,
        }
    }

    fn dimensions(&self) -> Dimensions {
        Dimensions {
            w: self.4 as f32,
            h: self.5 as f32,
        }
    }
}

fn main() {
    accessible::init();
    let cache = Arc::new(Mutex::new(Default::default()));
    let root = Arc::new(RootAccessible(Mutex::new((vec![], None, 0, 0, 0, 0))));

    let mut window = glutin::WindowBuilder::new()
        .with_accessibility(accessible::to_native_ref(root.clone(), cache))
        .build()
        .unwrap();
    unsafe {
        let r = root.clone();
        let mut root = root.0.lock().unwrap();
        root.1 = Some(window.platform_window() as NativeRef);
        let pos = window.get_position().unwrap();
        root.2 = pos.0;
        root.3 = pos.1;
        let size = window.get_inner_size().unwrap();
        root.4 = size.0;
        root.5 = size.1;

        let w = 100;
        let h = 50;
        let y = pos.1 + 50;
        let text1 = Arc::new(TextAccessible(r.clone(), "first text".to_owned(), pos.0, y , w, h));
        let text2 = Arc::new(TextAccessible(r.clone(), "second text".to_owned(), pos.0 + w as i32, y, w, h));
        let text3 = Arc::new(TextAccessible(r, "third text".to_owned(), pos.0 + 2*w as i32, y, w, h));
        root.0.extend(vec![text1 as Arc<Accessible>,
                           text2 as Arc<Accessible>,
                           text3 as Arc<Accessible>]);
    }
    window.set_title("A fantastic window!");
    window.set_window_resize_callback(Some(resize_callback as fn(u32, u32)));
    let _ = unsafe { window.make_current() };

    println!("Pixel format of the window: {:?}", window.get_pixel_format());

    support::load(&window);

    for event in window.wait_events() {
        support::draw_frame((0.0, 0.0, 0.0, 1.0));
        let _ = window.swap_buffers();

        //println!("{:?}", event);

        match event {
            glutin::Event::Closed => break,
            _ => ()
        }
    }
}

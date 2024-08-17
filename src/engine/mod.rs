pub mod events;

use crate::constants::*;
use crate::engine::events::*;
use std::{
    cell::RefCell,
    rc::Rc,
};
use wasm_bindgen::prelude::*;

pub struct Engine {}

fn create_closure(f: impl FnMut(f64) + 'static) -> Closure<dyn FnMut(f64)> {
    Closure::wrap(Box::new(f))
}

fn request_animation_frame(callback: &Closure<dyn FnMut(f64)>) -> Result<i32,JsValue> {
    web_sys::window().unwrap()
        .request_animation_frame(callback.as_ref().unchecked_ref())
}

fn now() -> f64 {
    web_sys::window().unwrap().performance().unwrap().now()
}

#[derive(Clone,Debug)]
pub struct Time {
    pub last_frame: f64,
    pub delta: f64,
}

pub trait Runnable {
    fn update(&mut self, time: &Time, evt: Option<Event>);
    fn draw(&self, canvas: &web_sys::HtmlCanvasElement);
}

impl Engine {
    pub async fn start(canvas: web_sys::HtmlCanvasElement,
                       mut runnable: impl Runnable + 'static) {
        let mut event_processor = EventProcessor::new(&canvas);

        let mut time = Time { last_frame: now(), delta: 0.0 };
        let mut accumulated_time = 0.0;

        let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(create_closure(move |perf: f64| {
            time.delta = (perf - time.last_frame) / 1000.0;
            accumulated_time += time.delta;
            while accumulated_time > FRAME_LENGTH {
                let evt = event_processor.process();
                runnable.update(&time, evt);
                accumulated_time -= FRAME_LENGTH;
            }
            time.last_frame = perf;
            runnable.draw(&canvas);
            request_animation_frame(f.borrow().as_ref().unwrap()).unwrap();
        }));

        request_animation_frame(g.borrow().as_ref().unwrap()).unwrap();
    }
}

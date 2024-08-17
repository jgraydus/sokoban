
use futures::channel::mpsc::{channel,Receiver};
use futures::stream::StreamExt;
use wasm_bindgen::prelude::*;

#[derive(Clone,Debug)]
pub enum Event {
    Click { x: i32, y: i32 }
}

pub struct EventProcessor {
    receiver: Receiver<Event>
}

impl EventProcessor {
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Self {
        let (s, r) = channel::<Event>(10);

        let mut s1 = s.clone();
        let click_handler: Closure<dyn FnMut(web_sys::PointerEvent)> =
            Closure::new(move |evt: web_sys::PointerEvent| {
                let e = Event::Click { x: evt.offset_x(), y: evt.offset_y() };
                s1.try_send(e).unwrap();
            });
        canvas.set_onclick(Some(click_handler.as_ref().unchecked_ref()));
        click_handler.forget();

        Self { receiver: r }
    }

    pub fn process(&mut self) -> Option<Event> {
        let mut evt = None;
        while let Ok(Some(e)) = self.receiver.try_next() {
            evt = Some(e);
        }
        evt
    }
}


mod constants;
mod game;
mod utils;
mod sprite;

use constants::*;
use futures::channel::mpsc::channel;
use futures::stream::StreamExt;
use game::*;
use wasm_bindgen::prelude::*;
use sprite::*;

#[wasm_bindgen]
pub fn run() {
    utils::set_panic_hook();

    wasm_bindgen_futures::spawn_local(async move {
        let canvas = utils::get_canvas();
        canvas.set_height(SIZE);
        canvas.set_width(SIZE);
    
        let (mut s, mut r) = channel::<(i32,i32)>(10);
    
        let click_handler: Closure<dyn FnMut(web_sys::PointerEvent)>
            = Closure::new(move |evt: web_sys::PointerEvent| {
                  s.try_send((evt.offset_x(), evt.offset_y())).unwrap();
              });
        canvas.set_onclick(Some(click_handler.as_ref().unchecked_ref()));
        click_handler.forget();
    
        let mut game = Game::example();
        let cxt = utils::get_context2d(&canvas);

        game.draw(&cxt);
        while let Some((x,y)) = r.next().await {
            game.handle_click(Location { x: x as f64, y: y as f64 });
            game.draw(&cxt);
        }
    });
}

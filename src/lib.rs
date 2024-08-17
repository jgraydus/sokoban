mod constants;
mod engine;
mod game;
mod utils;
mod sprite;

use crate::constants::*;
use crate::game::*;
use crate::engine::*;
use crate::sprite::*;
use futures::channel::mpsc::channel;
use futures::stream::StreamExt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
    utils::set_panic_hook();

    wasm_bindgen_futures::spawn_local(async move {
        let canvas = utils::get_canvas();
        canvas.set_height(SIZE);
        canvas.set_width(SIZE);
        Engine::start(canvas, Game::example()).await;
    });
}

mod blocks;
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
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
    utils::set_panic_hook();

    wasm_bindgen_futures::spawn_local(async move {
        let canvas = utils::get_canvas();
        canvas.set_height(SIZE);
        canvas.set_width(SIZE + 200);
 
        let cxt = canvas.get_context("2d")
            .expect("failed to get Context2d")
            .expect("Context2d missing")
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("failed to convert result into Context2d");
        cxt.set_image_smoothing_enabled(false);

        let game = Game::new().await;

        Engine::start(canvas, game).await;
    });
}


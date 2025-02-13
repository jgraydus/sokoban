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
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
pub fn run() {
    utils::set_panic_hook();

    wasm_bindgen_futures::spawn_local(async move {

        let canvas = utils::get_canvas();
        canvas.set_height(SIZE);
        canvas.set_width(SIZE);
 
        let cxt = canvas.get_context("2d")
            .expect("failed to get Context2d")
            .expect("Context2d missing")
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("failed to convert result into Context2d");
        cxt.set_image_smoothing_enabled(false);

        let wall = {
            let tmp = web_sys::window()
                .expect("window not found?!")
                .create_image_bitmap_with_image_data(&blocks::wall())
                .unwrap();
            JsFuture::from(tmp).await
                .expect("FOOOOOOO!")
                .dyn_into::<web_sys::ImageBitmap>()
                .expect("on no")
        };

        let floor = {
            let tmp = web_sys::window()
                .expect("window not found?!")
                .create_image_bitmap_with_image_data(&blocks::floor())
                .unwrap();
            JsFuture::from(tmp).await
                .expect("FOOOOOOO!")
                .dyn_into::<web_sys::ImageBitmap>()
                .expect("on no")
        };

        let moveable = {
            let tmp = web_sys::window()
                .expect("window not found?!")
                .create_image_bitmap_with_image_data(&blocks::moveable())
                .unwrap();
            JsFuture::from(tmp).await
                .expect("FOOOOOOO!")
                .dyn_into::<web_sys::ImageBitmap>()
                .expect("on no")
        };

        let mut sprites = BTreeMap::from([
            (SpriteType::Wall, wall),
            (SpriteType::Floor, floor),
            (SpriteType::Moveable, moveable),
        ]);

        Engine::start(canvas, Game::example(sprites)).await;
    });
}

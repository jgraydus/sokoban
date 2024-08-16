use wasm_bindgen::prelude::*;
use web_sys;

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn get_document() -> web_sys::Document {
    web_sys::window().expect("window not found?!")
        .document().expect("no document?!")
}

pub fn get_canvas() -> web_sys::HtmlCanvasElement {
    get_document()
        .get_element_by_id("sokoban-canvas")
        .expect("canvas is missing")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("conversion to canvas element failed")
}

pub fn get_context2d(canvas: &web_sys::HtmlCanvasElement) -> web_sys::CanvasRenderingContext2d {
    canvas.get_context("2d")
        .expect("failed to get Context2d")
        .expect("Context2d missing")
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .expect("conversion to Context2d failed")
}

pub fn create_canvas() -> web_sys::HtmlCanvasElement {
    get_document()
        .create_element("canvas")
        .expect("failed to create canvas")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("failed to convert created element into canvas")
}


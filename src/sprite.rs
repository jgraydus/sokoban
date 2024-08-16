use wasm_bindgen::prelude::*;
use web_sys;

#[derive(Clone,Copy,PartialEq)]
pub enum Direction { Up, Down, Left, Right }

impl Direction {
    pub fn as_diff(&self) -> (f64, f64) {
        match self {
            Direction::Up    => (0.0, -1.0),
            Direction::Down  => (0.0, 1.0),
            Direction::Left  => (-1.0, 0.0),
            Direction::Right => (1.0, 0.0),
        }
    }
}

#[derive(Clone,PartialEq)]
pub struct Location { pub x: f64, pub y: f64 }

impl Location {
    pub fn move_to(&self, dir: Direction) -> Self {
        let (dx, dy) = dir.as_diff();
        Self { x: self.x + dx, y: self.y + dy }
    }

    pub fn direction_to(&self, other: &Location) -> Option<Direction> {
        match (other.x - self.x, other.y - self.y) {
            (0.0, -1.0) => Some(Direction::Up),
            (0.0, 1.0)  => Some(Direction::Down),
            (-1.0, 0.0) => Some(Direction::Left),
            (1.0, 0.0)  => Some(Direction::Right),
            _ => None,
        }
    }
}

#[derive(Clone,PartialEq)]
pub struct Size { pub w: f64, pub h: f64 }

pub trait Sprite {
    fn draw(&self, cxt: &web_sys::CanvasRenderingContext2d, loc: Location, size: Size);
}

pub struct CanvasSprite {
    canvas: web_sys::HtmlCanvasElement,
}

impl CanvasSprite {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        Self { canvas }
    }
}

impl Sprite for CanvasSprite {
    fn draw(&self, cxt: &web_sys::CanvasRenderingContext2d, loc: Location, size: Size) {
        cxt.draw_image_with_html_canvas_element_and_dw_and_dh(
            &self.canvas, loc.x, loc.y, size.w, size.h
        );
    }
}

pub struct OffscreenCanvasSprite {
    canvas: web_sys::OffscreenCanvas,
}

impl OffscreenCanvasSprite {
    pub fn new(canvas: web_sys::OffscreenCanvas) -> Self {
        Self { canvas }
    }
}

impl Sprite for OffscreenCanvasSprite {
    fn draw(&self, cxt: &web_sys::CanvasRenderingContext2d, loc: Location, size: Size) {
        cxt.draw_image_with_offscreen_canvas_and_dw_and_dh(
            &self.canvas, loc.x, loc.y, size.w, size.h
        );
    }
}


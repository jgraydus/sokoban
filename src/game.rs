use crate::blocks;
use crate::constants::*;
use crate::engine::*;
use crate::engine::events::*;
use crate::sprite::*;
use crate::utils;
use std::{
    collections::BTreeMap,
    default::Default,
};
use wasm_bindgen::prelude::*;
use web_sys::ImageBitmap;

const CELLS: u32 = 8;
const CELL_SIZE: f64 = SIZE as f64 / CELLS as f64;

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
pub enum SpriteType {
    Player,
    Wall,
    Floor,
    Moveable,
}

pub struct Game {
    walls: Vec<Location>,
    blocks: Vec<Location>,
    goals: Vec<Location>,
    pub player: Location,
    background: Option<OffscreenCanvasSprite>,
    sprites: BTreeMap<SpriteType, ImageBitmap>, 
}

impl Game {
    pub fn from_string(sprites: BTreeMap<SpriteType, ImageBitmap>, s: String) -> Self {
        let mut g = Self {
            walls: Vec::new(),
            blocks: Vec::new(),
            goals: Vec::new(),
            player: Location { x: 0.0, y: 0.0 },
            background: None,
            sprites,
        };

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let loc = Location { x: x as f64, y: y as f64 };
                match c {
                    'O' => { g.walls.push(loc); }
                    '.' => { g.goals.push(loc); }
                    'x' => { g.blocks.push(loc); }
                    'X' => { g.blocks.push(loc.clone()); g.goals.push(loc); }
                    'p' => { g.player = loc; }
                    'P' => { g.goals.push(loc.clone()); g.player = loc; }
                    _ => {}
                }
            }
        }

        g.render_background();

        g
    }

    pub fn example(sprites: BTreeMap<SpriteType, ImageBitmap>) -> Game {
        Game::from_string(sprites,
r"  OOOOO
OOO   O
O.px  O
OOO x.O
O.OOx O
O O . OO
Ox Xxx.O
O   .  O
OOOOOOOO".into())
    }

    fn valid_moves(&self) -> Vec<Location> {
        let mut result = Vec::new();
        for dir in [Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
            if self.try_move(dir) {
                let loc = self.player.move_to(dir);
                result.push(loc);
            }
        }
        result
    }

    fn try_move(&self, dir: Direction) -> bool {
        let new_loc = self.player.move_to(dir);
        if self.walls.contains(&new_loc) {
            return false;
        }
        if self.blocks.contains(&new_loc) {
            let new_block_loc = new_loc.move_to(dir);
            if self.walls.contains(&new_block_loc) || self.blocks.contains(&new_block_loc) {
                return false;
            }
        }
        return true;
    }

    fn apply_move(&mut self, dir: Direction) {
        if self.try_move(dir) {
            let new_loc = self.player.move_to(dir);
            if let Some(i) = self.blocks.iter().position(|loc| *loc == new_loc) {
                let new_block_loc = new_loc.move_to(dir);
                self.blocks[i] = new_block_loc;
            }
            self.player = new_loc;
        }
    }

    pub fn handle_click(&mut self, Location { x, y }: Location) {
        let (x,y) = ((x / CELL_SIZE).floor(), (y / CELL_SIZE).floor());

        if let Some(dir) = self.player.direction_to(&Location { x, y }) {
            self.apply_move(dir);
        }
    }

    pub fn draw(&self, cxt: &web_sys::CanvasRenderingContext2d) {
        // draw background
        self.background
            .as_ref()
            .unwrap()
            .draw(&cxt,
                  Location { x: 0.0, y: 0.0 },
                  Size { w: SIZE as f64, h: SIZE as f64 });

        // draw blocks
        cxt.set_fill_style(&JsValue::from_str("#660000"));
        for item in &self.blocks {
            cxt.draw_image_with_image_bitmap_and_dw_and_dh(
                &self.sprites.get(&SpriteType::Moveable).unwrap(),
                item.x * CELL_SIZE,
                item.y * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE);
        }

        // draw goals
        cxt.set_fill_style(&JsValue::from_str("#664422"));
        for item in &self.goals {
            cxt.begin_path();
            cxt.ellipse(item.x * CELL_SIZE + CELL_SIZE / 2.0,
                        item.y * CELL_SIZE + CELL_SIZE / 2.0,
                        CELL_SIZE / 4.0,
                        CELL_SIZE / 4.0,
                        0.0,
                        0.0,
                        360.0);
            cxt.fill();
        }

        // draw player
        cxt.set_fill_style(&JsValue::from_str("#66FF88"));
        cxt.begin_path();
        cxt.ellipse(self.player.x * CELL_SIZE + CELL_SIZE / 2.0,
                    self.player.y * CELL_SIZE + CELL_SIZE / 2.0,
                    CELL_SIZE / 4.0,
                    CELL_SIZE / 4.0,
                    0.0,
                    0.0,
                    360.0);
        cxt.fill();

        // outline valid moves
        for item in self.valid_moves() {
            cxt.set_stroke_style(&JsValue::from_str("#669966"));
            cxt.set_line_width(2.0);
            cxt.stroke_rect(item.x * CELL_SIZE + 1.0,
                            item.y * CELL_SIZE + 1.0,
                            CELL_SIZE - 2.0,
                            CELL_SIZE - 2.0);
        }
    }

    fn render_background(&mut self) {
        let canvas = web_sys::OffscreenCanvas::new(SIZE, SIZE).expect("failed to create offscreen canvas");
        let cxt = canvas.get_context("2d")
            .expect("failed to get Context2d")
            .expect("Context2d missing")
            .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
            .expect("failed to convert result into Context2d");
        cxt.set_image_smoothing_enabled(false);

        cxt.set_fill_style(&JsValue::from_str("#444444"));
        cxt.fill_rect(0.0, 0.0, SIZE as f64, SIZE as f64);

        cxt.set_stroke_style(&JsValue::from_str("#333333"));
        for row in 0..CELLS {
            for col in 0..CELLS {
                let x = col as f64 * CELL_SIZE;
                let y = row as f64 * CELL_SIZE;
                cxt.draw_image_with_image_bitmap_and_dw_and_dh(
                    &self.sprites.get(&SpriteType::Floor).unwrap(), x, y, CELL_SIZE, CELL_SIZE);
            }
        }

        // draw walls
        for item in &self.walls {
            cxt.draw_image_with_image_bitmap_and_dw_and_dh(
                &self.sprites.get(&SpriteType::Wall).unwrap(),
                item.x * CELL_SIZE,
                item.y * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE);
        }

        self.background = Some(OffscreenCanvasSprite::new(canvas))
    }
}

impl Runnable for Game {
    fn update(&mut self, time: &Time, evt: Option<Event>) {
        if let Some(Event::Click { x, y }) = evt {
            self.handle_click(Location { x: x as f64, y: y as f64 });
        }
    }

    fn draw(&self, canvas: &web_sys::HtmlCanvasElement) {
        self.draw(&utils::get_context2d(canvas));
    }
}


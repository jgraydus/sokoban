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
use wasm_bindgen_futures::JsFuture;
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
    level: u32,
    is_won: bool,
}

const LEVEL_01: &str =
r"  OOOOO
OOO   O
O.px  O
OOO x.O
O.OOx O
O O . OO
Ox Xxx.O
O   .  O
OOOOOOOO";

const LEVEL_02: &str =
r"  OOOOOO
  O .. O
  O .  O
OOOx. OO
O   O OO
O pxxx O
O      O
OOOOOOOO";

impl Game {
    pub async fn new() -> Self {
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

        let mut g = Self {
            walls: Vec::new(),
            blocks: Vec::new(),
            goals: Vec::new(),
            player: Location { x: 0.0, y: 0.0 },
            level: 1,
            is_won: false,
            background: None,
            sprites,
        };

        g.load_level(1);

        g
    }

    fn load_level(&mut self, level: u32) {
        self.walls.clear();
        self.blocks.clear();
        self.goals.clear();
        self.background = None;
        self.level = level;
        self.is_won = false;

        let s = match level {
          1 => LEVEL_01,
          2 => LEVEL_02,
          _ => LEVEL_02,
        };

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let loc = Location { x: x as f64, y: y as f64 };
                match c {
                    'O' => { self.walls.push(loc); }
                    '.' => { self.goals.push(loc); }
                    'x' => { self.blocks.push(loc); }
                    'X' => { self.blocks.push(loc.clone()); self.goals.push(loc); }
                    'p' => { self.player = loc; }
                    'P' => { self.goals.push(loc.clone()); self.player = loc; }
                    _ => {}
                }
            }
        }

        self.render_background();
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
        self.check_for_win();
    }

    fn check_for_win(&mut self) {
        for goal in &self.goals {
            if !self.blocks.contains(&goal) { return; }
        }
        self.is_won = true;
    }

    pub fn handle_click(&mut self, Location { x, y }: Location) {
        // check if player clicked the button
        let s = SIZE as f64;
        if x > s + 50.0 && x < s + 150.0 && y > s - 100.0 && y < s - 50.0 {
            if self.is_won {
                self.load_level(self.level + 1);
            } else {
                self.load_level(self.level);
            }
        }

        let (x,y) = ((x / CELL_SIZE).floor(), (y / CELL_SIZE).floor());

        if let Some(dir) = self.player.direction_to(&Location { x, y }) {
            self.apply_move(dir);
        }

    }

    pub fn draw(&self, cxt: &web_sys::CanvasRenderingContext2d) {
        // clear
        cxt.set_fill_style_str(&"#000000");
        cxt.fill_rect(0.0, 0.0, SIZE as f64 + 200.0, SIZE as f64);

        // draw background
        self.background
            .as_ref()
            .unwrap()
            .draw(&cxt,
                  Location { x: 0.0, y: 0.0 },
                  Size { w: SIZE as f64, h: SIZE as f64 });

        // draw blocks
        cxt.set_fill_style_str(&"#660000");
        for item in &self.blocks {
            cxt.draw_image_with_image_bitmap_and_dw_and_dh(
                &self.sprites.get(&SpriteType::Moveable).unwrap(),
                item.x * CELL_SIZE,
                item.y * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE);
        }

        // draw goals
        cxt.set_fill_style_str(&"#664422");
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
        cxt.set_fill_style_str(&"#66FF88");
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
            cxt.set_stroke_style_str(&"#669966");
            cxt.set_line_width(2.0);
            cxt.stroke_rect(item.x * CELL_SIZE + 1.0,
                            item.y * CELL_SIZE + 1.0,
                            CELL_SIZE - 2.0,
                            CELL_SIZE - 2.0);
        }

        // draw ui stuff
        if self.is_won {
            cxt.set_stroke_style_str(&"#00FF00");
            cxt.stroke_rect(SIZE as f64 + 50.0, SIZE as f64 - 100.0, 100.0, 50.0);
            cxt.set_fill_style_str(&"#00FF00");
            cxt.set_font(&"12pt sans-serif");
            cxt.fill_text(&"NEXT", SIZE as f64 + 78.0, SIZE as f64 - 78.0);
            cxt.fill_text(&"LEVEL", SIZE as f64 + 75.0, SIZE as f64 - 60.0);
        } else {
            cxt.set_stroke_style_str(&"#FF0000");
            cxt.stroke_rect(SIZE as f64 + 50.0, SIZE as f64 - 100.0, 100.0, 50.0);
            cxt.set_fill_style_str(&"#FF0000");
            cxt.set_font(&"20pt sans-serif");
            cxt.fill_text(&"RESET", SIZE as f64 + 57.0, SIZE as f64 - 65.0);
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

        cxt.set_fill_style_str(&"#444444");
        cxt.fill_rect(0.0, 0.0, SIZE as f64, SIZE as f64);

        cxt.set_stroke_style_str(&"#333333");
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


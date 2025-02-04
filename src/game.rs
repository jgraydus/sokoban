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

const CELLS: u32 = 8;
const CELL_SIZE: f64 = SIZE as f64 / CELLS as f64;

struct GoalSprite {}

impl Sprite for GoalSprite {
    fn draw(&self, cxt: &web_sys::CanvasRenderingContext2d, loc: Location, size: Size) {
        cxt.set_fill_style(&JsValue::from_str("#664422"));
        cxt.begin_path();
        cxt.ellipse(loc.x * CELL_SIZE + CELL_SIZE / 2.0,
                    loc.y * CELL_SIZE + CELL_SIZE / 2.0,
                    CELL_SIZE / 4.0,
                    CELL_SIZE / 4.0,
                    0.0,
                    0.0,
                    360.0);
        cxt.fill();
    }
}

struct BlockSprite {}

impl Sprite for BlockSprite {
    fn draw(&self, cxt: &web_sys::CanvasRenderingContext2d, loc: Location, size: Size) {
        cxt.set_fill_style(&JsValue::from_str("#224466"));
        cxt.fill_rect(loc.x * CELL_SIZE + 1.0,
                      loc.y * CELL_SIZE + 1.0,
                      CELL_SIZE - 2.0,
                      CELL_SIZE - 2.0);
    }
}

struct PlayerSprite {}

impl Sprite for PlayerSprite {
    fn draw(&self, cxt: &web_sys::CanvasRenderingContext2d, loc: Location, size: Size) {
        cxt.set_fill_style(&JsValue::from_str("#66FF88"));
        cxt.begin_path();
        cxt.ellipse(loc.x * CELL_SIZE + CELL_SIZE / 2.0,
                    loc.y * CELL_SIZE + CELL_SIZE / 2.0,
                    CELL_SIZE / 4.0,
                    CELL_SIZE / 4.0,
                    0.0,
                    0.0,
                    360.0);
        cxt.fill();
    }
}


type EntityId = i32;

#[derive(Clone,Debug)]
enum EntityType { Block, Goal, Player }

#[derive(Default)]
pub struct Game_ {
    walls: Vec<Location>,
    entities: Vec<EntityId>,
    locations: BTreeMap<EntityId, Location>,
    entity_types: BTreeMap<EntityId, EntityType>,
    sprites: BTreeMap<EntityId, Box<dyn Sprite>>,
    background: Option<OffscreenCanvasSprite>,
}

impl Game_ {
    pub fn from_string(s: String) -> Self {
        let mut g = Game_::default();

        let mut next_id: i32 = 0;

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                match c {
                    'O' => {
                        g.walls.push(Location { x: x as f64, y: y as f64 });
                    }
                    '.' => {
                        let id = next_id;
                        next_id = next_id + 1;
                        g.entities.push(id);
                        g.entity_types.insert(id, EntityType::Goal);
                        g.sprites.insert(id, Box::new(GoalSprite {}));
                        g.locations.insert(id, Location { x: x as f64, y: y as f64 });
                    }
                    'x' => {
                        let id = next_id;
                        next_id = next_id + 1;
                        g.entities.push(id);
                        g.entity_types.insert(id, EntityType::Block);
                        g.sprites.insert(id, Box::new(BlockSprite {}));
                        g.locations.insert(id, Location { x: x as f64, y: y as f64 });
                    }
                    'X' => {
                        {
                            let id = next_id;
                            next_id = next_id + 1;
                            g.entities.push(id);
                            g.entity_types.insert(id, EntityType::Block);
                            g.sprites.insert(id, Box::new(BlockSprite {}));
                            g.locations.insert(id, Location { x: x as f64, y: y as f64 });
                        }
                        {
                            let id = next_id;
                            next_id = next_id + 1;
                            g.entities.push(id);
                            g.entity_types.insert(id, EntityType::Goal);
                            g.sprites.insert(id, Box::new(GoalSprite {}));
                            g.locations.insert(id, Location { x: x as f64, y: y as f64 });
                        }
                    }
                    'p' => {
                        let id = next_id;
                        next_id = next_id + 1;
                        g.entities.push(id);
                        g.entity_types.insert(id, EntityType::Player);
                        g.sprites.insert(id, Box::new(PlayerSprite {}));
                        g.locations.insert(id, Location { x: x as f64, y: y as f64 });
                    }
                    'P' => {
                        {
                            let id = next_id;
                            next_id = next_id + 1;
                            g.entities.push(id);
                            g.entity_types.insert(id, EntityType::Player);
                            g.sprites.insert(id, Box::new(PlayerSprite {}));
                            g.locations.insert(id, Location { x: x as f64, y: y as f64 });
                        }
                        {
                            let id = next_id;
                            next_id = next_id + 1;
                            g.entities.push(id);
                            g.entity_types.insert(id, EntityType::Goal);
                            g.sprites.insert(id, Box::new(GoalSprite {}));
                            g.locations.insert(id, Location { x: x as f64, y: y as f64 });
                        }
                    }
                    _ => {}
                }
            }
        }

        g.render_background();

        g
    }

    fn render_background(&mut self) {
        let canvas = web_sys::OffscreenCanvas::new(SIZE, SIZE).expect("failed to create offscreen canvas");
        let cxt = canvas.get_context("2d")
            .expect("failed to get Context2d")
            .expect("Context2d missing")
            .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
            .expect("failed to convert result into Context2d");

        cxt.set_fill_style(&JsValue::from_str("#444444"));
        cxt.fill_rect(0.0, 0.0, SIZE as f64, SIZE as f64);

        cxt.set_stroke_style(&JsValue::from_str("#333333"));
        for row in 0..CELLS {
            for col in 0..CELLS {
                let x = col as f64 * CELL_SIZE;
                let y = row as f64 * CELL_SIZE;
                cxt.stroke_rect(x, y, CELL_SIZE, CELL_SIZE);
            }
        }

        // draw walls
        cxt.set_fill_style(&JsValue::from_str("#111111"));
        for item in &self.walls {
            cxt.fill_rect(item.x * CELL_SIZE + 1.0,
                          item.y * CELL_SIZE + 1.0,
                          CELL_SIZE - 2.0,
                          CELL_SIZE - 2.0);
        }

        self.background = Some(OffscreenCanvasSprite::new(canvas))
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
        for (id, spr) in self.sprites.iter() {
            if let Some(EntityType::Block) = self.entity_types.get(&id) {
                if let Some(loc) = self.locations.get(&id) {
                    spr.draw(cxt, loc.clone(), Size { w: CELL_SIZE, h: CELL_SIZE });
                }
            }
        }

        // draw goals
        for (id, spr) in self.sprites.iter() {
            if let Some(EntityType::Goal) = self.entity_types.get(&id) {
                if let Some(loc) = self.locations.get(&id) {
                    spr.draw(cxt, loc.clone(), Size { w: CELL_SIZE, h: CELL_SIZE });
                }
            }
        }

        // draw player
        for (id, spr) in self.sprites.iter() {
            if let Some(EntityType::Player) = self.entity_types.get(&id) {
                if let Some(loc) = self.locations.get(&id) {
                    spr.draw(cxt, loc.clone(), Size { w: CELL_SIZE, h: CELL_SIZE });
                }
            }
        }
    }
}

pub struct Game {
    walls: Vec<Location>,
    blocks: Vec<Location>,
    goals: Vec<Location>,
    pub player: Location,
    background: Option<OffscreenCanvasSprite>,
}

impl Game {
    pub fn from_string(s: String) -> Self {
        let mut g = Self {
            walls: Vec::new(),
            blocks: Vec::new(),
            goals: Vec::new(),
            player: Location { x: 0.0, y: 0.0 },
            background: None,
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

    pub fn example() -> Game {
        Game::from_string(
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
        cxt.set_fill_style(&JsValue::from_str("#224466"));
        for item in &self.blocks {
            cxt.fill_rect(item.x * CELL_SIZE + 1.0,
                          item.y * CELL_SIZE + 1.0,
                          CELL_SIZE - 2.0,
                          CELL_SIZE - 2.0);
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

        cxt.set_fill_style(&JsValue::from_str("#444444"));
        cxt.fill_rect(0.0, 0.0, SIZE as f64, SIZE as f64);

        cxt.set_stroke_style(&JsValue::from_str("#333333"));
        for row in 0..CELLS {
            for col in 0..CELLS {
                let x = col as f64 * CELL_SIZE;
                let y = row as f64 * CELL_SIZE;
                cxt.stroke_rect(x, y, CELL_SIZE, CELL_SIZE);
            }
        }

        // draw walls
        cxt.set_fill_style(&JsValue::from_str("#111111"));
        for item in &self.walls {
            cxt.fill_rect(item.x * CELL_SIZE + 1.0,
                          item.y * CELL_SIZE + 1.0,
                          CELL_SIZE - 2.0,
                          CELL_SIZE - 2.0);
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


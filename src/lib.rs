mod utils;
mod sprite;

use futures::channel::mpsc::channel;
use futures::stream::StreamExt;
use wasm_bindgen::prelude::*;
use sprite::*;

const SIZE: u32 = 800;

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
    
        let cxt = utils::get_context2d(&canvas);
    
        let background = render_background();
    
        let mut game = Game::from_string(
r"  OOOOO 
OOO   O 
O.px  O 
OOO x.O 
O.OOx O 
O O . OO
Ox Xxx.O
O   .  O
OOOOOOOO".into());
    
        background.draw(&cxt, Location { x: 0.0, y: 0.0 }, Size { w: SIZE as f64, h: SIZE as f64 });
        game.draw(&cxt);
    
        while let Some((x,y)) = r.next().await {
            let cell_size = SIZE as f64 / 8.0;
            let (x,y) = ((x as f64 / cell_size).floor(), (y as f64 / cell_size).floor());

            if let Some(dir) = game.player.direction_to(&Location { x, y }) {
                game.apply_move(dir);
            }
            background.draw(&cxt, Location { x: 0.0, y: 0.0 }, Size { w: SIZE as f64, h: SIZE as f64 });
            game.draw(&cxt);
        }
    });
}

fn render_background() -> impl Sprite {
    let canvas = web_sys::OffscreenCanvas::new(SIZE, SIZE).expect("failed to create offscreen canvas");
    let cxt = canvas.get_context("2d")
        .expect("failed to get Context2d")
        .expect("Context2d missing")
        .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
        .expect("failed to convert result into Context2d");

    cxt.set_fill_style(&JsValue::from_str("#444444"));
    cxt.fill_rect(0.0, 0.0, SIZE as f64, SIZE as f64);

    let cells = 8;
    let (w, h) = (SIZE as f64 / cells as f64, SIZE as f64 / cells as f64);

    cxt.set_stroke_style(&JsValue::from_str("#333333"));
    for row in 0..cells {
        for col in 0..cells {
            let x = col as f64 * w;
            let y = row as f64 * h;
            cxt.stroke_rect(x, y, w, h);
        }
    }

    OffscreenCanvasSprite::new(canvas)
}

struct Game {
    walls: Vec<Location>,
    blocks: Vec<Location>,
    goals: Vec<Location>,
    player: Location,
}

impl Game {
    fn from_string(s: String) -> Self {
        let mut g = Self {
            walls: Vec::new(),
            blocks: Vec::new(),
            goals: Vec::new(),
            player: Location { x: 0.0, y: 0.0 },
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

        g
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

    fn draw(&self, cxt: &web_sys::CanvasRenderingContext2d) {
        let cell_size = SIZE as f64 / 8.0;

        // draw walls
        cxt.set_fill_style(&JsValue::from_str("#111111"));
        for item in &self.walls {
            cxt.fill_rect(item.x * cell_size + 1.0,
                          item.y * cell_size + 1.0,
                          cell_size - 2.0,
                          cell_size - 2.0);
        }

        // draw blocks
        cxt.set_fill_style(&JsValue::from_str("#224466"));
        for item in &self.blocks {
            cxt.fill_rect(item.x * cell_size + 1.0,
                          item.y * cell_size + 1.0,
                          cell_size - 2.0,
                          cell_size - 2.0);
        }

        // draw goals
        cxt.set_fill_style(&JsValue::from_str("#664422"));
        for item in &self.goals {
            cxt.begin_path();
            cxt.ellipse(item.x * cell_size + cell_size / 2.0,
                        item.y * cell_size + cell_size / 2.0,
                        cell_size / 4.0,
                        cell_size / 4.0,
                        0.0,
                        0.0,
                        360.0);
            cxt.fill();
        }

        // draw player
        cxt.set_fill_style(&JsValue::from_str("#66FF88"));
        cxt.begin_path();
        cxt.ellipse(self.player.x * cell_size + cell_size / 2.0,
                    self.player.y * cell_size + cell_size / 2.0,
                    cell_size / 4.0,
                    cell_size / 4.0,
                    0.0,
                    0.0,
                    360.0);
        cxt.fill();

        // outline valid moves
        for item in self.valid_moves() {
            cxt.set_stroke_style(&JsValue::from_str("#669966"));
            cxt.set_line_width(2.0);
            cxt.stroke_rect(item.x * cell_size + 1.0,
                            item.y * cell_size + 1.0,
                            cell_size - 2.0,
                            cell_size - 2.0);
        }
    }
}


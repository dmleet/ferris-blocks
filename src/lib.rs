extern crate js_sys;
extern crate web_sys;

mod block;
mod board;
mod coord;
mod utils;

use block::*;
use board::*;
use coord::Coord;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Game {
    board: Board,
    context: CanvasRenderingContext2d,
    tick_delay: f64,
    cooldown: f64, // Some moves provide a cooldown, during which the current block will not be copied to the board cells
}

impl Game {
    pub fn make(
        canvas: &HtmlCanvasElement,
        rows: usize,
        cols: usize,
        cell_size_px: usize,
    ) -> Rc<RefCell<Game>> {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        canvas.set_width((cols * cell_size_px) as u32);
        canvas.set_height((rows * cell_size_px) as u32);

        let rc_game = Rc::new(RefCell::new(Game {
            board: Board::new(rows, cols, cell_size_px),
            context,
            tick_delay: 400.0,
            cooldown: 0.0,
        }));

        run(rc_game.clone()).expect("Somebody set us up the bomb!");
        rc_game.borrow_mut().draw();
        rc_game
    }
}

impl Game {
    fn draw(&mut self) {
        let border = &JsValue::from_str("black");
        let fill = &JsValue::from_str("grey");
        let empty = &JsValue::from_str("cornsilk");

        // Board
        for row in 0..self.board.rows {
            for col in 0..self.board.cols {
                self.context
                    .set_fill_style(match self.board.cells[row][col] {
                        Cell::Empty => empty,
                        Cell::Filled => fill,
                    });
                self.context.fill_rect(
                    (col * self.board.cell_size_px) as f64,
                    (row * self.board.cell_size_px) as f64,
                    self.board.cell_size_px as f64,
                    self.board.cell_size_px as f64,
                );

                if self.board.cells[row][col] == Cell::Filled {
                    self.context.set_stroke_style(border);
                    self.context.stroke_rect(
                        (col * self.board.cell_size_px) as f64,
                        (row * self.board.cell_size_px) as f64,
                        self.board.cell_size_px as f64,
                        self.board.cell_size_px as f64,
                    );
                }
            }
        }

        // Active Block
        self.board
            .block
            .draw(&self.context, self.board.pos, self.board.cell_size_px);

        // Border
        self.context.set_stroke_style(border);
        self.context.stroke_rect(
            0.0,
            0.0,
            (self.board.cols * self.board.cell_size_px) as f64,
            (self.board.rows * self.board.cell_size_px) as f64,
        );

        // Temp stats
        let score_str = format!(
            "Score: {:?} | Lines: {:?} | Lv: {:?} | x{:?} | Next: {:?}",
            self.board.score, self.board.lines, self.board.level, self.board.combo, self.board.next_block.style
        );
        self.context.set_font("12px sans-serif");
        self.context.fill_text(&score_str, 10.0, 10.0).unwrap();
    }
}

impl Game {
    pub fn get_tick_time(&self) -> f64 {
        650.0 - (self.board.level as f64) * 50.0
    }

    pub fn reset_tick_time(&mut self) {
        self.tick_delay += self.get_tick_time();
    }

    pub fn set_cooldown(&mut self, cd_time: f64) {
        if cd_time > self.cooldown {
            self.cooldown = cd_time;
        }
    }
}

#[wasm_bindgen]
impl Game {
    pub fn update(&mut self, delta_t: f64) {
        self.tick_delay -= delta_t;
        if self.cooldown > 0.0 {
            self.cooldown -= delta_t;
        }
        
        if self.tick_delay < 0.0 {
            let new_pos = Coord::new(self.board.pos.x, self.board.pos.y + 1);
            if self
                .board
                .check_collision(new_pos.clone(), self.board.block.coords)
            {
                if self.cooldown > 0.0 {
                    self.tick_delay = self.cooldown;
                } else {
                    self.board.update();
                    self.board.pos = Coord::new((self.board.cols / 2) as i32, 0);
                    self.board.block = self.board.next_block.clone();
                    self.board.next_block = Block::next();
                }
            } else {
                self.board.pos = new_pos;
            }
            self.draw();
            self.reset_tick_time();
        }
    }
}

impl Game {
    pub fn on_key(&mut self, key: u32) {
        const KEY_UP: u32 = 38;
        const KEY_DOWN: u32 = 40;
        const KEY_LEFT: u32 = 37;
        const KEY_RIGHT: u32 = 39;
        const KEY_SPACE: u32 = 32;

        match key {
            KEY_UP => self.drop(),
            KEY_DOWN => {
                let _ = self.move_down();
            }
            KEY_LEFT => self.move_left(),
            KEY_RIGHT => self.move_right(),
            KEY_SPACE => self.rotate(),
            _ => ( /* do nothing for every other key */),
        };
    }

    pub fn move_left(&mut self) {
        self.set_cooldown(250.0);
        let new_pos = Coord::new(self.board.pos.x - 1, self.board.pos.y);
        if !self
            .board
            .check_collision(new_pos.clone(), self.board.block.coords)
        {
            self.board.pos = new_pos;
            self.draw();
        }
    }

    pub fn move_right(&mut self) {
        self.set_cooldown(250.0);
        let new_pos = Coord::new(self.board.pos.x + 1, self.board.pos.y);
        if !self
            .board
            .check_collision(new_pos.clone(), self.board.block.coords)
        {
            self.board.pos = new_pos;
            self.draw();
        }
    }

    pub fn move_down(&mut self) -> bool {
        let new_pos = Coord::new(self.board.pos.x, self.board.pos.y + 1);
        if !self
            .board
            .check_collision(new_pos.clone(), self.board.block.coords)
        {
            self.board.pos = new_pos;
            self.draw();
            return true;
        }
        false
    }

    pub fn rotate(&mut self) {
        self.set_cooldown(400.0);
        if self.board.block.style == BlockStyle::O {
            return;
        }
        let mut new_coords = self.board.block.coords;
        for coord in new_coords.iter_mut() {
            let new_y = -coord.x;
            coord.x = coord.y;
            coord.y = new_y;
        }
        if !self.board.check_collision(self.board.pos, new_coords) {
            self.board.block.coords = new_coords;
            self.draw();
        } else {
            // Find the closest, adjacent place to put the rotated block (if possible)
            for j in (-1..0).rev() {
                for i in -1..2 {
                    let new_pos = Coord::new(self.board.pos.x + i as i32, self.board.pos.y + j);
                    if !self.board.check_collision(new_pos, new_coords) {
                        self.board.pos = new_pos;
                        self.rotate();
                        return;
                    }
                }
            }
        }
    }

    pub fn drop(&mut self) {
        while self.move_down() {}
    }
}

#[wasm_bindgen]
pub fn make() -> HtmlCanvasElement {
    utils::set_panic_hook();

    let canvas = window()
        .document()
        .unwrap()
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    Game::make(&canvas, 20, 10, 25);
    canvas
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

use js_sys::Date;
// https://rustwasm.github.io/docs/wasm-bindgen/examples/request-animation-frame.html
// I hate this. Send help.
pub fn run(rc_game: Rc<RefCell<Game>>) -> Result<(), JsValue> {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let window = window();
    let rc_game_clone = rc_game.clone();

    let mut last_frame = Date::now();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        rc_game.borrow_mut().update(Date::now() - last_frame);
        last_frame = Date::now();
        //rc_game.borrow_mut().draw();
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());

    let onkeydown_handler = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let mut game = rc_game_clone.borrow_mut();
        game.on_key(event.key_code());
    }) as Box<dyn FnMut(KeyboardEvent)>);
    window.set_onkeydown(Some(onkeydown_handler.as_ref().unchecked_ref()));
    onkeydown_handler.forget();

    Ok(())
}

extern crate js_sys;

mod utils;
mod coord;
mod block;
mod board;

use coord::Coord;
use block::*;
use board::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

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
}

impl Game {
    pub fn make(canvas: &HtmlCanvasElement, rows: usize, cols: usize, cell_size_px: usize) -> Game {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        // TODO - should I do something about 'usize as u32'?
        canvas.set_width((cols * cell_size_px) as u32);
        canvas.set_height((rows * cell_size_px) as u32);

        let game = Game {
            board: Board::new(rows, cols, cell_size_px),
            context,
        };

        game.clone().draw();
        game
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
        self.board.block.draw(&self.context, self.board.pos, self.board.cell_size_px);

        // Border
        self.context.set_stroke_style(border);
        self.context.stroke_rect(
            0.0,
            0.0,
            (self.board.cols * self.board.cell_size_px) as f64,
            (self.board.rows * self.board.cell_size_px) as f64,
        );
    }
}

#[wasm_bindgen]
impl Game {
    pub fn tick(&mut self) {

        let new_pos = Coord::new(self.board.pos.x, self.board.pos.y + 1);
        if self.board.check_collision(new_pos.clone(), self.board.block.coords) {
            self.board.update();
            self.board.pos = Coord::new((self.board.cols / 2) as i32, 0);
            self.board.block = Block::next();
        } else {
            self.board.pos = new_pos;
        }
        self.draw();
    }
}

#[wasm_bindgen]
impl Game {
    pub fn move_left(&mut self) {
        let new_pos = Coord::new(self.board.pos.x - 1, self.board.pos.y);
        if !self.board.check_collision(new_pos.clone(), self.board.block.coords)
        {
            self.board.pos = new_pos;
            self.draw();
        }
    }

    pub fn move_right(&mut self) {
        let new_pos = Coord::new(self.board.pos.x + 1, self.board.pos.y);
        if !self.board.check_collision(new_pos.clone(), self.board.block.coords)
        {
            self.board.pos = new_pos;
            self.draw();
        }
    }

    pub fn move_down(&mut self) -> bool {
        let new_pos = Coord::new(self.board.pos.x, self.board.pos.y + 1);
        if !self.board.check_collision(new_pos.clone(), self.board.block.coords)
        {
            self.board.pos = new_pos;
            self.draw();
            return true;
        }
        false
    }

    pub fn rotate(&mut self) {
        let mut new_coords = self.board.block.coords;
        for coord in new_coords.iter_mut() {
            let new_y = -coord.x;
            coord.x = coord.y;
            coord.y = new_y;
        }
        if !self.board.check_collision(self.board.pos, new_coords) {
            self.board.block.coords = new_coords;
            self.draw();
        }
    }

    pub fn drop(&mut self) {
        while self.move_down() {}
    }
}

#[wasm_bindgen]
pub fn make() -> Game {
    utils::set_panic_hook();
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let canvas = document
        .get_element_by_id("ferris-blocks-canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    Game::make(&canvas, 20, 10, 20)
}

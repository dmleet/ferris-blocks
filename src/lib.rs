extern crate js_sys;

mod utils;

use core::panic;

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

//#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty = 0,
    Filled = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum BlockStyle {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}
use self::BlockStyle::*;

#[derive(Debug, Clone, Copy)]
pub struct Coord {
    x: i32,
    y: i32,
}

impl std::ops::Add<Coord> for Coord {
    type Output = Coord;

    fn add(self, _rhs: Coord) -> Coord {
        Coord::new(self.x + _rhs.x, self.y + _rhs.y)
    }
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Coord {
        Coord { x, y }
    }
}

type BlockCoords = [Coord; 4];
#[derive(Debug, Clone)]
pub struct Block {
    style: BlockStyle,
    color: JsValue,
    coords: BlockCoords,
}

impl Block {
    // unsafe for now, caused by the shit js_sys methods
    // next block style will come from the server
    pub fn next() -> Block {
        let rnd = js_sys::Math::floor(js_sys::Math::random() * 7.0) as usize;
        let style = [I, J, L, O, S, T, Z][rnd];
        Block {
            style,
            color: JsValue::from_str("coral"),
            coords: match style {
                BlockStyle::I => [
                    Coord::new(0, -2),
                    Coord::new(0, -1),
                    Coord::new(0, 0),
                    Coord::new(0, 1),
                ],
                BlockStyle::J => [
                    Coord::new(0, -2),
                    Coord::new(0, -1),
                    Coord::new(0, 0),
                    Coord::new(-1, 0),
                ],
                BlockStyle::L => [
                    Coord::new(0, -2),
                    Coord::new(0, -1),
                    Coord::new(0, 0),
                    Coord::new(1, 0),
                ],
                BlockStyle::O => [
                    Coord::new(0, -1),
                    Coord::new(1, -1),
                    Coord::new(1, 0),
                    Coord::new(0, 0),
                ],
                BlockStyle::S => [
                    Coord::new(-1, 0),
                    Coord::new(0, 0),
                    Coord::new(0, -1),
                    Coord::new(1, -1),
                ],
                BlockStyle::T => [
                    Coord::new(-1, 0),
                    Coord::new(0, 0),
                    Coord::new(1, 0),
                    Coord::new(0, -1),
                ],
                BlockStyle::Z => [
                    Coord::new(-1, -1),
                    Coord::new(0, -1),
                    Coord::new(0, 0),
                    Coord::new(1, 0),
                ],
            },
        }
    }
}

impl Block {
    pub fn draw(&mut self, context: &CanvasRenderingContext2d, pos: Coord, cell_size_px: usize) {
        for coord in self.coords.iter() {
            let abs_coords = pos + coord.clone();
            context.set_fill_style(&self.color);
            context.fill_rect(
                (abs_coords.x * cell_size_px as i32) as f64,
                (abs_coords.y * cell_size_px as i32) as f64,
                cell_size_px as f64,
                cell_size_px as f64,
            );
            context.set_stroke_style(&JsValue::from_str("black"));
            context.stroke_rect(
                (abs_coords.x * cell_size_px as i32) as f64,
                (abs_coords.y * cell_size_px as i32) as f64,
                cell_size_px as f64,
                cell_size_px as f64,
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    rows: usize,
    cols: usize,
    cell_size_px: usize,
    cells: Vec<Vec<Cell>>,
    block: Block,
    pos: Coord,
    score: u32,
    level: u32,
}

impl Board {
    fn new(rows: usize, cols: usize, cell_size_px: usize) -> Board {
        Board {
            rows,
            cols,
            cell_size_px,
            cells: vec![vec![Cell::Empty; cols]; rows],
            block: Block::next(),
            pos: Coord::new((cols / 2) as i32, 0),
            score: 0,
            level: 0,
        }
    }
}

impl Board {
    fn check_collision(&mut self, new_pos: Coord, new_coords: BlockCoords) -> bool {
        new_coords.iter().any(|coord| {
            let abs_x = &coord.x + new_pos.x;
            let abs_y = &coord.y + new_pos.y;
            abs_x < 0
                || abs_x >= self.cols as i32
                || abs_y >= self.rows as i32
                || (abs_x >= 0
                    && abs_y >= 0
                    && self.cells[abs_y as usize][abs_x as usize] == Cell::Filled)
        })
    }
}

impl Board {
    fn fill_block(&mut self) {
        for coord in self.block.coords.iter() {
            let x = (coord.x + self.pos.x) as usize;
            let y = (coord.y + self.pos.y) as usize;
            self.cells[y][x] = Cell::Filled;
        }

        // Pop empty rows
        self.cells.retain(|row| row.iter().any(|&cell| cell == Cell::Empty));
        if self.cells.len() < self.rows {
            // Create the new rows
            let new_row_count = self.rows - self.cells.len();
            let mut new_cells = vec![vec![Cell::Empty; self.cols]; new_row_count ];
            new_cells.append(&mut self.cells);
            self.cells = new_cells;

            // Adjust score and level
            self.score += match new_row_count {
                1 => 100,
                2 => 250,
                3 => 500,
                4 => 1000,
                _ => panic!("How can you have any pudding if you don't eat yer meat!?")
            };
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Game {
    board: Board,
    context: CanvasRenderingContext2d,
}

impl Game {
    fn make(canvas: &HtmlCanvasElement, rows: usize, cols: usize, cell_size_px: usize) -> Game {
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
    }
}

#[wasm_bindgen]
impl Game {
    pub fn tick(&mut self) {
        let new_pos = Coord::new(self.board.pos.x, self.board.pos.y + 1);
        if self.board.check_collision(new_pos.clone(), self.board.block.coords) {
            self.board.fill_block();
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

    pub fn move_down(&mut self) {
        let new_pos = Coord::new(self.board.pos.x, self.board.pos.y + 1);
        if !self.board.check_collision(new_pos.clone(), self.board.block.coords)
        {
            self.board.pos = new_pos;
            self.draw();
        }
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

extern crate js_sys;

mod utils;

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

#[derive(Debug, Clone)]
pub struct Coord {
    x: usize,
    y: usize,
}

impl Coord {
    pub fn new(x: usize, y: usize) -> Coord {
        Coord { x, y }
    }
}

type BlockCoords = [Coord; 4];
#[derive(Debug, Clone)]
pub struct Block {
    style: BlockStyle,
    coords: BlockCoords,
    color: JsValue,
}

impl Block {
    // unsafe for now, caused by the shit js_sys methods
    // next block style will come from the server
    pub fn next() -> Block {
        let rnd =js_sys::Math::floor(
            js_sys::Math::random() * 7.0) as usize;
        let style = [ I, J, L, O, S, T, Z ][rnd];
        Block {
            style,
            color: JsValue::from_str("coral"),
            coords: match style {
                BlockStyle::I => [
                    Coord::new(1, 0),
                    Coord::new(1, 1),
                    Coord::new(1, 2),
                    Coord::new(1, 3),
                ],
                BlockStyle::J => [
                    Coord::new(1, 0),
                    Coord::new(1, 1),
                    Coord::new(1, 2),
                    Coord::new(0, 2),
                ],
                BlockStyle::L => [
                    Coord::new(1, 0),
                    Coord::new(1, 1),
                    Coord::new(1, 2),
                    Coord::new(2, 2),
                ],
                BlockStyle::O => [
                    Coord::new(1, 0),
                    Coord::new(2, 0),
                    Coord::new(2, 1),
                    Coord::new(1, 1),
                ],
                BlockStyle::S => [
                    Coord::new(0, 1),
                    Coord::new(1, 1),
                    Coord::new(1, 0),
                    Coord::new(2, 0),
                ],
                BlockStyle::T => [
                    Coord::new(0, 1),
                    Coord::new(1, 1),
                    Coord::new(2, 1),
                    Coord::new(1, 2),
                ],
                BlockStyle::Z => [
                    Coord::new(0, 0),
                    Coord::new(1, 0),
                    Coord::new(1, 1),
                    Coord::new(2, 1),
                ],
            },
        }
    }
}

/*impl Block {
    pub fn draw(&mut self, context: &CanvasRenderingContext2d) {
        for coord in self.board.block.coords.iter() {
            context.set_fill_style(&self.board.block.color);
            context.fill_rect(
                ((self.board.pos.x + coord.x) * self.board.cell_size_px) as f64,
                ((self.board.pos.y + coord.y) * self.board.cell_size_px) as f64,
                self.board.cell_size_px as f64,
                self.board.cell_size_px as f64,
            );
            context.set_stroke_style(border);
            context.stroke_rect(
                ((self.board.pos.x + coord.x) * self.board.cell_size_px) as f64,
                ((self.board.pos.y + coord.y) * self.board.cell_size_px) as f64,
                self.board.cell_size_px as f64,
                self.board.cell_size_px as f64,
            );
        }
    }
}*/

#[derive(Debug, Clone)]
pub struct Board {
    rows: usize,
    cols: usize,
    cell_size_px: usize,
    cells: Vec<Vec<Cell>>,
    block: Block,
    pos: Coord,
    rotation: usize,
}

impl Board {
    fn new(rows: usize, cols: usize, cell_size_px: usize) -> Board {
        Board {
            rows,
            cols,
            cell_size_px,
            cells: vec![vec![Cell::Empty; cols]; rows],
            block: Block::next(),
            pos: Coord::new(4, 0),
            rotation: 0,
        }
    }
}

impl Board {
    fn check_collision(&mut self, new_pos: Coord) -> bool {
        self.block.coords.iter().any(|coord| {
            let x = &coord.x + new_pos.x;
            let y = &coord.y + new_pos.y;
            // TODO - why does this not throw index oob? could this ever cause fuckiness?
            y >= self.rows || self.cells[y][x] == Cell::Filled
        })
    }
}

impl Board {
    fn fill_block(&mut self) {
        for coord in self.block.coords.iter() {
            self.cells[coord.y + self.pos.y][coord.x + self.pos.x] = Cell::Filled;
        }

        self.pos = Coord::new(4, 0);
        self.block = Block::next();
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
            }
        }

        // Active Block
        for coord in self.board.block.coords.iter() {
            self.context.set_fill_style(&self.board.block.color);
            self.context.fill_rect(
                ((self.board.pos.x + coord.x) * self.board.cell_size_px) as f64,
                ((self.board.pos.y + coord.y) * self.board.cell_size_px) as f64,
                self.board.cell_size_px as f64,
                self.board.cell_size_px as f64,
            );
            self.context.set_stroke_style(border);
            self.context.stroke_rect(
                ((self.board.pos.x + coord.x) * self.board.cell_size_px) as f64,
                ((self.board.pos.y + coord.y) * self.board.cell_size_px) as f64,
                self.board.cell_size_px as f64,
                self.board.cell_size_px as f64,
            );
        }

        // Border
        self.context.set_stroke_style(border);
        self.context.stroke_rect(
            0.0, 
            0.0, 
            (self.board.cols * self.board.cell_size_px) as f64, 
            (self.board.rows * self.board.cell_size_px) as f64);
    }
}

#[wasm_bindgen]
impl Game {
    pub fn tick(&mut self) {
        let new_pos = Coord::new(self.board.pos.x, self.board.pos.y + 1);
        //let msg = format!("{:?}", new_pos);
        //alert(&msg);
        if self.board.check_collision(new_pos.clone()) {
            self.board.fill_block();
        } else {
            self.board.pos = new_pos;
        }
        self.draw();
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

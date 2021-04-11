use crate::coord::Coord;
use crate::JsValue;
use crate::CanvasRenderingContext2d;
use crate::js_sys;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockStyle {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

pub type BlockCoords = [Coord; 4];

#[derive(Debug, Clone)]
pub struct Block {
    pub style: BlockStyle,
    pub color: JsValue,
    pub coords: BlockCoords,
}

use self::BlockStyle::*;
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
                    Coord::new(0, -1),
                    Coord::new(0, 0),
                    Coord::new(0, 1),
                    Coord::new(-1, 1),
                ],
                BlockStyle::L => [
                    Coord::new(0, -1),
                    Coord::new(0, 0),
                    Coord::new(0, 1),
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
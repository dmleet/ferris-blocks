use crate::coord::Coord;
use crate::block::*;

//#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty = 0,
    Filled = 1,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub rows: usize,
    pub cols: usize,
    pub cell_size_px: usize,
    pub cells: Vec<Vec<Cell>>,
    pub block: Block,
    pub pos: Coord,
    pub score: u32,
    pub level: u32,
}

impl Board {
    pub fn new(rows: usize, cols: usize, cell_size_px: usize) -> Board {
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
    pub fn check_collision(&mut self, new_pos: Coord, new_coords: BlockCoords) -> bool {
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
    pub fn update(&mut self) {
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
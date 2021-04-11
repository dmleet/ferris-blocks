#[derive(Debug, Clone, Copy)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl std::ops::Add<Coord> for Coord {
    type Output = Coord;

    fn add(self, _rhs: Coord) -> Coord {
        Coord::new(self.x + _rhs.x, self.y + _rhs.y)
    }
}

impl std::ops::Sub<Coord> for Coord {
    type Output = Coord;

    fn sub(self, _rhs: Coord) -> Coord {
        Coord::new(self.x - _rhs.x, self.y - _rhs.y)
    }
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Coord {
        Coord { x, y }
    }
}

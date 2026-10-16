//! The battlefield around one tower: a square grid of cells. Enemies enter
//! at one edge and walk the shortest open route to the tower, found by a
//! breadth-first search outward from the tower (a flow field). Walls and
//! other blocking features close cells; a placement that would leave no
//! route at all is refused, so a wall may bend the path but never close it.

use serde::{Deserialize, Serialize};

use crate::fx::{Fx, ONE};
use crate::geom::{v3, V3};

pub const SIZE: i32 = 32;
/// Cell edge, 1.5 m.
pub const CELL: Fx = ONE * 3 / 2;
pub const UNREACHABLE: u16 = u16::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Open,
    /// Trees and rocks the seed placed.
    Scenery,
    /// A placed feature that blocks, by its index in the arena's feature list.
    Feature(u16),
    Tower,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arena {
    pub cells: Vec<Cell>,
    pub entry: (i32, i32),
    pub tower: (i32, i32),
    /// Steps from each cell to the tower, by the 8-neighbour search.
    pub dist: Vec<u16>,
}

pub fn idx(x: i32, z: i32) -> usize {
    (z * SIZE + x) as usize
}

pub fn inside(x: i32, z: i32) -> bool {
    (0..SIZE).contains(&x) && (0..SIZE).contains(&z)
}

/// The centre of a cell in metres; the grid is centred on the origin.
pub fn centre(x: i32, z: i32) -> V3 {
    let half = SIZE * CELL / 2;
    v3(x * CELL + CELL / 2 - half, 0, z * CELL + CELL / 2 - half)
}

/// The cell a point lies in, clamped onto the grid.
pub fn cell_of(p: V3) -> (i32, i32) {
    let half = SIZE * CELL / 2;
    let c = |v: Fx| ((v + half).div_euclid(CELL)).clamp(0, SIZE - 1);
    (c(p.x), c(p.z))
}

const STEPS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

impl Arena {
    /// An empty field with the entry and tower where every arena has them.
    pub fn new() -> Arena {
        let mut a = Arena {
            cells: vec![Cell::Open; (SIZE * SIZE) as usize],
            entry: (2, SIZE - 3),
            tower: (SIZE - 5, 4),
            dist: Vec::new(),
        };
        // the tower takes a 3×3 footprint
        for dz in -1..=1 {
            for dx in -1..=1 {
                a.cells[idx(a.tower.0 + dx, a.tower.1 + dz)] = Cell::Tower;
            }
        }
        a.recompute();
        a
    }

    pub fn at(&self, x: i32, z: i32) -> Cell {
        if inside(x, z) {
            self.cells[idx(x, z)]
        } else {
            Cell::Scenery
        }
    }

    fn passable(&self, x: i32, z: i32) -> bool {
        matches!(self.at(x, z), Cell::Open)
    }

    /// Breadth-first search from the tower's footprint over open cells. A
    /// diagonal step needs both side cells open, so no route cuts a corner.
    fn search(&self) -> Vec<u16> {
        let mut dist = vec![UNREACHABLE; (SIZE * SIZE) as usize];
        let mut queue: Vec<(i32, i32)> = Vec::new();
        for z in 0..SIZE {
            for x in 0..SIZE {
                if self.at(x, z) == Cell::Tower {
                    dist[idx(x, z)] = 0;
                    queue.push((x, z));
                }
            }
        }
        let mut head = 0;
        while head < queue.len() {
            let (x, z) = queue[head];
            head += 1;
            let d = dist[idx(x, z)];
            for (dx, dz) in STEPS {
                let (nx, nz) = (x + dx, z + dz);
                if !self.passable(nx, nz) || dist[idx(nx, nz)] != UNREACHABLE {
                    continue;
                }
                if dx != 0 && dz != 0 && !(self.passable(x + dx, z) && self.passable(x, z + dz)) {
                    continue;
                }
                dist[idx(nx, nz)] = d + 1;
                queue.push((nx, nz));
            }
        }
        dist
    }

    pub fn recompute(&mut self) {
        self.dist = self.search();
    }

    pub fn reachable(&self) -> bool {
        self.dist[idx(self.entry.0, self.entry.1)] != UNREACHABLE
    }

    /// Blocks the cells of a feature if a route remains; otherwise leaves
    /// the arena as it was and says so.
    pub fn block(&mut self, cells: &[(i32, i32)], feature: u16) -> Result<(), BlockError> {
        for &(x, z) in cells {
            if !inside(x, z) {
                return Err(BlockError::OffGrid);
            }
            if (x, z) == self.entry {
                return Err(BlockError::Entry);
            }
            if self.at(x, z) != Cell::Open {
                return Err(BlockError::Taken);
            }
        }
        let before = self.cells.clone();
        for &(x, z) in cells {
            self.cells[idx(x, z)] = Cell::Feature(feature);
        }
        let dist = self.search();
        if dist[idx(self.entry.0, self.entry.1)] == UNREACHABLE {
            self.cells = before;
            return Err(BlockError::ClosesPath);
        }
        self.dist = dist;
        Ok(())
    }

    /// Opens every cell a feature held, as when a glass wall breaks.
    pub fn clear_feature(&mut self, feature: u16) {
        for c in self.cells.iter_mut() {
            if *c == Cell::Feature(feature) {
                *c = Cell::Open;
            }
        }
        self.recompute();
    }

    /// Where a walker in this cell heads next: the centre of the
    /// neighbouring cell nearest the tower. In the tower's own footprint,
    /// or cut off, it heads for the tower.
    pub fn next_step(&self, from: V3) -> V3 {
        let (x, z) = cell_of(from);
        let here = self.dist[idx(x, z)];
        let mut best = (here, (x, z));
        for (dx, dz) in STEPS {
            let (nx, nz) = (x + dx, z + dz);
            if !inside(nx, nz) {
                continue;
            }
            if dx != 0 && dz != 0 && !(self.passable(x + dx, z) && self.passable(x, z + dz)) {
                continue;
            }
            let d = self.dist[idx(nx, nz)];
            if d < best.0 {
                best = (d, (nx, nz));
            }
        }
        if best.0 == UNREACHABLE || best.1 == (x, z) {
            return centre(self.tower.0, self.tower.1);
        }
        centre(best.1 .0, best.1 .1)
    }

    /// The route from the entry to the tower, cell by cell, for drawing the trail.
    pub fn route(&self) -> Vec<(i32, i32)> {
        let mut out = vec![self.entry];
        let mut at = self.entry;
        for _ in 0..(SIZE * SIZE) {
            let d = self.dist[idx(at.0, at.1)];
            if d == 0 || d == UNREACHABLE {
                break;
            }
            let next = crate::arena::cell_of(self.next_step(centre(at.0, at.1)));
            if next == at {
                break;
            }
            out.push(next);
            at = next;
        }
        out
    }
}

impl Default for Arena {
    fn default() -> Self {
        Arena::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockError {
    OffGrid,
    Entry,
    Taken,
    ClosesPath,
}

impl BlockError {
    /// The key of the string the page shows.
    pub fn key(self) -> &'static str {
        match self {
            BlockError::OffGrid => "layout.off_grid",
            BlockError::Entry => "layout.entry",
            BlockError::Taken => "layout.taken",
            BlockError::ClosesPath => "layout.closes_path",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_open_field_reaches_the_tower_by_diagonal_steps() {
        let a = Arena::new();
        assert!(a.reachable());
        // entry (2,29) to the footprint's edge (26,5): max(24,24) = 24 diagonal steps
        assert_eq!(a.dist[idx(2, 29)], 24);
    }

    #[test]
    fn a_wall_bends_the_route_and_a_closing_wall_is_refused() {
        let mut a = Arena::new();
        // a wall across the whole width except one gap at x = 0
        let wall: Vec<(i32, i32)> = (1..SIZE).map(|x| (x, 16)).collect();
        assert_eq!(a.block(&wall, 0), Ok(()));
        assert!(a.reachable());
        assert!(a.dist[idx(2, 29)] > 24, "the gap at x = 0 makes the route longer");
        // closing the last gap would cut the entry off: refused, nothing changes
        let before = a.clone();
        assert_eq!(a.block(&[(0, 16)], 1), Err(BlockError::ClosesPath));
        assert_eq!(a, before);
        assert_eq!(BlockError::ClosesPath.key(), "layout.closes_path");
    }

    #[test]
    fn a_route_does_not_cut_a_corner_between_two_walls() {
        let mut a = Arena::new();
        // two blocks touching at a corner leave no diagonal squeeze
        a.block(&[(10, 10)], 0).unwrap();
        a.block(&[(11, 11)], 1).unwrap();
        let from = centre(10, 11);
        let (sx, sz) = cell_of(a.next_step(from));
        let (dx, dz) = (sx - 10, sz - 11);
        // every step to x = 11 is equally near the tower (15), and both
        // diagonals there squeeze past a wall: (11,10) past both, (11,12) past (11,11)
        if dx != 0 && dz != 0 {
            assert!(
                a.at(10 + dx, 11) == Cell::Open && a.at(10, 11 + dz) == Cell::Open,
                "cut a corner to ({sx},{sz})"
            );
        }
    }

    #[test]
    fn clearing_a_feature_reopens_its_cells() {
        let mut a = Arena::new();
        a.block(&[(5, 5), (6, 5)], 3).unwrap();
        a.clear_feature(3);
        assert_eq!(a.at(5, 5), Cell::Open);
        assert_eq!(a.at(6, 5), Cell::Open);
    }

    #[test]
    fn the_route_runs_from_the_entry_to_the_tower() {
        let a = Arena::new();
        let r = a.route();
        assert_eq!(r[0], a.entry);
        let last = *r.last().unwrap();
        assert_eq!(a.dist[idx(last.0, last.1)], 0, "ends on the tower's footprint");
        // distances 24 down to 0, one cell each
        assert_eq!(r.len(), 25);
    }
}

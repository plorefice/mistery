use crate::math::{Point, Rect};

use rand::Rng;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq)]
pub enum TileKind {
    Wall,
    Floor,
}

#[derive(Default)]
pub struct WorldMap {
    width: u32,
    height: u32,
    rooms: Vec<Rect>,
    tiles: Vec<TileKind>,
    revealed: Vec<bool>,
    visible: Vec<bool>,
}

impl WorldMap {
    pub fn rooms_and_corridors(width: u32, height: u32) -> WorldMap {
        const MAX_ROOMS: usize = 30;
        const MIN_SIZE: u32 = 7;
        const MAX_SIZE: u32 = 12;

        let n = (width * height) as usize;

        let mut map = WorldMap {
            width,
            height,
            rooms: Vec::with_capacity(MAX_ROOMS),
            tiles: vec![TileKind::Wall; n],
            revealed: vec![false; n],
            visible: vec![false; n],
        };

        let mut rng = rand::thread_rng();

        for _ in 0..MAX_ROOMS {
            let w = rng.gen_range(MIN_SIZE, MAX_SIZE);
            let h = rng.gen_range(MIN_SIZE, MAX_SIZE);
            let x = rng.gen_range(1, width - w - 1);
            let y = rng.gen_range(1, height - h - 1);

            let r = Rect::new(x, y, w, h);

            if !map.rooms.iter().any(|r2| r.intersects(r2)) {
                map.create_room(&r);

                if let Some(rp) = map.rooms.last() {
                    let (x1, y1) = (rp.center()[0], rp.center()[1]);
                    let (x2, y2) = (r.center()[0], r.center()[1]);

                    if rng.gen::<bool>() {
                        map.create_horizontal_corridor(x1, x2, y1);
                        map.create_vertical_corridor(y1, y2, x2);
                    } else {
                        map.create_vertical_corridor(y1, y2, x1);
                        map.create_horizontal_corridor(x1, x2, y2);
                    }
                }

                map.rooms.push(r);
            }
        }

        map
    }

    /// Returns the map's width, ie. the number of columns.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the map's height, ie. the number of rows.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the tile at the given point, if present.
    pub fn get(&self, p: &Point) -> Option<TileKind> {
        self.tiles.get(self.pt_to_idx(p)).cloned()
    }

    /// Returns whether the tile at the given point has been revealed, if present.
    pub fn is_revealed(&self, p: &Point) -> Option<bool> {
        self.revealed.get(self.pt_to_idx(p)).cloned()
    }

    /// Returns whether the tile at the given point is currently visible, if present.
    pub fn is_visible(&self, p: &Point) -> Option<bool> {
        self.visible.get(self.pt_to_idx(p)).cloned()
    }

    /// Marks the tile at the given point as revealed.
    pub fn reveal(&mut self, p: &Point) {
        let idx = self.pt_to_idx(p);
        self.revealed[idx] = true;
    }

    /// Changes the visibility of the tile at the given point.
    pub fn set_visible(&mut self, p: &Point, visible: bool) {
        let idx = self.pt_to_idx(p);
        self.visible[idx] = visible;
    }

    /// Sets all tiles as not visible.
    pub fn clear_visibility(&mut self) {
        for viz in self.visible.iter_mut() {
            *viz = false;
        }
    }

    /// Returns the a reference to the rooms in this map.
    pub fn rooms(&self) -> &[Rect] {
        &self.rooms
    }

    fn xy_to_idx(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    fn pt_to_idx(&self, p: &Point) -> usize {
        (p.y() * self.width + p.x()) as usize
    }

    fn create_room(&mut self, room: &Rect) {
        for y in room.bottom() + 1..room.top() {
            for x in room.left() + 1..room.right() {
                let idx = self.xy_to_idx(x, y);
                self.tiles[idx] = TileKind::Floor
            }
        }
    }

    fn create_horizontal_corridor(&mut self, x1: u32, x2: u32, y: u32) {
        for x in x1.min(x2)..=x1.max(x2) {
            let idx = self.xy_to_idx(x, y);
            self.tiles[idx] = TileKind::Floor;
        }
    }

    fn create_vertical_corridor(&mut self, y1: u32, y2: u32, x: u32) {
        for y in y1.min(y2)..=y1.max(y2) {
            let idx = self.xy_to_idx(x, y);
            self.tiles[idx] = TileKind::Floor;
        }
    }
}

/// Implementation of the FoV algorithm using recursive shadowcasting.
///
/// The algorithm itself is described in great detail at [RogueBasin].
/// This is based on the C++ implementation of the algorithm available [here].
///
/// [RogueBasin]: http://roguebasin.roguelikedevelopment.org/index.php?title=FOV_using_recursive_shadowcasting
/// [here]: http://roguebasin.roguelikedevelopment.org/index.php?title=C%2B%2B_shadowcasting_implementation
pub struct ShadowcastFoV<'a> {
    x: i32,
    y: i32,
    radius: i32,
    map: &'a WorldMap,
    visible: HashSet<Point>,
}

impl<'a> ShadowcastFoV<'a> {
    const DIAGONALS_MULTIPLIES: [[i32; 8]; 4] = [
        [1, 0, 0, -1, -1, 0, 0, 1],
        [0, 1, -1, 0, 0, -1, 1, 0],
        [0, 1, 1, 0, 0, -1, -1, 0],
        [1, 0, 0, 1, -1, 0, 0, -1],
    ];

    /// Executes a run of the algorithm on the map for the specified circle.
    pub fn run(map: &WorldMap, x: u32, y: u32, radius: u32) -> HashSet<Point> {
        let mut fov = ShadowcastFoV {
            map,
            x: x as i32,
            y: y as i32,
            radius: radius as i32,
            visible: HashSet::with_capacity((radius * radius * 4) as usize),
        };

        for i in 0..8 {
            fov.cast_light(
                1,
                1.0,
                0.0,
                (
                    ShadowcastFoV::DIAGONALS_MULTIPLIES[0][i],
                    ShadowcastFoV::DIAGONALS_MULTIPLIES[1][i],
                    ShadowcastFoV::DIAGONALS_MULTIPLIES[2][i],
                    ShadowcastFoV::DIAGONALS_MULTIPLIES[3][i],
                ),
            );
        }

        fov.visible
    }

    fn cast_light(&mut self, row: i32, mut start: f32, end: f32, mul: (i32, i32, i32, i32)) {
        let mut blocked = false;
        let mut next_start_slope = start;

        if start < end {
            return;
        }

        for i in row..=self.radius {
            if blocked {
                break;
            }
            for dx in -i..=0 {
                let dy = -i;
                let l_slope = (dx as f32 - 0.5) / (dy as f32 + 0.5);
                let r_slope = (dx as f32 + 0.5) / (dy as f32 - 0.5);

                if start < r_slope {
                    continue;
                } else if end > l_slope {
                    break;
                }

                let sax = dx * mul.0 + dy * mul.1;
                let say = dx * mul.2 + dy * mul.3;
                if (sax < 0 && sax.abs() > self.x) || (say < 0 && say.abs() > self.y) {
                    continue;
                }

                let ax = self.x + sax;
                let ay = self.y + say;
                if ax >= self.map.width() as i32 || ay >= self.map.height() as i32 {
                    continue;
                }

                let radius2 = self.radius * self.radius;
                if (dx * dx + dy * dy) < radius2 {
                    self.visible.insert(Point::new(ax as u32, ay as u32));
                }

                if blocked {
                    if self.map.get(&(ax as u32, ay as u32).into()) == Some(TileKind::Wall) {
                        next_start_slope = r_slope;
                        continue;
                    } else {
                        blocked = false;
                        start = next_start_slope;
                    }
                } else if self.map.get(&(ax as u32, ay as u32).into()) == Some(TileKind::Wall) {
                    blocked = true;
                    self.cast_light(i + 1, start, l_slope, mul);
                    next_start_slope = r_slope;
                }
            }
        }
    }
}

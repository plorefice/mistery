use crate::math::{Point2, Rect};

use rand::Rng;

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

    /// Returns the tile at coordinates `(x, y)`, if present.
    pub fn get(&self, x: u32, y: u32) -> Option<TileKind> {
        self.tiles.get(self.xy_to_idx(x, y)).cloned()
    }

    /// Returns whether the tile at coordinates `(x, y)` has been revealed, if present.
    pub fn is_revealed(&self, x: u32, y: u32) -> Option<bool> {
        self.revealed.get(self.xy_to_idx(x, y)).cloned()
    }

    /// Returns whether the tile at coordinates `(x, y)` is currently visible, if present.
    pub fn is_visible(&self, x: u32, y: u32) -> Option<bool> {
        self.visible.get(self.xy_to_idx(x, y)).cloned()
    }

    /// Marks the tile at coordinates `(x, y)` as revealed.
    pub fn reveal(&mut self, x: u32, y: u32) {
        let idx = self.xy_to_idx(x, y);
        self.revealed[idx] = true;
    }

    /// Changes the visibility of the tile at coordinates `(x, y)`.
    pub fn set_visible(&mut self, x: u32, y: u32, visible: bool) {
        let idx = self.xy_to_idx(x, y);
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

fn cast_light(
    map: &WorldMap,
    x: u32,
    y: u32,
    radius: u32,
    row: u32,
    mut start_slope: f32,
    end_slope: f32,
    xx: i32,
    xy: i32,
    yx: i32,
    yy: i32,
) -> Vec<Point2<u32>> {
    let mut visible = vec![];

    if start_slope < end_slope {
        return visible;
    }

    let mut next_start_slope = start_slope;

    for i in row..=radius {
        let mut blocked = false;

        for dx in -(i as i32)..=0 {
            let dy = -(i as i32);

            let l_slope = (dx as f32 - 0.5) / (dy as f32 + 0.5);
            let r_slope = (dx as f32 + 0.5) / (dy as f32 - 0.5);

            if start_slope < r_slope {
                continue;
            } else if end_slope > l_slope {
                break;
            }

            let sax = dx * xx + dy * xy;
            let say = dx * yx + dy * yy;
            if (sax < 0 && sax.abs() as u32 > x) || (say < 0 && say.abs() as u32 > y) {
                continue;
            }

            let ax = (x as i32 + sax) as u32;
            let ay = (y as i32 + say) as u32;
            if ax >= map.width() || ay >= map.height() {
                continue;
            }

            let radius2 = radius * radius;
            if (dx * dx + dy * dy) < radius2 as i32 {
                visible.push(Point2::new(ax, ay));
            }

            if blocked {
                if map.get(ax as u32, ay as u32) == Some(TileKind::Wall) {
                    next_start_slope = r_slope;
                    continue;
                } else {
                    blocked = false;
                    start_slope = next_start_slope;
                }
            } else if map.get(ax as u32, ay as u32) == Some(TileKind::Wall) {
                blocked = true;
                next_start_slope = r_slope;
                cast_light(
                    map,
                    x,
                    y,
                    radius,
                    i + 1,
                    start_slope,
                    l_slope,
                    xx,
                    xy,
                    yx,
                    yy,
                );
            }
        }
        if blocked {
            break;
        }
    }

    visible
}

pub fn do_fov(map: &WorldMap, x: u32, y: u32, radius: u32) -> Vec<Point2<u32>> {
    let mut visible = Vec::new();

    for i in 0..8 {
        visible.append(&mut cast_light(
            map,
            x,
            y,
            radius,
            1,
            1.0,
            0.0,
            MULTIPLIERS[0][i],
            MULTIPLIERS[1][i],
            MULTIPLIERS[2][i],
            MULTIPLIERS[3][i],
        ));
    }

    visible
}

const MULTIPLIERS: [[i32; 8]; 4] = [
    [1, 0, 0, -1, -1, 0, 0, 1],
    [0, 1, -1, 0, 0, -1, 1, 0],
    [0, 1, 1, 0, 0, -1, -1, 0],
    [1, 0, 0, 1, -1, 0, 0, -1],
];

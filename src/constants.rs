use std::collections::HashMap;

use rand::Rng;

pub struct Tiles {
    pub floor1: Position,
    pub floor2: Position,
    pub floor3: Position,
    pub floor4: Position,
    pub floor5: Position,
    pub floor6: Position,
    pub floor7: Position,
    pub wall_top_edge_left: Position,
    pub wall_top_mid: Position,
    pub wall_top_edge_right: Position,
    pub wall_top_cross_edge_right: Position,
    pub wall_top_cross_edge_left: Position,
    pub wall_bottom_cross_edge_left: Position,
    pub wall_bottom_cross_edge_right: Position,
    pub wall_right: Position,
    pub wall_bottom_edge_right_top: Position,
    pub wall_bottom_edge_right_bottom: Position,
    pub wall_bottom_mid_bottom: Position,
    pub wall_bottom_mid_top: Position,
    pub wall_bottom_edge_left_top: Position,
    pub wall_bottom_edge_left_bottom: Position,
    pub wall_left: Position,
    pub wall_ceil: Position,
    pub player_walk_down_1: Position,
    pub player_walk_down_2: Position,
    pub player_walk_down_3: Position,
    pub player_walk_down_4: Position,
}

pub const TILES: Tiles = Tiles {
    floor1: Position {
        x: 0. * SPRITE_SIZE,
        y: 0. * SPRITE_SIZE,
    },
    floor2: Position {
        x: 1. * SPRITE_SIZE,
        y: 0. * SPRITE_SIZE,
    },
    floor3: Position {
        x: 2. * SPRITE_SIZE,
        y: 0. * SPRITE_SIZE,
    },
    floor4: Position {
        x: 3. * SPRITE_SIZE,
        y: 0. * SPRITE_SIZE,
    },
    floor5: Position {
        x: 4. * SPRITE_SIZE,
        y: 0. * SPRITE_SIZE,
    },
    floor6: Position {
        x: 5. * SPRITE_SIZE,
        y: 0. * SPRITE_SIZE,
    },
    floor7: Position {
        x: 6. * SPRITE_SIZE,
        y: 0. * SPRITE_SIZE,
    },
    wall_top_edge_left: Position {
        x: 0. * SPRITE_SIZE,
        y: 1. * SPRITE_SIZE,
    },
    wall_top_mid: Position {
        x: 1. * SPRITE_SIZE,
        y: 1. * SPRITE_SIZE,
    },
    wall_top_edge_right: Position {
        x: 2. * SPRITE_SIZE,
        y: 1. * SPRITE_SIZE,
    },
    wall_right: Position {
        x: 2. * SPRITE_SIZE,
        y: 2. * SPRITE_SIZE,
    },
    wall_bottom_edge_right_top: Position {
        x: 2. * SPRITE_SIZE,
        y: 3. * SPRITE_SIZE,
    },
    wall_bottom_edge_right_bottom: Position {
        x: 2. * SPRITE_SIZE,
        y: 4. * SPRITE_SIZE,
    },
    wall_bottom_mid_bottom: Position {
        x: 1. * SPRITE_SIZE,
        y: 4. * SPRITE_SIZE,
    },
    wall_bottom_mid_top: Position {
        x: 1. * SPRITE_SIZE,
        y: 3. * SPRITE_SIZE,
    },
    wall_bottom_edge_left_top: Position {
        x: 0. * SPRITE_SIZE,
        y: 3. * SPRITE_SIZE,
    },
    wall_bottom_edge_left_bottom: Position {
        x: 0. * SPRITE_SIZE,
        y: 4. * SPRITE_SIZE,
    },
    wall_left: Position {
        x: 0. * SPRITE_SIZE,
        y: 2. * SPRITE_SIZE,
    },
    wall_ceil: Position {
        x: 1. * SPRITE_SIZE,
        y: 2. * SPRITE_SIZE,
    },
    player_walk_down_1: Position {
        x: 3. * SPRITE_SIZE,
        y: 2. * SPRITE_SIZE,
    },
    player_walk_down_2: Position {
        x: 4. * SPRITE_SIZE,
        y: 2. * SPRITE_SIZE,
    },
    player_walk_down_3: Position {
        x: 5. * SPRITE_SIZE,
        y: 2. * SPRITE_SIZE,
    },
    player_walk_down_4: Position {
        x: 6. * SPRITE_SIZE,
        y: 2. * SPRITE_SIZE,
    },
    wall_top_cross_edge_right: Position {
        x: 0. * SPRITE_SIZE,
        y: 6. * SPRITE_SIZE,
    },
    wall_top_cross_edge_left: Position {
        x: 2. * SPRITE_SIZE,
        y: 6. * SPRITE_SIZE,
    },
    wall_bottom_cross_edge_right: Position {
        x: 0. * SPRITE_SIZE,
        y: 5. * SPRITE_SIZE,
    },
    wall_bottom_cross_edge_left: Position {
        x: 2. * SPRITE_SIZE,
        y: 5. * SPRITE_SIZE,
    },
};

pub const TILE_SIZE: usize = 48;
pub const SPRITE_SIZE: f32 = 16.;

pub const MAP: &str = r##"
########################################
########################################
########################################
#......................................#
#......................................#
#......................................#
#......................................#
#......................................#
#...############........#####..#####...#
#...############........#####..#####...#
#...############........#####..#####...#
#...##........##........##........##...#
#...##........##.......................#
#...##........##.......................#
#...##........##........##........##...#
#...##........##........##........##...#
#...#####..#####........#####..#####...#
#...#####..#####........#####..#####...#
#...#####..#####........#####..#####...#
#......................................#
#......................................#
#......................................#
#......................................#
#......................................#
########################################
"##;

const FLOOR_TILES: [Position; 7] = [
    TILES.floor1,
    TILES.floor2,
    TILES.floor3,
    TILES.floor4,
    TILES.floor5,
    TILES.floor6,
    TILES.floor7,
];

fn _get_floor_tile() -> Position {
    let mut rng = rand::thread_rng();
    let tile_num: usize = rng.gen_range(0..7);
    FLOOR_TILES[tile_num]
}

type ParsedMap = (HashMap<(usize, usize), Position>, Vec<f32>, u32, u32);

pub fn parse_map(map: &str) -> ParsedMap {
    let mut tiles = HashMap::new();
    let map_lines: Vec<&str> = map.trim().split('\n').rev().collect();
    let map_height = map_lines.len();
    let map_width = map_lines[0].len();
    let texture_width = map_width * TILE_SIZE;
    let texture_height = map_height * TILE_SIZE;
    let mut sdf_data = vec![0.0; texture_width * texture_height];

    for (y, line) in map_lines.iter().enumerate() {
        for (x, char) in line.chars().enumerate() {
            let (tile, color) = match char {
                '#' => (determine_wall_type(x, y, &map_lines), 0.1),
                _ => (_get_floor_tile(), f32::MAX),
            };
            tiles.insert((x, y), tile);

            for ty in 0..TILE_SIZE {
                for tx in 0..TILE_SIZE {
                    let index =
                        ((y * TILE_SIZE + ty) * texture_width + (x * TILE_SIZE + tx)) as usize;
                    sdf_data[index] = color;
                }
            }
        }
    }

    (tiles, sdf_data, texture_width as u32, texture_height as u32)
}

fn determine_wall_type(x: usize, y: usize, map_lines: &Vec<&str>) -> Position {
    let y_max = map_lines.len() - 1;
    let x_max = map_lines[y].len() - 1;
    let two_below_is = if y > 1 {
        map_lines[y - 2].chars().nth(x)
    } else {
        None
    };
    let two_below_right_is = if y > 1 && x < x_max {
        map_lines[y - 2].chars().nth(x + 1)
    } else {
        None
    };
    let two_below_left_is = if y > 1 && x > 0 {
        map_lines[y - 2].chars().nth(x - 1)
    } else {
        None
    };
    let below_is = if y > 0 {
        map_lines[y - 1].chars().nth(x)
    } else {
        None
    };
    let above_is = if y < y_max {
        map_lines[y + 1].chars().nth(x)
    } else {
        None
    };
    let right_is = if x < x_max {
        map_lines[y].chars().nth(x + 1)
    } else {
        None
    };
    let left_is = if x > 0 {
        map_lines[y].chars().nth(x - 1)
    } else {
        None
    };
    let top_right_is = if y < y_max && x < x_max {
        map_lines[y + 1].chars().nth(x + 1)
    } else {
        None
    };
    let top_left_is = if y < y_max && x > 0 {
        map_lines[y + 1].chars().nth(x - 1)
    } else {
        None
    };
    let bottom_right_is = if y > 0 && x < x_max {
        map_lines[y - 1].chars().nth(x + 1)
    } else {
        None
    };
    let bottom_left_is = if y > 0 && x > 0 {
        map_lines[y - 1].chars().nth(x - 1)
    } else {
        None
    };
    match (
        above_is,
        below_is,
        right_is,
        left_is,
        two_below_is,
        top_right_is,
        top_left_is,
        bottom_right_is,
        bottom_left_is,
        two_below_right_is,
        two_below_left_is,
    ) {
        (
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
        ) => TILES.wall_ceil,
        (
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('.'),
            Some('#') | None,
        ) => TILES.wall_bottom_cross_edge_right,
        (
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            Some('.'),
        ) => TILES.wall_bottom_cross_edge_left,
        (
            Some('#'),
            Some('#') | None,
            Some('#'),
            Some('#') | None,
            Some('#') | None,
            Some('.'),
            Some('#') | None,
            Some('#') | None,
            Some('#') | None,
            ..,
        ) => TILES.wall_top_cross_edge_right,
        (
            Some('#'),
            Some('#') | None,
            Some('#') | None,
            Some('#'),
            Some('#') | None,
            Some('#') | None,
            Some('.'),
            Some('#') | None,
            Some('#') | None,
            ..,
        ) => TILES.wall_top_cross_edge_left,
        (
            Some('#'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('.'),
            Some('#'),
            ..,
        ) => TILES.wall_right,
        (
            Some('#'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('.'),
            ..,
        ) => TILES.wall_left,
        (
            Some('#'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('.'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('.'),
            Some('.'),
            Some('.'),
        ) => TILES.wall_bottom_edge_left_top,
        (
            Some('#'),
            Some('#'),
            Some('#'),
            Some('#'),
            Some('.'),
            Some('#'),
            Some('#'),
            Some('.'),
            Some('#'),
            Some('.'),
            Some('.'),
        ) => TILES.wall_bottom_edge_right_top,
        (Some('#'), Some('#'), Some('#'), Some('.'), Some('.'), ..) => {
            TILES.wall_bottom_edge_left_top
        }
        (Some('#'), Some('#'), Some('.'), Some('#'), Some('#'), ..) => TILES.wall_right,
        (Some('#'), Some('#'), Some('#'), Some('.'), Some('#'), ..) => TILES.wall_left,
        (Some('.'), Some('#'), Some('#'), Some('.'), Some('#'), ..) => TILES.wall_top_edge_left,
        (Some('.'), Some('#'), Some('.'), Some('#'), Some('#'), ..) => TILES.wall_top_edge_right,
        (Some('#'), Some('.'), Some('#'), Some('.'), ..) => TILES.wall_bottom_edge_left_bottom,
        (Some('#'), Some('#'), Some('.'), Some('#'), ..) => TILES.wall_bottom_edge_right_top,
        (Some('#'), Some('.'), Some('.'), Some('#'), ..) => TILES.wall_bottom_edge_right_bottom,
        (Some('#'), Some('.'), Some('#'), Some('#'), ..) => TILES.wall_bottom_mid_bottom,
        (Some('.'), Some('#') | None, Some('#'), Some('#'), Some('#') | None, ..) => {
            TILES.wall_top_mid
        }
        (_, Some('#'), Some('#'), Some('#'), ..) => TILES.wall_bottom_mid_top,
        (Some('#') | None, Some('#'), Some('.') | Some('#'), ..) => TILES.wall_right,
        (Some('#') | None, Some('#'), _, Some('.') | Some('#'), ..) => TILES.wall_left,
        _ => TILES.player_walk_down_4,
    }
}

// THis has potential, but is broken
// fn determine_wall_type(x: usize, y: usize, map_lines: &Vec<&str>) -> Position {
//     let y_max = map_lines.len().saturating_sub(1);
//     let x_max = map_lines
//         .get(y)
//         .map_or(0, |line| line.len().saturating_sub(1));

//     let char_at = |dx: isize, dy: isize| {
//         map_lines
//             .get((y as isize).wrapping_add(dy) as usize)
//             .and_then(|line| line.chars().nth((x as isize).wrapping_add(dx) as usize))
//     };

//     let is_wall = |dx: isize, dy: isize| char_at(dx, dy) == Some('#');
//     let is_floor = |dx: isize, dy: isize| char_at(dx, dy) == Some('.');

//     // Pre-compute common conditions to simplify the match arms
//     let around_wall = is_wall(0, -1) && is_wall(0, 1) && is_wall(1, 0) && is_wall(-1, 0);
//     let bottom_left_conditions = is_floor(1, -2)
//         && is_floor(-1, -2)
//         && is_floor(1, 1)
//         && is_floor(-1, 1)
//         && is_floor(0, -2)
//         && is_floor(1, 0)
//         && is_floor(-1, 0);

//     match (
//         around_wall,
//         bottom_left_conditions,
//         is_floor(-1, -2),
//         is_floor(1, -2),
//         is_floor(-1, 1),
//         is_floor(1, 1),
//     ) {
//         (true, false, false, false, true, true) => TILES.wall_ceil,
//         (true, _, true, false, false, _) => TILES.wall_bottom_cross_edge_right,
//         (true, _, false, true, _, false) => TILES.wall_bottom_cross_edge_left,
//         (true, _, _, _, true, _) => TILES.wall_top_cross_edge_right,
//         (true, _, _, _, _, true) => TILES.wall_top_cross_edge_left,
//         (true, _, _, _, false, true) => TILES.wall_right,
//         (true, _, _, _, true, false) => TILES.wall_left,
//         (true, true, ..) => match (is_floor(-1, -2), is_floor(1, -2)) {
//             (true, _) => TILES.wall_bottom_edge_left_top,
//             (_, true) => TILES.wall_bottom_edge_right_top,
//             _ => TILES.wall_bottom_mid_bottom, // Default case if specific conditions aren't met
//         },
//         // Additional consolidated conditions can be placed here
//         _ => TILES.player_walk_down_4,
//     }
// }
#[derive(Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

pub struct Size {
    pub width: f32,
    pub height: f32,
}

pub struct Translation {
    pub position: Position,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Types {
    PLAYER,
    ENVIRONMENT,
}

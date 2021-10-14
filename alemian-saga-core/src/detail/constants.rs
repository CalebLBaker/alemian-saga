use crate::*;
use numeric_types::*;

pub const CURSOR_IMAGE: &str = "cursor.png";
pub const INFO_BAR_IMAGE: &str = "infobar.png";
pub const UNIT_INFO_BAR_IMAGE: &str = "unit-infobar.png";

pub const ZERO_TILES: MapDistance = map_dist(0);
pub const ONE_TILE: MapDistance = map_dist(1);
pub const UNREACHABLE: MapDistance = map_dist(-1);

pub const UP: Vector<MapDistance> = Vector {
    x: ZERO_TILES,
    y: map_dist(-1),
};
pub const DOWN: Vector<MapDistance> = Vector {
    x: ZERO_TILES,
    y: ONE_TILE,
};
pub const LEFT: Vector<MapDistance> = Vector {
    x: map_dist(-1),
    y: ZERO_TILES,
};
pub const RIGHT: Vector<MapDistance> = Vector {
    x: ONE_TILE,
    y: ZERO_TILES,
};

pub const ZERO_HP: HitPoints = hp(0);

pub const BASE_EVADE_BONUS: AccuracyPoints = accuracy_pts(0);

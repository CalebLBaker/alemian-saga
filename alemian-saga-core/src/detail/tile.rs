use crate::*;
use detail::constants::UNREACHABLE;
use detail::unit::Unit;

pub struct TileType<P: Platform> {
    pub name: String,
    pub image: Option<*const P::Image>,
    pub move_cost: numeric_types::MapDistance,
    pub defense: numeric_types::HitPoints,
    pub evade: numeric_types::AccuracyPoints
}

// Represents a tile in the map
pub struct Tile<P: Platform> {
    pub info: *const TileType<P>,
    pub unit: Option<Unit>,
    pub remaining_move: numeric_types::MapDistance,
}

pub fn make_tile<P: Platform>(
    info: *const TileType<P>,
) -> Tile<P> {
    Tile {
        info,
        unit: None,
        remaining_move: UNREACHABLE,
    }
}

pub fn get_tile<P: Platform>(
    tile_types: &[TileType<P>],
    type_id: usize,
) -> Option<Tile<P>> {
    Some(make_tile::<P>(tile_types.get(type_id)?))
}

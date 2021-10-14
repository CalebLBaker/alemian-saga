use crate::*;
use detail::constants::UNREACHABLE;

// Represents a tile in the map
pub struct Tile<'a, P: Platform> {
    pub image: Option<&'a P::Image>,
    pub info: &'a serialization::TileType<'a>,
    pub unit: std::cell::Cell<Option<&'a serialization::Unit<'a>>>,
    pub remaining_move: std::cell::Cell<numeric_types::MapDistance>,
}

pub fn make_tile<'a, P: Platform>(
    image: Option<&'a P::Image>,
    info: &'a serialization::TileType<'a>,
) -> Tile<'a, P> {
    Tile {
        image,
        info,
        unit: std::cell::Cell::new(None),
        remaining_move: std::cell::Cell::new(UNREACHABLE),
    }
}

pub fn get_tile<'a, P: Platform>(
    image_map: &'a std::collections::HashMap<&str, P::Image>,
    tile_types: &'a [serialization::TileType],
    type_id: usize,
) -> Option<Tile<'a, P>> {
    let tile_type = tile_types.get(type_id)?;
    Some(make_tile(image_map.get(tile_type.image), tile_type))
}

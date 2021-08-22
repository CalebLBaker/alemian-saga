use crate::*;

// Represents a tile in the map
pub struct Tile<'a, P: Platform> {
    pub image: Option<&'a P::Image>,
    pub info: &'a serialization::TileType<'a>,
    pub unit: Option<&'a serialization::Unit<'a>>
}

pub fn get_tile<'a, P: Platform>(
    image_map: &'a std::collections::HashMap<&str, P::Image>,
    tile_types: &'a [serialization::TileType],
    type_id: usize,
) -> Option<Tile<'a, P>> {
    let tile_type = tile_types.get(type_id)?;
    Some(Tile {
        image: image_map.get(tile_type.image),
        info: &tile_type,
        unit: None
    })
}


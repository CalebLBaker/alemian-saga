use crate::*;
pub use numeric_types::*;

// Serialized format for metadata about a particular type of tile
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TileType<'a> {
    pub image: &'a str,
    pub name: &'a str,
    pub defense: HitPoints,
    pub evade: AccuracyPoints,
    pub move_cost: MapDistance,
}

// Serialized format for maps
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Map<'a> {
    #[serde(borrow)]
    pub tile_types: Vec<TileType<'a>>,
    pub map: ndarray::Array2<u32>,
    pub blue: Vec<Unit<'a>>,
}

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr,
)]
#[repr(u8)]
pub enum Class {
    Noble,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Unit<'a> {
    pub name: &'a str,
    pub class: Class,
    pub level: Level,
    pub hp: HitPoints,
    pub movement: MapDistance,
    pub remaining_move: MapDistance,
    pub position: Vector<MapDistance>,
}

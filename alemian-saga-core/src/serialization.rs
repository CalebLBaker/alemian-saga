use crate::Vector;

pub type MapDistance = u32;

// Serialized format for metadata about a particular type of tile
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TileType {
    pub image: String,
    pub name: String,
    pub defense: i32,
    pub evade: i32,
    pub move_cost: u32,
}

// Serialized format for maps
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Map {
    pub tile_types: Vec<TileType>,
    pub map: ndarray::Array2<u32>,
    pub blue: Vec<Unit>
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum Class {
    Noble
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Unit {
    pub class: Class,
    pub position: Vector<MapDistance>
}


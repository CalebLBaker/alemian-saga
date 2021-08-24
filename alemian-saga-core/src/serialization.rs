use crate::Vector;

pub type MapDistance = u32;

// Serialized format for metadata about a particular type of tile
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TileType<'a> {
    pub image: &'a str,
    pub name: &'a str,
    pub defense: i32,
    pub evade: i32,
    pub move_cost: u32,
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
    pub level: u32,
    pub hp: u32,
    pub position: Vector<MapDistance>,
}

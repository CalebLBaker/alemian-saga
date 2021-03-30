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
}

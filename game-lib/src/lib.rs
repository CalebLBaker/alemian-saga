#![cfg_attr(feature = "strict", deny(warnings))]

use async_trait::async_trait;

// Trait used for abstracting away logic that is specific to a particular platform
#[async_trait(?Send)]
pub trait Platform {
    // Type used to represent images
    type Image;

    // Future type returned by get_image
    type ImageFuture: std::future::Future<Output = Option<Self::Image>>;

    // Type used to represent files
    type File: std::io::Read;

    // Draw an image to the screen
    fn draw(&self, img: &Self::Image, left: f64, top: f64, width: f64, height: f64);

    // Get the width of the game screen
    fn get_width(&self) -> f64;

    // Get the height of the game screen
    fn get_height(&self) -> f64;

    // Retrieve an image from a specified file path
    fn get_image(path: &str) -> Self::ImageFuture;

    // Retrieve a file from a specified file path
    async fn get_file(&self, path: &str) -> Result<Self::File, String>;

    // Log a message (typically to stdout or the equivalent)
    fn log(path: &str);
}

// Serialized format for metadata about a particular type of tile
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TileType {
    pub image: String,
}

// Serialized format for maps
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Map {
    pub tile_types: Vec<TileType>,
    pub map: ndarray::Array2<u32>,
}

// Entry point for starting game logic
pub async fn run<P: Platform>(platform: P) {
    if let Err(e) = run_internal(&platform).await {
        P::log(e.msg.as_str());
    }
}

// Represents a tile in the map
struct Tile<'a, P: Platform> {
    image: Option<&'a P::Image>,
}

// Retrieves a reference to an image using a tile type id and the image_map and tile_types data
// structures
fn get_image<'a, P: Platform>(
    image_map: &'a std::collections::HashMap<&str, P::Image>,
    tile_types: &Vec<TileType>,
    type_id: usize,
) -> Option<&'a P::Image> {
    image_map.get(tile_types.get(type_id)?.image.as_str())
}

// Error message type
struct Error {
    msg: String,
}

// Conversion from rmp serde errors to Error
impl From<rmp_serde::decode::Error> for Error {
    fn from(err: rmp_serde::decode::Error) -> Error {
        Error {
            msg: err.to_string(),
        }
    }
}

// Conversion from String to Error
impl From<String> for Error {
    fn from(msg: String) -> Error {
        Error { msg }
    }
}

// Main function containing all of the game logic
async fn run_internal<P: Platform>(platform: &P) -> Result<(), Error> {
    // Retrieve map file
    let map_file: Map = rmp_serde::decode::from_read(platform.get_file("map.map").await?)?;

    // Get a map from image paths to images
    let mut image_map = std::collections::HashMap::new();
    let images = map_file.tile_types.iter().map(|x| {
        let image_str = x.image.as_str();
        (image_str, P::get_image(image_str))
    });
    for (n, f) in images.collect::<Vec<_>>().into_iter() {
        if let Some(image) = f.await {
            image_map.insert(n, image);
        }
    }

    // Generate the map
    let map = map_file.map.map(|i| Tile::<P> {
        image: get_image::<P>(&image_map, &map_file.tile_types, *i as usize),
    });

    // Render the map
    let (rows, columns) = map.dim();
    let tile_width = platform.get_width() / columns as f64;
    let tile_height = platform.get_height() / rows as f64;
    for ((r, c), t) in map.indexed_iter() {
        if let Some(i) = t.image {
            let x = c as f64 * tile_width;
            platform.draw(i, x, r as f64 * tile_height, tile_width, tile_height);
        }
    }

    Ok(())
}

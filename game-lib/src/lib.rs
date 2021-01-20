#![cfg_attr(feature = "strict", deny(warnings))]

use async_trait::async_trait;
use futures::StreamExt;

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

    // Attempt to draw an image
    fn attempt_draw(
        &self,
        img: Option<&Self::Image>,
        left: f64,
        top: f64,
        width: f64,
        height: f64,
    ) {
        if let Some(i) = img {
            self.draw(i, left, top, width, height);
        }
    }
}

// Type used to represent user input events
#[derive(Clone, Copy)]
pub enum Event {
    Right,
    Left,
    Up,
    Down,
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
pub async fn run<P: Platform>(
    platform: P,
    mut event_queue: futures::channel::mpsc::Receiver<Event>,
) {
    if let Err(e) = run_internal(platform, &mut event_queue).await {
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

// Conversion into error type
impl<E: std::string::ToString> From<E> for Error {
    fn from(err: E) -> Error {
        Error {
            msg: err.to_string(),
        }
    }
}

// Struct for holding game state
struct Game<'a, P: Platform> {
    platform: P,
    cursor_row: usize,
    cursor_column: usize,
    map: ndarray::Array2<Tile<'a, P>>,
    cursor_image: Option<P::Image>,
}

impl<'a, P: Platform> Game<'a, P> {
    fn move_cursor(&mut self, row: usize, column: usize) {
        let (num_rows, num_columns) = self.map.dim();
        let tile_width = self.platform.get_width() / num_columns as f64;
        let tile_height = self.platform.get_height() / num_rows as f64;
        let old_row = self.cursor_row;
        let old_column = self.cursor_column;
        let old_x = old_column as f64 * tile_width;
        let old_y = old_row as f64 * tile_height;
        let image = self.map[[old_row, old_column]].image;
        self.platform
            .attempt_draw(image, old_x, old_y, tile_width, tile_height);
        self.cursor_row = row;
        self.cursor_column = column;
        let x = column as f64 * tile_width;
        let y = row as f64 * tile_height;
        self.platform
            .attempt_draw(self.cursor_image.as_ref(), x, y, tile_width, tile_height);
    }
}

// Main function containing all of the game logic
async fn run_internal<P: Platform>(
    platform: P,
    event_queue: &mut futures::channel::mpsc::Receiver<Event>,
) -> Result<(), Error> {
    // Retrieve map file
    let map_file: Map = rmp_serde::decode::from_read(platform.get_file("map.map").await?)?;

    // Create map from image paths to images
    let mut image_map = std::collections::HashMap::new();
    let images = map_file.tile_types.iter().map(|x| {
        let image_str = x.image.as_str();
        (image_str, P::get_image(image_str))
    });
    let cursor_future = P::get_image("cursor.png");
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
        let x = c as f64 * tile_width;
        platform.attempt_draw(t.image, x, r as f64 * tile_height, tile_width, tile_height);
    }

    let cursor_image = cursor_future.await;
    platform.attempt_draw(cursor_image.as_ref(), 0.0, 0.0, tile_width, tile_height);

    let mut game = Game {
        platform,
        cursor_row: 0,
        cursor_column: 0,
        map,
        cursor_image,
    };

    let last_column = columns - 1;
    let last_row = rows - 1;

    while let Some(e) = event_queue.next().await {
        match e {
            Event::Right => {
                if game.cursor_column < last_column {
                    game.move_cursor(game.cursor_row, game.cursor_column + 1);
                }
            }
            Event::Left => {
                if game.cursor_column > 0 {
                    game.move_cursor(game.cursor_row, game.cursor_column - 1);
                }
            }
            Event::Up => {
                if game.cursor_row > 0 {
                    game.move_cursor(game.cursor_row - 1, game.cursor_column);
                }
            }
            Event::Down => {
                if game.cursor_row < last_row {
                    game.move_cursor(game.cursor_row + 1, game.cursor_column);
                }
            }
        }
    }
    P::log("closing");

    Ok(())
}

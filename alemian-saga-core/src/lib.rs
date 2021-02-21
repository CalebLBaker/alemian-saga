#![cfg_attr(feature = "strict", deny(warnings))]

use std::ops;

use async_trait::async_trait;
use futures::channel::mpsc;
use futures::StreamExt;

const KEYBINDINGS_PATH: &str = "keybindings/us.json";
const MAP_FILE: &str = "map.map";
const CURSOR_IMAGE: &str = "cursor.png";

pub trait Scalar: ops::Div<Output = Self> + ops::Mul<Output = Self> + Copy {}
impl<T> Scalar for T where T: ops::Div<Output = T> + ops::Mul<Output = Self> + Copy {}

// Represents a vector
#[derive(Clone, Copy)]
pub struct Vector<T> {
    pub x: T,
    pub y: T,
}

impl<T: Scalar + num_traits::ToPrimitive> Vector<T> {
    fn lossy_cast<U: num_traits::NumCast>(self) -> Option<Vector<U>> {
        Some(Vector {
            x: U::from(self.x)?,
            y: U::from(self.y)?,
        })
    }
}

impl<T: Scalar> Vector<T> {
    fn piecewise_divde<U: Scalar + Into<T>>(self, rhs: Vector<U>) -> Vector<T> {
        Vector {
            x: self.x / rhs.x.into(),
            y: self.y / rhs.y.into(),
        }
    }
    fn piecewise_multiply<U: Scalar + Into<T>>(self, rhs: Vector<U>) -> Vector<T> {
        Vector {
            x: self.x * rhs.x.into(),
            y: self.y * rhs.y.into(),
        }
    }
    fn cast<U: Scalar + From<T>>(self) -> Vector<U> {
        Vector {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}

// Represents a rectangle
pub struct Rectangle<T> {
    top_left: Vector<T>,
    size: Vector<T>,
}

impl<T: Scalar> Rectangle<T> {
    fn top(&self) -> T {
        self.top_left.y
    }
    fn left(&self) -> T {
        self.top_left.x
    }
    fn width(&self) -> T {
        self.size.x
    }
    fn height(&self) -> T {
        self.size.y
    }
}

// Trait used for abstracting away logic that is specific to a particular platform
#[async_trait(?Send)]
pub trait Platform {
    // Type used to represent images
    type Image;

    // Type used to represent user input (keyboard or button)
    type InputType: Eq + std::hash::Hash;

    // Type used to represent distance in mouse events (should be the same ScreenDistance
    type MouseDistance: Scalar;

    // Type used to represent distance on the screen
    type ScreenDistance: Scalar + From<u32> + From<Self::MouseDistance> + num_traits::ToPrimitive;

    // Future type returned by get_image
    type ImageFuture: std::future::Future<Output = Option<Self::Image>>;

    // Type used to represent files
    type File: std::io::Read;

    // Draw an image to the screen
    fn draw_primitive(
        &self,
        img: &Self::Image,
        left: Self::ScreenDistance,
        top: Self::ScreenDistance,
        width: Self::ScreenDistance,
        height: Self::ScreenDistance,
    );

    fn string_to_input(input: String) -> Self::InputType;

    // Get the width of the game screen
    fn get_width(&self) -> Self::ScreenDistance;

    // Get the height of the game screen
    fn get_height(&self) -> Self::ScreenDistance;

    // Retrieve an image from a specified file path
    fn get_image(path: &str) -> Self::ImageFuture;

    // Retrieve a file from a specified file path
    async fn get_file(&self, path: &str) -> Result<Self::File, String>;

    // Log a message (typically to stdout or the equivalent)
    fn log(path: &str);

    fn get_screen_size(&self) -> Vector<Self::ScreenDistance> {
        Vector {
            x: self.get_width(),
            y: self.get_height(),
        }
    }

    // Draw an image to the screen
    fn draw(&self, img: &Self::Image, location: &Rectangle<Self::ScreenDistance>) {
        let left = location.left();
        self.draw_primitive(
            img,
            left,
            location.top(),
            location.width(),
            location.height(),
        );
    }

    // Attempt to draw an image
    fn attempt_draw(&self, img: Option<&Self::Image>, location: &Rectangle<Self::ScreenDistance>) {
        if let Some(i) = img {
            self.draw(i, location);
        }
    }

    async fn get_keybindings(
        &self,
    ) -> Option<std::collections::HashMap<Self::InputType, Event<Self::MouseDistance>>> {
        let mut ret = std::collections::HashMap::new();
        let bindings_file = self.get_file(KEYBINDINGS_PATH).await.ok()?;
        let bindings: Keybindings = serde_json::from_reader(bindings_file).ok()?;
        for k in bindings.right.into_iter() {
            ret.insert(Self::string_to_input(k), Event::Right);
        }
        for k in bindings.left.into_iter() {
            ret.insert(Self::string_to_input(k), Event::Left);
        }
        for k in bindings.up.into_iter() {
            ret.insert(Self::string_to_input(k), Event::Up);
        }
        for k in bindings.down.into_iter() {
            ret.insert(Self::string_to_input(k), Event::Down);
        }
        Some(ret)
    }
}

// Type used to represent user input events
#[derive(Clone, Copy)]
pub enum Event<P: Scalar> {
    Right,
    Left,
    Up,
    Down,
    MouseMove(Vector<P>),
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
    mut event_queue: mpsc::Receiver<Event<P::MouseDistance>>,
) {
    if let Err(e) = run_internal(platform, &mut event_queue).await {
        P::log(e.msg.as_str());
    }
}

#[derive(serde::Deserialize)]
struct Keybindings {
    right: Vec<String>,
    left: Vec<String>,
    up: Vec<String>,
    down: Vec<String>,
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

type MapDistance = u32;

// Struct for holding game state
struct Game<'a, P: Platform> {
    platform: P,
    cursor_pos: Vector<MapDistance>,
    map: ndarray::Array2<Tile<'a, P>>,
    cursor_image: Option<P::Image>,
}

impl<'a, P: Platform> Game<'a, P> {
    fn get_tile_size(&self) -> Vector<P::ScreenDistance> {
        let (num_rows, num_columns) = self.map.dim();
        let map_dims = Vector {
            x: num_columns as MapDistance,
            y: num_rows as MapDistance,
        };
        self.platform.get_screen_size().piecewise_divde(map_dims)
    }

    fn move_cursor(&mut self, pos: Vector<MapDistance>) {
        let tile_size = self.get_tile_size();
        let old_pos = self.cursor_pos;
        let old_screen_pos = tile_size.piecewise_multiply(old_pos);
        let image = self.map[[old_pos.y as usize, old_pos.x as usize]].image;
        self.platform.attempt_draw(
            image,
            &Rectangle {
                top_left: old_screen_pos,
                size: tile_size,
            },
        );
        self.cursor_pos = pos;
        let screen_pos = Rectangle {
            top_left: tile_size.piecewise_multiply(pos),
            size: tile_size,
        };
        self.platform
            .attempt_draw(self.cursor_image.as_ref(), &screen_pos);
    }
}

// Main function containing all of the game logic
async fn run_internal<P: Platform>(
    platform: P,
    event_queue: &mut mpsc::Receiver<Event<P::MouseDistance>>,
) -> Result<(), Error> {
    // Retrieve map file
    let map_file_future = platform.get_file(MAP_FILE);
    let cursor_future = P::get_image(CURSOR_IMAGE);
    let map_file: Map = rmp_serde::decode::from_read(map_file_future.await?)?;

    // Create map from image paths to images
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
    let map_size = Vector {
        x: columns as MapDistance,
        y: rows as MapDistance,
    };
    let tile_size = platform.get_screen_size().piecewise_divde(map_size);
    for ((r, c), t) in map.indexed_iter() {
        let map_pos = Vector {
            x: c as MapDistance,
            y: r as MapDistance,
        };
        let map_rect = Rectangle {
            top_left: tile_size.piecewise_multiply(map_pos),
            size: tile_size,
        };
        platform.attempt_draw(t.image, &map_rect);
    }

    let cursor_image = cursor_future.await;
    let cursor_pos = Rectangle {
        top_left: Vector {
            x: 0.into(),
            y: 0.into(),
        },
        size: tile_size,
    };
    platform.attempt_draw(cursor_image.as_ref(), &cursor_pos);

    let mut game = Game {
        platform,
        cursor_pos: Vector { x: 0, y: 0 },
        map,
        cursor_image,
    };

    let last_column = map_size.x - 1;
    let last_row = map_size.y - 1;

    while let Some(e) = event_queue.next().await {
        match e {
            Event::Right => {
                if game.cursor_pos.x < last_column {
                    game.move_cursor(Vector {
                        x: game.cursor_pos.x + 1,
                        y: game.cursor_pos.y,
                    });
                }
            }
            Event::Left => {
                if game.cursor_pos.x > 0 {
                    game.move_cursor(Vector {
                        x: game.cursor_pos.x - 1,
                        y: game.cursor_pos.y,
                    });
                }
            }
            Event::Up => {
                if game.cursor_pos.y > 0 {
                    game.move_cursor(Vector {
                        x: game.cursor_pos.x,
                        y: game.cursor_pos.y - 1,
                    });
                }
            }
            Event::Down => {
                if game.cursor_pos.y < last_row {
                    game.move_cursor(Vector {
                        x: game.cursor_pos.x,
                        y: game.cursor_pos.y + 1,
                    });
                }
            }
            Event::MouseMove(mouse_pos) => {
                let screen_pos = mouse_pos.cast::<P::ScreenDistance>();
                let cursor_pos = screen_pos.piecewise_divde(game.get_tile_size());
                if let Some(p) = cursor_pos.lossy_cast::<MapDistance>() {
                    if p.x <= last_column && p.y <= last_row {
                        game.move_cursor(p);
                    }
                }
            }
        }
    }
    P::log("closing");

    Ok(())
}

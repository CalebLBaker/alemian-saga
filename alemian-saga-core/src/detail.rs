use futures::channel::mpsc;
use futures::StreamExt;
use ndarray::prelude::*;
use num_traits::FromPrimitive;

use crate::{serialization, Event, Platform, Scalar, Vector};

const CURSOR_IMAGE: &str = "cursor.png";
const INFO_BAR_IMAGE: &str = "infobar.png";

impl<T: Scalar + num_traits::ToPrimitive> Vector<T> {
    fn lossy_cast<U: num_traits::NumCast>(self) -> Option<Vector<U>> {
        Some(Vector {
            x: U::from(self.x)?,
            y: U::from(self.y)?,
        })
    }
}

impl<T: Scalar> Vector<T> {
    fn piecewise_divide<U: Scalar + Into<T>>(self, rhs: Vector<U>) -> Vector<T> {
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

impl<T: std::ops::Add<Output = T>> std::ops::Add for Vector<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: std::ops::Sub<Output = T>> std::ops::Sub for Vector<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: std::ops::Div<Output = T> + Copy> std::ops::Div<T> for Vector<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

// Represents a rectangle
pub struct Rectangle<T> {
    top_left: Vector<T>,
    size: Vector<T>,
}

impl<T: Scalar> Rectangle<T> {
    pub fn top(&self) -> T {
        self.top_left.y
    }
    pub fn left(&self) -> T {
        self.top_left.x
    }
    pub fn width(&self) -> T {
        self.size.x
    }
    pub fn height(&self) -> T {
        self.size.y
    }
    fn bottom(&self) -> T {
        self.top() + self.height()
    }
    fn right(&self) -> T {
        self.left() + self.width()
    }
}

#[derive(serde::Deserialize)]
#[allow(non_snake_case)]
pub struct Keybindings {
    #[serde(default)]
    pub Right: Vec<String>,
    #[serde(default)]
    pub Left: Vec<String>,
    #[serde(default)]
    pub Up: Vec<String>,
    #[serde(default)]
    pub Down: Vec<String>,
    #[serde(default)]
    pub ZoomIn: Vec<String>,
    #[serde(default)]
    pub ZoomOut: Vec<String>,
}

// Represents a tile in the map
struct Tile<'a, P: Platform> {
    image: Option<&'a P::Image>,
    info: &'a serialization::TileType,
}

fn get_tile<'a, P: Platform>(
    image_map: &'a std::collections::HashMap<&str, P::Image>,
    tile_types: &'a Vec<serialization::TileType>,
    type_id: usize,
) -> Option<Tile<'a, P>> {
    let tile_type = tile_types.get(type_id)?;
    Some(Tile {
        image: image_map.get(tile_type.image.as_str()),
        info: &tile_type,
    })
}

// Error message type
pub struct Error {
    pub msg: String,
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
    infobar_image: Option<P::Image>,
    screen: Rectangle<MapDistance>,
    last_mouse_pan: P::Instant,
}

fn multiply_frac<T: Scalar + From<u32>>(x: T, num: u32, den: u32) -> T {
    x * num.into() / den.into()
}

impl<'a, P: Platform> Game<'a, P> {
    fn get_tile_size(&self) -> Vector<P::ScreenDistance> {
        self.platform
            .get_screen_size()
            .piecewise_divide(self.screen.size)
    }

    fn get_tile(&self, pos: Vector<MapDistance>) -> &Tile<'a, P> {
        return &self.map[[pos.y as usize, pos.x as usize]];
    }

    fn get_screen_pos(&self, pos: Vector<MapDistance>) -> Rectangle<P::ScreenDistance> {
        let tile_size = self.get_tile_size();
        Rectangle {
            top_left: tile_size.piecewise_multiply(pos - self.screen.top_left),
            size: tile_size,
        }
    }

    fn get_map_size(&self) -> Vector<MapDistance> {
        let (rows, columns) = self.map.dim();
        Vector {
            x: columns as MapDistance,
            y: rows as MapDistance,
        }
    }

    fn get_map_pos(&self, pos: Vector<P::MouseDistance>) -> Option<Vector<MapDistance>> {
        let screen_pos = pos.cast::<P::ScreenDistance>();
        let pos_on_screen = screen_pos.piecewise_divide(self.get_tile_size());
        Some(pos_on_screen.lossy_cast::<MapDistance>()? + self.screen.top_left)
    }

    fn move_cursor(&mut self, pos: Vector<MapDistance>) {
        let old_pos = self.cursor_pos;
        self.platform
            .attempt_draw(self.get_tile(old_pos).image, &self.get_screen_pos(old_pos));
        self.cursor_pos = pos;
        self.draw_cursor();
        self.draw_infobar();
    }

    fn draw_cursor(&self) {
        let cursor_pos_on_screen = self.get_screen_pos(self.cursor_pos);
        self.platform
            .attempt_draw(self.cursor_image.as_ref(), &cursor_pos_on_screen);
    }

    fn draw_infobar(&self) {
        let height = self.platform.get_height() / 15.into();
        let size = Vector {
            x: height * 4.into(),
            y: height,
        };
        let position = Rectangle {
            top_left: Vector {
                x: 0.into(),
                y: 0.into(),
            },
            size,
        };
        self.platform
            .attempt_draw(self.infobar_image.as_ref(), &position);
        let offset_scalar = size.y / 4.into();
        let offset = Vector {
            x: offset_scalar,
            y: offset_scalar,
        };
        let max_width = size.x * P::ScreenDistance::from_f64(0.75).unwrap_or(1.into());
        let tile = self.get_tile(self.cursor_pos);
        let stat_y = multiply_frac(height, 5, 8);
        let info = &tile.info;
        self.platform
            .draw_text(info.name.as_str(), offset, max_width);
        let stat_width = height * 13.into() / 16.into();
        let move_pos = Vector {
            x: multiply_frac(height, 3, 4),
            y: stat_y,
        };
        let defense_pos = Vector {
            x: multiply_frac(height, 15, 8),
            y: stat_y,
        };
        let evade_pos = Vector {
            x: height * 3.into(),
            y: stat_y,
        };
        self.platform
            .draw_text(info.move_cost.to_string().as_str(), move_pos, stat_width);
        self.platform
            .draw_text(info.defense.to_string().as_str(), defense_pos, stat_width);
        self.platform
            .draw_text(info.evade.to_string().as_str(), evade_pos, stat_width);
    }

    fn redraw(&self) {
        let top_left = self.screen.top_left;
        let top_left_index = top_left.lossy_cast::<usize>().expect("Failed cast");
        let bottom_right_option = (top_left + self.screen.size).lossy_cast::<usize>();
        let bottom_right = bottom_right_option.expect("Failed cast");
        let slice_helper = s![
            top_left_index.y..bottom_right.y,
            top_left_index.x..bottom_right.x
        ];
        for ((r, c), t) in self.map.slice(slice_helper).indexed_iter() {
            let map_pos = Vector {
                x: c as MapDistance,
                y: r as MapDistance,
            } + top_left;
            self.platform
                .attempt_draw(t.image, &self.get_screen_pos(map_pos));
        }
        self.draw_cursor();
        self.draw_infobar();
    }
}

fn partial_ord_min<T: std::cmp::PartialOrd>(a: T, b: T) -> T {
    if b < a {
        b
    } else {
        a
    }
}

// Main function containing all of the game logic
pub async fn run_internal<P: Platform>(
    platform: P,
    event_queue: &mut mpsc::Receiver<Event<P::MouseDistance>>,
    language: &str,
) -> Result<(), Error> {
    let last_mouse_pan = P::now();

    let error_tile = serialization::TileType {
        image: "".to_owned(),
        name: "ERROR".to_owned(),
        defense: 0,
        evade: 0,
        move_cost: 1,
    };

    // Retrieve map file
    let map_path = format!("{}/map.map", language);
    let map_file_future = platform.get_file(map_path.as_str());
    let cursor_future = P::get_image(CURSOR_IMAGE);
    let info_future = P::get_image(INFO_BAR_IMAGE);
    let map_file: serialization::Map = rmp_serde::decode::from_read(map_file_future.await?)?;

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
    let map = map_file.map.map(|i| {
        let tile = get_tile::<P>(&image_map, &map_file.tile_types, *i as usize);
        tile.unwrap_or_else(|| {
            P::log("Error: Invalid map file");
            Tile {
                image: None,
                info: &error_tile,
            }
        })
    });

    let (rows, columns) = map.dim();
    let map_size = Vector {
        x: columns as MapDistance,
        y: rows as MapDistance,
    };

    let mut game = Game {
        platform,
        cursor_pos: Vector { x: 0, y: 0 },
        map,
        cursor_image: cursor_future.await,
        infobar_image: info_future.await,
        screen: Rectangle {
            top_left: Vector { x: 0, y: 0 },
            size: map_size,
        },
        last_mouse_pan,
    };

    game.redraw();

    let last_column = map_size.x - 1;
    let last_row = map_size.y - 1;
    let mouse_pan_delay = P::nanoseconds(100000000);

    while let Some(e) = event_queue.next().await {
        match e {
            Event::Right => {
                if game.cursor_pos.x < last_column {
                    if game.cursor_pos.x == game.screen.right() - 1 {
                        game.cursor_pos.x += 1;
                        game.screen.top_left.x += 1;
                        game.redraw();
                    } else {
                        game.move_cursor(Vector {
                            x: game.cursor_pos.x + 1,
                            y: game.cursor_pos.y,
                        });
                    }
                }
            }
            Event::Left => {
                if game.cursor_pos.x > 0 {
                    if game.cursor_pos.x == game.screen.left() {
                        game.cursor_pos.x -= 1;
                        game.screen.top_left.x -= 1;
                        game.redraw();
                    } else {
                        game.move_cursor(Vector {
                            x: game.cursor_pos.x - 1,
                            y: game.cursor_pos.y,
                        });
                    }
                }
            }
            Event::Up => {
                if game.cursor_pos.y > 0 {
                    if game.cursor_pos.y == game.screen.top() {
                        game.cursor_pos.y -= 1;
                        game.screen.top_left.y -= 1;
                        game.redraw();
                    } else {
                        game.move_cursor(Vector {
                            x: game.cursor_pos.x,
                            y: game.cursor_pos.y - 1,
                        });
                    }
                }
            }
            Event::Down => {
                if game.cursor_pos.y < last_row {
                    if game.cursor_pos.y == game.screen.bottom() - 1 {
                        game.cursor_pos.y += 1;
                        game.screen.top_left.y += 1;
                        game.redraw();
                    } else {
                        game.move_cursor(Vector {
                            x: game.cursor_pos.x,
                            y: game.cursor_pos.y + 1,
                        });
                    }
                }
            }
            Event::ZoomIn => {
                let tile_size = game.get_tile_size();
                let size = &mut game.screen.size;
                let cursor_pos_on_screen = game.cursor_pos - game.screen.top_left;
                if tile_size.x >= tile_size.y && size.y > 1 {
                    size.y -= 1;
                    if cursor_pos_on_screen.y > size.y / 2 {
                        game.screen.top_left.y += 1;
                    }
                }
                if tile_size.y >= tile_size.x && size.x > 1 {
                    size.x -= 1;
                    if cursor_pos_on_screen.x > size.x / 2 {
                        game.screen.top_left.x += 1;
                    }
                }
                game.redraw();
            }
            Event::ZoomOut => {
                let tile_size = game.get_tile_size();
                let map_size = game.get_map_size();
                let cursor_pos_on_screen = game.cursor_pos - game.screen.top_left;
                let size = game.screen.size;
                if size.y < map_size.y && (tile_size.y >= tile_size.x || size.x == map_size.x) {
                    game.screen.size.y += 1;
                    if game.screen.bottom() > map_size.y
                        || game.screen.top() > 0
                            && cursor_pos_on_screen.y < game.screen.height() / 2
                    {
                        game.screen.top_left.y -= 1;
                    }
                }
                if size.x < map_size.x && (tile_size.x >= tile_size.y || size.y == map_size.y) {
                    game.screen.size.x += 1;
                    if game.screen.right() > map_size.x
                        || game.screen.left() > 0 && cursor_pos_on_screen.x < size.x / 2
                    {
                        game.screen.top_left.x -= 1;
                    }
                }
                game.redraw();
            }
            Event::MouseMove(mouse_pos) => {
                let time = P::now();
                let pan = if P::duration_between(game.last_mouse_pan, time) > mouse_pan_delay {
                    let screen_pos = mouse_pos.cast::<P::ScreenDistance>();
                    let half_tile_size = game.get_tile_size() / 2.into();
                    let screen_size = game.platform.get_screen_size();
                    let quarter_screen_size = screen_size / 4.into();
                    let border_size = Vector {
                        x: partial_ord_min(half_tile_size.x, quarter_screen_size.x),
                        y: partial_ord_min(half_tile_size.y, quarter_screen_size.y),
                    };
                    let near_end = screen_size - border_size;
                    let map_size = game.get_map_size();
                    if screen_pos.y < border_size.y && game.screen.top() > 0 {
                        game.screen.top_left.y -= 1;
                        true
                    } else if screen_pos.y > near_end.y && game.screen.bottom() < map_size.y {
                        game.screen.top_left.y += 1;
                        true
                    } else if screen_pos.x < border_size.x && game.screen.left() > 0 {
                        game.screen.top_left.x -= 1;
                        true
                    } else if screen_pos.x > near_end.x && game.screen.right() < map_size.x {
                        game.screen.top_left.x += 1;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if let Some(p) = game.get_map_pos(mouse_pos) {
                    if p.x <= last_column && p.y <= last_row {
                        if pan {
                            game.cursor_pos = p;
                            game.last_mouse_pan = time;
                            game.redraw();
                        } else {
                            game.move_cursor(p);
                        }
                    }
                }
            }
            Event::Redraw => game.redraw(),
        }
    }
    P::log("closing");

    Ok(())
}

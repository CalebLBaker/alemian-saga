use futures::StreamExt;

use crate::*;
use constants::*;
use detail::*;
use numeric_types::*;

// Main function containing all of the game logic
// We use collect to avoid lazy iterator evaluation so that asynchronous tasks can run in parallel
// There is a purpose to it, but clippy doesn't realize that
#[allow(clippy::needless_collect)]
pub async fn run_internal<P: Platform>(
    platform: P,
    event_queue: &mut futures::channel::mpsc::Receiver<Event<P::MouseDistance>>,
    language: &str,
) -> Result<(), utility::Error> {
    let last_mouse_pan = P::now();

    let error_tile = serialization::TileType {
        image: "",
        name: "ERROR",
        defense: ZERO_HP,
        evade: BASE_EVADE_BONUS,
        move_cost: ONE_TILE,
    };

    // Retrieve map file
    let map_path = format!("{}/map.map", language);
    let map_file_future = platform.get_file(map_path.as_str());
    let cursor_future = P::get_image(constants::CURSOR_IMAGE);
    let info_future = P::get_image(constants::INFO_BAR_IMAGE);
    let unit_info_future = P::get_image(constants::UNIT_INFO_BAR_IMAGE);
    let raw_map_file = map_file_future.await?;
    let map_file: serialization::Map = rmp_serde::decode::from_read_ref(&raw_map_file)?;

    // Create map from image paths to images
    let mut image_map = std::collections::HashMap::new();
    let images = map_file.tile_types.iter().map(|x| {
        let image_str = x.image;
        (image_str, P::get_image(image_str))
    });
    let tile_image_futures = images.collect::<Vec<_>>();
    let mut unit_image_futures = std::collections::HashMap::new();
    for u in map_file.blue.iter() {
        unit_image_futures.entry(u.class).or_insert_with(|| {
            P::get_image(format!("blue/{}.png", utility::get_class_name(u.class)).as_str())
        });
    }

    for (n, f) in tile_image_futures.into_iter() {
        if let Some(image) = f.await {
            image_map.insert(n, image);
        }
    }

    // Generate the map
    let mut map = map_file.map.map(|i| {
        let tile = tile::get_tile::<P>(&image_map, &map_file.tile_types, *i as usize);
        tile.unwrap_or_else(|| {
            P::log("Error: Invalid map file");
            tile::make_tile(None, &error_tile)
        })
    });

    let (rows, columns) = map.dim();
    let map_size = Vector {
        x: map_dist(columns as i32),
        y: map_dist(rows as i32),
    };

    let mut game = Game::new(
        platform,
        &mut map,
        cursor_future.await,
        info_future.await,
        unit_info_future.await,
        last_mouse_pan,
    );

    for (c, f) in unit_image_futures.into_iter() {
        if let Some(image) = f.await {
            game.unit_images.insert(c, image);
        }
    }

    for u in map_file.blue.iter() {
        game.add_unit(u);
    }

    game.redraw();

    let last_column = map_size.x - ONE_TILE;
    let last_row = map_size.y - ONE_TILE;
    let mouse_pan_delay = P::nanoseconds(100000000);

    while let Some(e) = event_queue.next().await {
        match e {
            Event::Right => {
                if game.cursor_pos.x < last_column {
                    if game.cursor_pos.x == game.screen.right() - ONE_TILE {
                        game.cursor_pos.x += ONE_TILE;
                        game.screen.top_left.x += ONE_TILE;
                        game.redraw();
                    } else {
                        game.move_cursor(Vector {
                            x: game.cursor_pos.x + ONE_TILE,
                            y: game.cursor_pos.y,
                        });
                    }
                }
            }
            Event::Left => {
                if game.cursor_pos.x > ZERO_TILES {
                    if game.cursor_pos.x == game.screen.left() {
                        game.cursor_pos.x -= ONE_TILE;
                        game.screen.top_left.x -= ONE_TILE;
                        game.redraw();
                    } else {
                        game.move_cursor(Vector {
                            x: game.cursor_pos.x - ONE_TILE,
                            y: game.cursor_pos.y,
                        });
                    }
                }
            }
            Event::Up => {
                if game.cursor_pos.y > ZERO_TILES {
                    if game.cursor_pos.y == game.screen.top() {
                        game.cursor_pos.y -= ONE_TILE;
                        game.screen.top_left.y -= ONE_TILE;
                        game.redraw();
                    } else {
                        game.move_cursor(Vector {
                            x: game.cursor_pos.x,
                            y: game.cursor_pos.y - ONE_TILE,
                        });
                    }
                }
            }
            Event::Down => {
                if game.cursor_pos.y < last_row {
                    if game.cursor_pos.y == game.screen.bottom() - ONE_TILE {
                        game.cursor_pos.y += ONE_TILE;
                        game.screen.top_left.y += ONE_TILE;
                        game.redraw();
                    } else {
                        game.move_cursor(Vector {
                            x: game.cursor_pos.x,
                            y: game.cursor_pos.y + ONE_TILE,
                        });
                    }
                }
            }
            Event::ZoomIn => {
                let tile_size = game.get_tile_size();
                let size = &mut game.screen.size;
                let cursor_pos_on_screen = game.cursor_pos - game.screen.top_left;
                if tile_size.x >= tile_size.y && size.y > ONE_TILE {
                    size.y -= ONE_TILE;
                    if cursor_pos_on_screen.y > size.y / 2 {
                        game.screen.top_left.y += ONE_TILE;
                    }
                }
                if tile_size.y >= tile_size.x && size.x > ONE_TILE {
                    size.x -= ONE_TILE;
                    if cursor_pos_on_screen.x > size.x / 2 {
                        game.screen.top_left.x += ONE_TILE;
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
                    game.screen.size.y += ONE_TILE;
                    if game.screen.bottom() > map_size.y
                        || game.screen.top() > ZERO_TILES
                            && cursor_pos_on_screen.y < game.screen.height() / 2
                    {
                        game.screen.top_left.y -= ONE_TILE;
                    }
                }
                if size.x < map_size.x && (tile_size.x >= tile_size.y || size.y == map_size.y) {
                    game.screen.size.x += ONE_TILE;
                    if game.screen.right() > map_size.x
                        || game.screen.left() > ZERO_TILES && cursor_pos_on_screen.x < size.x / 2
                    {
                        game.screen.top_left.x -= ONE_TILE;
                    }
                }
                game.redraw();
            }
            Event::MouseMove(mouse_pos) => {
                let time = P::now();
                let pan = if P::duration_between(game.last_mouse_pan, time) > mouse_pan_delay {
                    let screen_pos = mouse_pos.cast::<P::ScreenDistance>();
                    let half_tile_size = game.get_tile_size() / P::ScreenDistance::from(2);
                    let screen_size = game.platform.get_screen_size();
                    let quarter_screen_size = screen_size / 4.into();
                    let border_size = Vector {
                        x: utility::partial_ord_min(half_tile_size.x, quarter_screen_size.x),
                        y: utility::partial_ord_min(half_tile_size.y, quarter_screen_size.y),
                    };
                    let near_end = screen_size - border_size;
                    let map_size = game.get_map_size();
                    if screen_pos.y < border_size.y && game.screen.top() > ZERO_TILES {
                        game.screen.top_left.y -= ONE_TILE;
                        true
                    } else if screen_pos.y > near_end.y && game.screen.bottom() < map_size.y {
                        game.screen.top_left.y += ONE_TILE;
                        true
                    } else if screen_pos.x < border_size.x && game.screen.left() > ZERO_TILES {
                        game.screen.top_left.x -= ONE_TILE;
                        true
                    } else if screen_pos.x > near_end.x && game.screen.right() < map_size.x {
                        game.screen.top_left.x += ONE_TILE;
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
            Event::Select => {
                game.select_tile();
            }
        }
    }
    P::log("closing");

    Ok(())
}

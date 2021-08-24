use futures::StreamExt;

use crate::*;
use detail::*;

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
        defense: 0,
        evade: 0,
        move_cost: 1,
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
    let map = map_file.map.map(|i| {
        let tile = tile::get_tile::<P>(&image_map, &map_file.tile_types, *i as usize);
        tile.unwrap_or_else(|| {
            P::log("Error: Invalid map file");
            Tile {
                image: None,
                info: &error_tile,
                unit: None,
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
        unit_infobar: unit_info_future.await,
        screen: Rectangle {
            top_left: Vector { x: 0, y: 0 },
            size: map_size,
        },
        last_mouse_pan,
        unit_images: std::collections::HashMap::new(),
    };

    for (c, f) in unit_image_futures.into_iter() {
        if let Some(image) = f.await {
            game.unit_images.insert(c, image);
        }
    }

    for u in map_file.blue.iter() {
        if let Some(t) = game
            .map
            .get_mut((u.position.y as usize, u.position.x as usize))
        {
            t.unit = Some(u)
        }
    }

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
                        x: utility::partial_ord_min(half_tile_size.x, quarter_screen_size.x),
                        y: utility::partial_ord_min(half_tile_size.y, quarter_screen_size.y),
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

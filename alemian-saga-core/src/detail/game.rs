use crate::*;
use constants::*;
use detail::*;

// Struct for holding game state
pub struct Game<P: Platform> {
    platform: P,
    cursor_pos: Vector<MapDistance>,
    screen: Rectangle<MapDistance>,
    last_mouse_pan: P::Instant,
    unit_images: std::collections::HashMap<serialization::Class, P::Image>,
    map: ndarray::Array2<Tile<P>>,
    cursor_image: Option<P::Image>,
    infobar_image: Option<P::Image>,
    unit_infobar: Option<P::Image>,
    highlighted_tiles: Vec<*const Tile<P>>,
    image_map: std::collections::HashMap<String, P::Image>
}

fn get_map_tile<P: Platform>(
    map: &ndarray::Array2<Tile<P>>,
    pos: Vector<MapDistance>,
) -> &Tile<P> {
    &map[[pos.y.value as usize, pos.x.value as usize]]
}

impl<P: Platform> Game<P> {
    pub async fn new(platform: P, language: &str) -> Result<Game<P>, utility::Error> {
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
            (x.image, P::get_image(x.image))
        });
        let tile_image_futures = images.collect::<Vec<_>>();

        let mut unit_image_futures = std::collections::HashMap::new();
        for u in map_file.blue.iter() {
            unit_image_futures.entry(u.class).or_insert_with(|| {
                P::get_image(format!("blue/{}.png", utility::get_class_name(u.class)).as_str())
            });
        }

        let mut tile_images = Vec::with_capacity(tile_image_futures.len());
        for (n, f) in tile_image_futures.into_iter() {
            if let Some(image) = f.await {
                tile_images.push(n);
                image_map.insert(n.to_owned(), image);
            }
        }
        let mut image_iter = tile_images.into_iter();

        let mut tile_types : Vec<tile::TileType<P>> = map_file.tile_types.into_iter().map(|x| {
            tile::TileType{
                name: x.name.to_owned(),
                image: image_map.get(image_iter.next().unwrap()).map(|i| i as *const P::Image),
                move_cost: x.move_cost,
                defense: x.defense,
                evade: x.evade
            }
        }).collect();

        tile_types.push(tile::TileType { name: "ERROR".to_owned(), image: None, move_cost: ONE_TILE, defense: ZERO_HP, evade: BASE_EVADE_BONUS });

        // Generate the map
        let map = map_file.map.map(|i| {
            let tile = tile::get_tile::<P>(&tile_types, *i as usize);
            tile.unwrap_or_else(|| {
                P::log("Error: Invalid map file");
                tile::make_tile(tile_types.last().unwrap())
            })
        });

        let (rows, columns) = map.dim();
        let mut ret = Self {
            platform,
            cursor_pos: Vector {
                x: ZERO_TILES,
                y: ZERO_TILES,
            },
            screen: Rectangle {
                top_left: Vector {
                    x: ZERO_TILES,
                    y: ZERO_TILES,
                },
                size: Vector {
                    x: numeric_types::map_dist(columns as i32),
                    y: numeric_types::map_dist(rows as i32),
                },
            },
            last_mouse_pan,
            unit_images: std::collections::HashMap::new(),
            map,
            cursor_image: cursor_future.await,
            infobar_image: info_future.await,
            unit_infobar: unit_info_future.await,
            highlighted_tiles: Vec::new(),
            image_map
        };

        for (c, f) in unit_image_futures.into_iter() {
            if let Some(image) = f.await {
                ret.unit_images.insert(c, image);
            }
        }

        for u in map_file.blue.into_iter() {
            ret.add_unit(u);
        }

        Ok(ret)
    }

    fn get_tile_size(&self) -> Vector<P::ScreenDistance> {
        self.platform
            .get_screen_size()
            .piecewise_divide(self.screen.size.lossy_cast::<P::ScreenDistance>().unwrap())
    }

    fn get_map_size(&self) -> Vector<MapDistance> {
        let (rows, columns) = self.map.dim();
        Vector {
            x: numeric_types::map_dist(columns as i32),
            y: numeric_types::map_dist(rows as i32),
        }
    }

    fn get_map_pos(&self, pos: Vector<P::MouseDistance>) -> Option<Vector<MapDistance>> {
        let screen_pos = pos.cast::<P::ScreenDistance>();
        let pos_on_screen = screen_pos.piecewise_divide(self.get_tile_size());
        Some(Vector::<MapDistance>::from(pos_on_screen.lossy_cast::<i32>()?) + self.screen.top_left)
    }

    fn move_cursor(&mut self, pos: Vector<MapDistance>) {
        let old_pos = self.cursor_pos;
        self.draw_tile(self.get_tile(old_pos), old_pos);
        self.cursor_pos = pos;
        self.draw_cursor();
        self.draw_infobar();
    }

    fn redraw(&self) {
        let top_left = self.screen.top_left;
        let top_left_index = top_left.lossy_cast::<usize>().expect("Failed cast");
        let bottom_right_option = (top_left + self.screen.size).lossy_cast::<usize>();
        let bottom_right = bottom_right_option.expect("Failed cast");
        let slice_helper = ndarray::s![
            top_left_index.y..bottom_right.y,
            top_left_index.x..bottom_right.x
        ];
        for ((r, c), t) in self.map.slice(slice_helper).indexed_iter() {
            let map_pos = Vector {
                x: numeric_types::map_dist(c as i32),
                y: numeric_types::map_dist(r as i32),
            } + top_left;
            self.draw_tile(t, map_pos);
        }
        self.draw_cursor();
        self.draw_infobar();
    }

    fn select_tile(&mut self) {
        for t in &self.highlighted_tiles {
            (**t).remaining_move = UNREACHABLE;
        }
        self.highlighted_tiles.clear();
        let start_tile = self.get_tile(self.cursor_pos);
        if let Some(u) = start_tile.unit {
            let map = self.map;
            let mut tiles_to_process = binary_heap_plus::BinaryHeap::new_by(
                |a: &Vector<MapDistance>, b: &Vector<MapDistance>| {
                    get_map_tile(&map, *a)
                        .remaining_move
                        .cmp(&get_map_tile(&map, *b).remaining_move)
                },
            );
            start_tile.remaining_move = u.remaining_move;
            tiles_to_process.push(self.cursor_pos);
            while let Some(p) = tiles_to_process.pop() {
                let tile = self.get_tile(p);
                self.highlighted_tiles.push(tile);
                let remaining_move = tile.remaining_move;
                self.queue_for_processing(&mut tiles_to_process, p + UP, remaining_move);
                self.queue_for_processing(&mut tiles_to_process, p + DOWN, remaining_move);
                self.queue_for_processing(&mut tiles_to_process, p + LEFT, remaining_move);
                self.queue_for_processing(&mut tiles_to_process, p + RIGHT, remaining_move);
            }
        }
        self.redraw();
    }

    fn add_unit(&mut self, unit: serialization::Unit) {
        if let Some(t) = self.map.get((
            unit.position.y.value as usize,
            unit.position.x.value as usize,
        )) {
            t.unit = Some(unit.into())
        }
    }

    fn try_get_tile(&self, pos: Vector<MapDistance>) -> Option<& Tile<P>> {
        self.map.get((pos.y.value as usize, pos.x.value as usize))
    }

    fn get_tile(&self, pos: Vector<MapDistance>) -> &Tile<P> {
        get_map_tile::<P>(&self.map, pos)
    }

    fn get_screen_pos(&self, pos: Vector<MapDistance>) -> Rectangle<P::ScreenDistance> {
        let tile_size = self.get_tile_size();
        let top_left = pos - self.screen.top_left;
        Rectangle {
            top_left: tile_size
                .piecewise_multiply(top_left.lossy_cast::<P::ScreenDistance>().unwrap()),
            size: tile_size,
        }
    }

    fn draw_cursor(&self) {
        let cursor_pos_on_screen = self.get_screen_pos(self.cursor_pos);
        self.platform
            .attempt_draw(self.cursor_image.as_ref(), &cursor_pos_on_screen);
    }

    fn draw_infobar(&self) {
        let height = self.platform.get_height() / P::ScreenDistance::from(15);
        let size = Vector::<P::ScreenDistance> {
            x: height * P::ScreenDistance::from(4),
            y: height,
        };
        let position = Rectangle {
            top_left: Vector {
                x: 0.into(),
                y: 0.into(),
            },
            size,
        };

        let tile = self.get_tile(self.cursor_pos);

        let offset_scalar = size.y / P::ScreenDistance::from(4);
        let offset = Vector {
            x: offset_scalar,
            y: offset_scalar,
        };
        let max_width = size.x * P::ScreenDistance::from_f64(0.75).unwrap_or_else(|| 1.into());
        let stat_y = utility::multiply_frac(height, 5, 8);

        if let Some(unit) = tile.unit {
            self.platform
                .attempt_draw(self.unit_infobar.as_ref(), &position);
            self.platform.draw_text(&unit.name, offset, max_width);
            self.platform.draw_text(
                format!("lv {}", unit.level.value).as_str(),
                Vector {
                    x: offset_scalar,
                    y: stat_y,
                },
                size.y,
            );
            let hp_x = utility::multiply_frac(size.y, 5, 2);
            let hp_str = format!("{}/{}", unit.hp.value, unit.hp.value);
            self.platform
                .draw_text(hp_str.as_str(), Vector { x: hp_x, y: stat_y }, size.y);
        } else {
            let info = *tile.info;

            self.platform
                .attempt_draw(self.infobar_image.as_ref(), &position);
            self.platform.draw_text(&info.name, offset, max_width);
            let stat_width = height * P::ScreenDistance::from(13) / P::ScreenDistance::from(16);
            let move_pos = Vector {
                x: utility::multiply_frac(height, 3, 4),
                y: stat_y,
            };
            let defense_pos = Vector {
                x: utility::multiply_frac(height, 15, 8),
                y: stat_y,
            };
            let evade_pos = Vector {
                x: height * P::ScreenDistance::from(3),
                y: stat_y,
            };
            self.platform.draw_text(
                info.move_cost.value.to_string().as_str(),
                move_pos,
                stat_width,
            );
            self.platform.draw_text(
                info.defense.value.to_string().as_str(),
                defense_pos,
                stat_width,
            );
            self.platform
                .draw_text(info.evade.value.to_string().as_str(), evade_pos, stat_width);
        }
    }

    fn draw_tile(&self, tile: &Tile<P>, pos: Vector<MapDistance>) {
        let screen_pos = self.get_screen_pos(pos);
        self.platform.attempt_draw((*tile.info).image.map(|i| i), &screen_pos);
        if let Some(u) = tile.unit {
            self.platform
                .attempt_draw(self.unit_images.get(&u.class), &screen_pos);
        }
        if tile.remaining_move >= ZERO_TILES {
            self.platform.draw_rectangle(
                screen_pos.left(),
                screen_pos.top(),
                screen_pos.width(),
                screen_pos.height(),
            );
        }
    }

    fn queue_for_processing<C: compare::Compare<Vector<MapDistance>>>(
        &mut self,
        tiles_to_process: &mut binary_heap_plus::BinaryHeap<Vector<MapDistance>, C>,
        pos: Vector<MapDistance>,
        remaining_move: MapDistance,
    ) {
        if let Some(t) = self.try_get_tile(pos) {
            let rem = remaining_move - (*t.info).move_cost;
            if rem > t.remaining_move {
                t.remaining_move = rem;
                tiles_to_process.push(pos);
            }
        }
    }

}

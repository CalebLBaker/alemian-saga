use crate::*;
use detail::*;

// Struct for holding game state
pub struct Game<'a, P: Platform> {
    pub platform: P,
    pub cursor_pos: Vector<MapDistance>,
    pub map: ndarray::Array2<Tile<'a, P>>,
    pub cursor_image: Option<P::Image>,
    pub infobar_image: Option<P::Image>,
    pub unit_infobar: Option<P::Image>,
    pub screen: Rectangle<MapDistance>,
    pub last_mouse_pan: P::Instant,
    pub unit_images: std::collections::HashMap<serialization::Class, P::Image>,
    // Stored as pointers to avoid keeping multiple mutable references to tile
    // Will only be used internally when self is mutable
    highlighted_tiles: Vec<*mut Tile<'a, P>>,
}

impl<'a, P: Platform> Game<'a, P> {

    pub fn new(platform: P, map: ndarray::Array2<Tile<'a, P>>, cursor_image: Option<P::Image>,
               infobar_image: Option<P::Image>, unit_infobar: Option<P::Image>, last_mouse_pan: P::Instant) -> Self {
        let (rows, columns) = map.dim();
        Self {
            platform,
            cursor_pos: Vector { x: numeric_types::ZERO_TILES, y: numeric_types::ZERO_TILES },
            map,
            cursor_image,
            infobar_image,
            unit_infobar,
            screen: Rectangle {
                top_left: Vector {
                    x: numeric_types::ZERO_TILES,
                    y: numeric_types::ZERO_TILES
                },
                size: Vector {
                    x: numeric_types::map_dist(columns as i32),
                    y: numeric_types::map_dist(rows as i32)
                }
            },
            last_mouse_pan,
            unit_images: std::collections::HashMap::new(),
            highlighted_tiles: Vec::new()
        }
    }

    pub fn get_tile_size(&self) -> Vector<P::ScreenDistance> {
        self.platform
            .get_screen_size()
            .piecewise_divide(self.screen.size.lossy_cast::<P::ScreenDistance>().unwrap())
    }

    fn get_mut_tile(&mut self, pos: Vector<MapDistance>) -> &mut Tile<'a, P> {
        &mut self.map[[pos.y.value as usize, pos.x.value as usize]]
    }

    pub fn get_tile(&self, pos: Vector<MapDistance>) -> &Tile<'a, P> {
        &self.map[[pos.y.value as usize, pos.x.value as usize]]
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

    pub fn get_map_size(&self) -> Vector<MapDistance> {
        let (rows, columns) = self.map.dim();
        Vector {
            x: numeric_types::map_dist(columns as i32),
            y: numeric_types::map_dist(rows as i32),
        }
    }

    pub fn get_map_pos(&self, pos: Vector<P::MouseDistance>) -> Option<Vector<MapDistance>> {
        let screen_pos = pos.cast::<P::ScreenDistance>();
        let pos_on_screen = screen_pos.piecewise_divide(self.get_tile_size());
        Some(Vector::<MapDistance>::from(pos_on_screen.lossy_cast::<i32>()?) + self.screen.top_left)
    }

    pub fn move_cursor(&mut self, pos: Vector<MapDistance>) {
        let old_pos = self.cursor_pos;
        self.draw_tile(self.get_tile(old_pos), old_pos);
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
            self.platform.draw_text(unit.name, offset, max_width);
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
            let info = &tile.info;

            self.platform
                .attempt_draw(self.infobar_image.as_ref(), &position);
            self.platform.draw_text(info.name, offset, max_width);
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

    pub fn redraw(&self) {
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

    fn draw_tile(&self, tile: &Tile<'a, P>, pos: Vector<MapDistance>) {
        let screen_pos = self.get_screen_pos(pos);
        self.platform.attempt_draw(tile.image, &screen_pos);
        if let Some(u) = tile.unit {
            self.platform
                .attempt_draw(self.unit_images.get(&u.class), &screen_pos);
        }
        if tile.highlighted {
            self.platform.draw_rectangle(screen_pos.left(), screen_pos.top(), screen_pos.width(), screen_pos.height());
        }
    }

    pub fn select_tile(&mut self) {
        for t in &self.highlighted_tiles {
            // We could do the same thing safely by storing Vector's in highlighted_tiles and using
            // get_mut_tile, but this should be more performant.
            // And it's pretty trivial to see that this is actually quite safe, even if the
            // compiler can't be convinced of it
            unsafe { (**t).highlighted = false; }
        }
        self.highlighted_tiles.clear();
        let tile = self.get_mut_tile(self.cursor_pos);
        if tile.unit.is_some() {
            tile.highlighted = true;
            let tile_ptr = tile as *mut Tile<'a, P>;
            self.highlighted_tiles.push(tile_ptr);
        }
        self.redraw();
    }
}

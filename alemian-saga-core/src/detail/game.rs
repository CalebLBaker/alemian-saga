use crate::*;
use constants::*;
use detail::*;

// Struct for holding game state
pub struct Game<'a, 'b, P: Platform> {
    pub platform: P,
    pub cursor_pos: Vector<MapDistance>,
    pub map: &'b ndarray::Array2<Tile<'a, P>>,
    pub cursor_image: Option<P::Image>,
    pub infobar_image: Option<P::Image>,
    pub unit_infobar: Option<P::Image>,
    pub screen: Rectangle<MapDistance>,
    pub last_mouse_pan: P::Instant,
    pub unit_images: std::collections::HashMap<serialization::Class, P::Image>,
    highlighted_tiles: Vec<&'b Tile<'a, P>>,
}

fn get_map_tile<'a, 'b, P: Platform>(
    map: &'b ndarray::Array2<Tile<'a, P>>,
    pos: Vector<MapDistance>,
) -> &'b Tile<'a, P> {
    &map[[pos.y.value as usize, pos.x.value as usize]]
}

impl<'a, 'b, P: Platform> Game<'a, 'b, P> {
    pub fn new(
        platform: P,
        map: &'b mut ndarray::Array2<Tile<'a, P>>,
        cursor_image: Option<P::Image>,
        infobar_image: Option<P::Image>,
        unit_infobar: Option<P::Image>,
        last_mouse_pan: P::Instant,
    ) -> Self {
        let (rows, columns) = map.dim();
        Self {
            platform,
            cursor_pos: Vector {
                x: ZERO_TILES,
                y: ZERO_TILES,
            },
            map,
            cursor_image,
            infobar_image,
            unit_infobar,
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
            highlighted_tiles: Vec::new(),
        }
    }

    pub fn get_tile_size(&self) -> Vector<P::ScreenDistance> {
        self.platform
            .get_screen_size()
            .piecewise_divide(self.screen.size.lossy_cast::<P::ScreenDistance>().unwrap())
    }

    fn try_get_tile(&self, pos: Vector<MapDistance>) -> Option<&'b Tile<'a, P>> {
        self.map.get((pos.y.value as usize, pos.x.value as usize))
    }

    pub fn get_tile(&self, pos: Vector<MapDistance>) -> &'b Tile<'a, P> {
        get_map_tile(self.map, pos)
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

        if let Some(unit) = tile.unit.get() {
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
        if let Some(u) = tile.unit.get() {
            self.platform
                .attempt_draw(self.unit_images.get(&u.class), &screen_pos);
        }
        if tile.remaining_move.get() >= ZERO_TILES {
            self.platform.draw_rectangle(
                screen_pos.left(),
                screen_pos.top(),
                screen_pos.width(),
                screen_pos.height(),
            );
        }
    }

    fn queue_for_processing<C: compare::Compare<Vector<MapDistance>>>(
        &self,
        tiles_to_process: &mut binary_heap_plus::BinaryHeap<Vector<MapDistance>, C>,
        pos: Vector<MapDistance>,
        remaining_move: MapDistance,
    ) {
        if let Some(t) = self.try_get_tile(pos) {
            let rem = remaining_move - t.info.move_cost;
            if rem > t.remaining_move.get() {
                t.remaining_move.set(rem);
                tiles_to_process.push(pos);
            }
        }
    }

    pub fn select_tile(&mut self) {
        for t in &self.highlighted_tiles {
            t.remaining_move.set(UNREACHABLE);
        }
        self.highlighted_tiles.clear();
        let start_tile = self.get_tile(self.cursor_pos);
        if let Some(u) = start_tile.unit.get() {
            let map = self.map;
            let mut tiles_to_process = binary_heap_plus::BinaryHeap::new_by(
                |a: &Vector<MapDistance>, b: &Vector<MapDistance>| {
                    get_map_tile(map, *a)
                        .remaining_move
                        .cmp(&get_map_tile(map, *b).remaining_move)
                },
            );
            start_tile.remaining_move.set(u.remaining_move);
            tiles_to_process.push(self.cursor_pos);
            while let Some(p) = tiles_to_process.pop() {
                let tile = self.get_tile(p);
                self.highlighted_tiles.push(tile);
                let remaining_move = tile.remaining_move.get();
                self.queue_for_processing(&mut tiles_to_process, p + UP, remaining_move);
                self.queue_for_processing(&mut tiles_to_process, p + DOWN, remaining_move);
                self.queue_for_processing(&mut tiles_to_process, p + LEFT, remaining_move);
                self.queue_for_processing(&mut tiles_to_process, p + RIGHT, remaining_move);
            }
        }
        self.redraw();
    }

    pub fn add_unit(&mut self, unit: &'a serialization::Unit<'a>) {
        if let Some(t) = self.map.get((
            unit.position.y.value as usize,
            unit.position.x.value as usize,
        )) {
            t.unit.set(Some(unit))
        }
    }
}

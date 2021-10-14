#![cfg_attr(feature = "strict", deny(warnings))]
#![feature(const_fn_trait_bound)]

mod detail;
pub mod numeric_types;
pub mod serialization;

#[macro_use]
extern crate uom;

use std::{cmp, ops};

use async_trait::async_trait;
use num_traits::FromPrimitive;

use detail::Rectangle;

// Trait used for abstracting away logic that is specific to a particular platform
#[async_trait(?Send)]
pub trait Platform {
    // Tyep used to represent errors
    type Error: std::string::ToString;

    // Type used to represent images
    type Image;

    // Type used to represent user input (keyboard or button)
    type InputType: Eq + std::hash::Hash;

    // Type used to represent distance in mouse events (should be the same ScreenDistance
    type MouseDistance: Copy;

    // Type used to represent distance on the screen
    type ScreenDistance: From<i32>
        + Copy
        + From<Self::MouseDistance>
        + ops::Add<Output = Self::ScreenDistance>
        + ops::Sub<Output = Self::ScreenDistance>
        + ops::Mul<Output = Self::ScreenDistance>
        + ops::Div<Output = Self::ScreenDistance>
        + cmp::PartialOrd
        + num_traits::ToPrimitive
        + num_traits::NumCast
        + FromPrimitive;

    // Future type returned by get_image
    type ImageFuture: std::future::Future<Output = Option<Self::Image>>;

    // Type used to represent files
    type File: std::convert::AsRef<[u8]>;

    // Type used to represent user-specific files (might be different type from more general
    // files
    type UserFile: std::convert::AsRef<[u8]>;

    // Type used to represent moments in time
    type Instant: Copy;

    // Type used to represent lengths of time
    type Duration: cmp::PartialOrd;

    // Draw an image to the screen
    fn draw_primitive(
        &self,
        img: &Self::Image,
        left: Self::ScreenDistance,
        top: Self::ScreenDistance,
        width: Self::ScreenDistance,
        height: Self::ScreenDistance,
    );

    // Draw a rectangle to the screen
    fn draw_rectangle(
        &self,
        left: Self::ScreenDistance,
        top: Self::ScreenDistance,
        width: Self::ScreenDistance,
        height: Self::ScreenDistance,
    );

    // Renders text to the screen
    fn draw_text_primitive(
        &self,
        text: &str,
        x: Self::ScreenDistance,
        y: Self::ScreenDistance,
        max_width: Self::ScreenDistance,
    );

    // Converts a Sring into an InputType
    fn string_to_input(input: &str) -> Self::InputType;

    // Get the width of the game screen
    fn get_width(&self) -> Self::ScreenDistance;

    // Get the height of the game screen
    fn get_height(&self) -> Self::ScreenDistance;

    // Retrieve an image from a specified file path
    fn get_image(path: &str) -> Self::ImageFuture;

    // Retrieve a file from a specified file path
    async fn get_file(&self, path: &str) -> Result<Self::File, Self::Error>;

    // Retrieve a user specific file
    async fn get_user_file(&self, path: &str) -> Result<Self::UserFile, Self::Error>;

    // Log a message (typically to stdout or the equivalent)
    fn log(path: &str);

    // Gets the current moment in time
    fn now() -> Self::Instant;

    // Converts an integer value in nanoseconds into a Duration object
    fn nanoseconds(ns: usize) -> Self::Duration;

    // Gets the amount of time between two moments
    fn duration_between(fist: Self::Instant, second: Self::Instant) -> Self::Duration;

    // Gets the size of the screen
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

    // Adds keybinds to a keybinding map
    fn add_bindings(
        map: &mut std::collections::HashMap<Self::InputType, Event<Self::MouseDistance>>,
        keys: Vec<&str>,
        event: Event<Self::MouseDistance>,
    ) {
        for k in keys.into_iter() {
            map.insert(Self::string_to_input(k), event);
        }
    }

    // Retrieves a keybinding map describing the what keys map to what actions
    async fn get_keybindings(
        &self,
        locale: &str,
    ) -> Option<std::collections::HashMap<Self::InputType, Event<Self::MouseDistance>>> {
        let mut ret = std::collections::HashMap::new();
        let user_file = self.get_user_file("keybindings.json").await;
        let file: detail::FileWrapper<Self> = match user_file {
            Ok(f) => detail::FileWrapper::User(f),
            _ => {
                let keybindings_path = format!("keybindings/{}.json", locale);
                detail::FileWrapper::Global(self.get_file(keybindings_path.as_str()).await.ok()?)
            }
        };
        let bindings: detail::Keybindings = serde_json::from_slice(file.as_ref()).ok()?;
        Self::add_bindings(&mut ret, bindings.Right, Event::Right);
        Self::add_bindings(&mut ret, bindings.Left, Event::Left);
        Self::add_bindings(&mut ret, bindings.Up, Event::Up);
        Self::add_bindings(&mut ret, bindings.Down, Event::Down);
        Self::add_bindings(&mut ret, bindings.ZoomIn, Event::ZoomIn);
        Self::add_bindings(&mut ret, bindings.ZoomOut, Event::ZoomOut);
        Self::add_bindings(&mut ret, bindings.Select, Event::Select);
        Some(ret)
    }

    // Renders text to the screen
    fn draw_text(
        &self,
        text: &str,
        offset: Vector<Self::ScreenDistance>,
        max_width: Self::ScreenDistance,
    ) {
        self.draw_text_primitive(text, offset.x, offset.y, max_width);
    }
}

// Represents a vector
#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Vector<T> {
    pub x: T,
    pub y: T,
}

// Type used to represent user input events
#[derive(Clone, Copy)]
pub enum Event<P> {
    Right,
    Left,
    Up,
    Down,
    ZoomIn,
    ZoomOut,
    MouseMove(Vector<P>),
    Redraw,
    Select,
}

// Entry point for starting game logic
pub async fn run<P: Platform>(
    platform: P,
    mut event_queue: futures::channel::mpsc::Receiver<Event<P::MouseDistance>>,
    language: &str,
) {
    if let Err(e) = detail::run_internal(platform, &mut event_queue, language).await {
        P::log(e.msg.as_str());
    }
}

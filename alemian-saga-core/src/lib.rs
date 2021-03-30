#![cfg_attr(feature = "strict", deny(warnings))]

mod detail;
pub mod serialization;

use std::{cmp, ops};

use async_trait::async_trait;
use num_traits::FromPrimitive;

use detail::Rectangle;

// A trait that should be implemented by all primitive numberic types
pub trait Scalar:
    ops::Sub<Output = Self>
    + ops::Div<Output = Self>
    + ops::Mul<Output = Self>
    + ops::Add<Output = Self>
    + cmp::PartialOrd
    + Copy
{
}

impl<T> Scalar for T where
    T: ops::Sub<Output = T>
        + ops::Div<Output = T>
        + ops::Mul<Output = Self>
        + ops::Add<Output = Self>
        + cmp::PartialOrd
        + Copy
{
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
    type ScreenDistance: Scalar
        + From<u32>
        + From<Self::MouseDistance>
        + num_traits::ToPrimitive
        + FromPrimitive;

    // Future type returned by get_image
    type ImageFuture: std::future::Future<Output = Option<Self::Image>>;

    // Type used to represent files
    type File: std::io::Read;

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

    // Renders text to the screen
    fn draw_text_primitive(
        &self,
        text: &str,
        x: Self::ScreenDistance,
        y: Self::ScreenDistance,
        max_width: Self::ScreenDistance,
    );

    // Converts a Sring into an InputType
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
        keys: Vec<String>,
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
        let keybindings_path = format!("keybindings/{}.json", locale);
        let bindings_file = self.get_file(keybindings_path.as_str()).await.ok()?;
        let bindings: detail::Keybindings = serde_json::from_reader(bindings_file).ok()?;
        Self::add_bindings(&mut ret, bindings.Right, Event::Right);
        Self::add_bindings(&mut ret, bindings.Left, Event::Left);
        Self::add_bindings(&mut ret, bindings.Up, Event::Up);
        Self::add_bindings(&mut ret, bindings.Down, Event::Down);
        Self::add_bindings(&mut ret, bindings.ZoomIn, Event::ZoomIn);
        Self::add_bindings(&mut ret, bindings.ZoomOut, Event::ZoomOut);
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
#[derive(Clone, Copy)]
pub struct Vector<T> {
    pub x: T,
    pub y: T,
}

// Type used to represent user input events
#[derive(Clone, Copy)]
pub enum Event<P: Scalar> {
    Right,
    Left,
    Up,
    Down,
    ZoomIn,
    ZoomOut,
    MouseMove(Vector<P>),
    Redraw,
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

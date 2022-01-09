#![cfg_attr(feature = "strict", deny(warnings))]

use async_trait::async_trait;
use futures::task::SpawnExt;

struct Application {
    // _window: piston_window::PistonWindow
}

impl Application {
    fn new() -> Self {
        // let settings = piston_window::WindowSettings::new("Alemian Saga", [640, 480]).fullscreen(true);
        Application{ /* _window: settings.decorated(false).build().unwrap() */ }
    }
}

struct File {
    data: [u8; 0]
}

impl AsRef<[u8]> for File {
    fn as_ref(&self) -> &[u8] { 
        &self.data
    }
}

struct Error {
}

impl std::fmt::Display for Error {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

#[async_trait]
impl alemian_saga_core::Platform for Application {
    type Error = Error;
    type Image = ();
    type InputType = i32;
    type MouseDistance = i32;
    type ScreenDistance = i32;
    type ImageFuture = std::future::Ready<Option<()>>;
    type File = File;
    type UserFile = File;
    type Instant = ();
    type Duration = i32;
    fn draw_primitive(
        &self,
        _img: &Self::Image,
        _left: Self::ScreenDistance,
        _top: Self::ScreenDistance,
        _width: Self::ScreenDistance,
        _height: Self::ScreenDistance,
    ) {
    }

    fn draw_rectangle(
        &self,
        _left: Self::ScreenDistance,
        _top: Self::ScreenDistance,
        _width: Self::ScreenDistance,
        _height: Self::ScreenDistance,
    ) {
    }

    // Renders text to the screen
    fn draw_text_primitive(
        &self,
        _text: &str,
        _x: Self::ScreenDistance,
        _y: Self::ScreenDistance,
        _max_width: Self::ScreenDistance,
    ) {
    }

    // Converts a Sring into an InputType
    fn string_to_input(_input: &str) -> Self::InputType {
        0
    }

    // Get the width of the game screen
    fn get_width(&self) -> Self::ScreenDistance {
        0
    }

    // Get the height of the game screen
    fn get_height(&self) -> Self::ScreenDistance {
        0
    }

    // Retrieve an image from a specified file path
    fn get_image(_path: &str) -> Self::ImageFuture {
        std::future::ready(None)
    }

    // Retrieve a file from a specified file path
    async fn get_file(&self, _path: &str) -> Result<Self::File, Self::Error> {
        Err(Error{})
    }

    // Retrieve a user specific file
    async fn get_user_file(&self, _path: &str) -> Result<Self::UserFile, Self::Error> {
        Err(Error{})
    }

    // Log a message (typically to stdout or the equivalent)
    fn log(_path: &str) {}

    // Gets the current moment in time
    fn now() -> Self::Instant {}

    // Converts an integer value in nanoseconds into a Duration object
    fn nanoseconds(_ns: usize) -> Self::Duration {
        0
    }

    // Gets the amount of time between two moments
    fn duration_between(_fist: Self::Instant, _second: Self::Instant) -> Self::Duration {
        0
    }
}

async fn run_game() {
    let (_sender, receiver) = futures::channel::mpsc::channel(8);
    alemian_saga_core::run(Application::new(), receiver, "english").await;
}

fn main() {
    match futures::executor::ThreadPool::new() {
        Ok(pool) => match pool.spawn_with_handle(
            if let Err(e) = pool.spawn(run_game()) {
            println!("{}", e);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}

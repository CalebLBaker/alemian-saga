#![cfg_attr(feature = "strict", deny(warnings))]

use async_trait::async_trait;

#[async_trait(?Send)]
pub trait Platform {
    type Image;
    type ImageFuture: std::future::Future<Output = Option<Self::Image>>;
    type File: std::io::Read;
    fn draw(&self, img: &Self::Image, left: f64, top: f64, width: f64, height: f64);
    fn get_width(&self) -> f64;
    fn get_height(&self) -> f64;
    fn get_image(path: &str) -> Self::ImageFuture;
    async fn get_file(&self, path: &str) -> Option<Self::File>;
}

pub type TileType = String;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Map {
    pub tile_types: Vec<TileType>,
    pub map: ndarray::Array2<u32>,
}

pub async fn run<P: Platform>(platform: P) -> Option<()> {
    let image = P::get_image("plain.png");
    platform.draw(
        &image.await?,
        0.0,
        0.0,
        platform.get_width(),
        platform.get_height(),
    );
    Some({})
}


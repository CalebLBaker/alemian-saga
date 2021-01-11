pub trait Platform {
    type Image;
    type ImageFuture: std::future::Future<Output = Option<Self::Image>>;
    fn draw(&self, img: &Self::Image, left: f64, top: f64, width: f64, height: f64);
    fn get_width(&self) -> f64;
    fn get_height(&self) -> f64;
    fn get_image(path: &str) -> Self::ImageFuture;
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

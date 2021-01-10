pub trait Platform {
    type Image;
    fn draw(&self, img: &Self::Image, left: f64, top: f64, width: f64, height: f64);
    fn get_width(&self) -> f64;
    fn get_height(&self) -> f64;
}

pub async fn run<P: Platform, F: std::future::Future<Output = P::Image>, G: Fn(&str) -> Option<F>>(platform: P, get_image: G) -> Option<()> {
    let image = get_image("plain.png")?;
    platform.draw(&image.await, 0.0, 0.0, platform.get_width(), platform.get_height());
    Some({})
}


use async_trait::async_trait;
use alemian_saga_core::*;
use futures::SinkExt;
use Event::*;
use ndarray::array;

#[derive(Clone)]
enum Drawing {
    Image{
        source: String,
        x: u32,
        y: u32,
        w: u32,
        h: u32
    },
    Text{
        txt: String,
        tx: u32,
        ty: u32,
    }
}

struct TestPlatform {
    drawings: std::sync::mpsc::Receiver<Drawing>
}

#[async_trait(?Send)]
impl alemian_saga_core::Platform for TestPlatform {
    type Image = String;
    type InputType = ();
    type MouseDistance = u32;
    type ScreenDistance = u32;
    type ImageFuture = std::future::Ready<Option<Self::Image>>;
    type File = std::io::Cursor<Vec<u8>>;
    type Instant = ();
    type Duration = u32;
    fn draw_primitive(&self, img: &Self::Image, left: Self::ScreenDistance, top: Self::ScreenDistance, width: Self::ScreenDistance, height: Self::ScreenDistance) {
        println!("drawing {}", img);
        match self.drawings.try_recv().expect(format!("unexpected draw: {}", img).as_str()) {
            Drawing::Image{source, x, y, w, h} => {
                assert_eq!(img, &source);
                assert_eq!(left, x);
                assert_eq!(top, y);
                assert_eq!(width, w);
                assert_eq!(height, h);
            }
            _ => panic!()
        }
    }
    fn draw_text_primitive(&self, text: &str, x: Self::ScreenDistance, y: Self::ScreenDistance, _max_width: Self::ScreenDistance) {
        match self.drawings.try_recv().expect(format!("unexpected write: {}", text).as_str()) {
            Drawing::Text{txt, tx, ty} => {
                assert_eq!(&txt, text);
                assert_eq!(tx, x);
                assert_eq!(ty, y);
            }
            _ => panic!()
        }
    }
    fn string_to_input(_input: String) -> Self::InputType { panic!(); }
    fn get_width(&self) -> Self::ScreenDistance { 80 }
    fn get_height(&self) -> Self::ScreenDistance { 60 }
    fn get_image(path: &str) -> Self::ImageFuture {
        std::future::ready(Some(path.to_owned()))
    }
    async fn get_file(&self, path: &str) -> Result<Self::File, String> {
        if path == "map.map" {
            Ok(std::io::Cursor::new(rmp_serde::encode::to_vec(&serialization::Map{
                tile_types: vec![
                    serialization::TileType{image: "a".to_owned(), name: "a".to_owned() },
                    serialization::TileType{image: "b".to_owned(), name: "b".to_owned() },
                    serialization::TileType{image: "c".to_owned(), name: "c".to_owned() },
                    serialization::TileType{image: "d".to_owned(), name: "d".to_owned() }
                ],
                map: array![
                    [0, 1],
                    [2, 3]
                ]
            }).unwrap()))
        }
        else {
            panic!("Unknown file: {}", path);
        }
    }
    fn log(path: &str) {
        println!("{}", path);
    }
    fn now() -> Self::Instant { }
    fn nanoseconds(_ns: usize) -> Self::Duration { 0 }
    fn duration_between(_first: Self::Instant, _second: Self::Instant) -> Self::Duration {
        1
    }
}

fn image(source: &str, x: u32, y: u32, width: u32, height: u32) -> Drawing {
    Drawing::Image{
        source: source.to_owned(),
        x,
        y,
        w: width,
        h: height
    }
}

fn expect_infobar(sender: &mut std::sync::mpsc::Sender<Drawing>, text: &str) {
    let _ = sender.send(image("infobar.png", 0, 0, 16, 4));
    let _ = sender.send(Drawing::Text{ txt: text.to_owned(), tx: 1, ty: 1 });
}

async fn run_test() {
    let (mut drawing_sender, drawing_receiver) = std::sync::mpsc::channel();
    let (mut event_sender, event_receiver) = futures::channel::mpsc::channel(512);
    let mut tile_height = 30;
    let mut tile_width = 40;

    let platform = TestPlatform { drawings: drawing_receiver };
    let game_future = alemian_saga_core::run(platform, event_receiver);

    let _ = drawing_sender.send(image("a", 0, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("b", tile_width, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("c", 0, tile_height, tile_width, tile_height));
    let _ = drawing_sender.send(image("d", tile_width, tile_height, tile_width, tile_height));
    let _ = drawing_sender.send(image("cursor.png", 0, 0, tile_width, tile_height));
    expect_infobar(&mut drawing_sender, "a");

    let _ = drawing_sender.send(image("a", 0, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("cursor.png", tile_width, 0, tile_width, tile_height));
    expect_infobar(&mut drawing_sender, "b");
    event_sender.send(Right).await.unwrap();

    let _ = drawing_sender.send(image("b", tile_width, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("cursor.png", tile_width, tile_height, tile_width, tile_height));
    expect_infobar(&mut drawing_sender, "d");
    event_sender.send(Down).await.unwrap();

    tile_height *= 2;
    let _ = drawing_sender.send(image("c", 0, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("d", tile_width, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("cursor.png", tile_width, 0, tile_width, tile_height));
    expect_infobar(&mut drawing_sender, "d");
    event_sender.send(ZoomIn).await.unwrap();

    tile_width *= 2;
    let _ = drawing_sender.send(image("d", 0, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("cursor.png", 0, 0, tile_width, tile_height));
    expect_infobar(&mut drawing_sender, "d");
    event_sender.send(ZoomIn).await.unwrap();

    let _ = drawing_sender.send(image("c", 0, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("cursor.png", 0, 0, tile_width, tile_height));
    expect_infobar(&mut drawing_sender, "c");
    event_sender.send(Left).await.unwrap();

    let _ = drawing_sender.send(image("a", 0, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("cursor.png", 0, 0, tile_width, tile_height));
    expect_infobar(&mut drawing_sender, "a");
    event_sender.send(Up).await.unwrap();

    let _ = drawing_sender.send(image("b", 0, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("cursor.png", 0, 0, tile_width, tile_height));
    expect_infobar(&mut drawing_sender, "b");
    event_sender.send(MouseMove(Vector{x: 79, y: 30})).await.unwrap();

    tile_width /= 2;
    let _ = drawing_sender.send(image("a", 0, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("b", tile_width, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("cursor.png", tile_width, 0, tile_width, tile_height));
    expect_infobar(&mut drawing_sender, "b");
    event_sender.send(ZoomOut).await.unwrap();

    let _ = drawing_sender.send(image("a", 0, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("b", tile_width, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("cursor.png", tile_width, 0, tile_width, tile_height));
    expect_infobar(&mut drawing_sender, "b");
    event_sender.send(Redraw).await.unwrap();

    let _ = drawing_sender.send(image("b", tile_width, 0, tile_width, tile_height));
    let _ = drawing_sender.send(image("cursor.png", 0, 0, tile_width, tile_height));
    expect_infobar(&mut drawing_sender, "a");
    event_sender.send(MouseMove(Vector{x: 0, y: 0})).await.unwrap();

    event_sender.close_channel();

    game_future.await;

}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    futures::executor::block_on(run_test());
}


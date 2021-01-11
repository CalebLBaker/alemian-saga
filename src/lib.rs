#![cfg_attr(feature = "strict", deny(warnings))]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod game;

fn log(msg: &str) {
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(msg));
}

#[wasm_bindgen]
pub extern "C" fn start() {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(wrapper());
}

async fn wrapper() {
    match WebBrowser::new() {
        Some(b) => {
            let result = game::run(b);
            if !result.await.is_some() {
                log("Failed while running the game");
            }
        }
        None => {
            log("Failed to construct platform context");
        }
    }
}

struct LoadedImageElement {
    element: Option<web_sys::HtmlImageElement>,
    handler: Option<wasm_bindgen::closure::Closure<dyn FnMut()>>,
}

impl std::future::Future for LoadedImageElement {
    type Output = Option<web_sys::HtmlImageElement>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let future = self.get_mut();
        let element = future.element.as_mut();
        match element {
            Some(e) => {
                if e.complete() {
                    std::task::Poll::Ready(Some(future.element.take().unwrap()))
                } else {
                    let waker = cx.waker().clone();
                    let closure = Box::new(move || waker.wake_by_ref()) as Box<dyn FnMut()>;
                    future.handler = Some(wasm_bindgen::closure::Closure::wrap(closure));
                    let onload = Some(future.handler.as_ref().unwrap().as_ref().unchecked_ref());
                    e.set_onload(onload);
                    std::task::Poll::Pending
                }
            }
            None => std::task::Poll::Ready(None),
        }
    }
}

struct WebBrowser {
    context: web_sys::CanvasRenderingContext2d,
    width: f64,
    height: f64,
}

fn clear_margin_and_padding(element: &web_sys::HtmlElement) {
    let style = element.style();
    let _ = style.set_property("margin", "0");
    let _ = style.set_property("padding", "0");
}

impl WebBrowser {
    fn new() -> Option<WebBrowser> {
        const SIZE_MULTIPLIER: f64 = 0.995;
        let window = web_sys::window()?;
        let document = window.document()?;
        let width = window.inner_width().ok()?.as_f64()? * SIZE_MULTIPLIER;
        let height = window.inner_height().ok()?.as_f64()? * SIZE_MULTIPLIER;
        let canvas_element = document.get_element_by_id("gameCanvas")?;
        let canvas = canvas_element.dyn_ref::<web_sys::HtmlCanvasElement>()?;
        clear_margin_and_padding(
            document
                .document_element()?
                .dyn_ref::<web_sys::HtmlElement>()?,
        );
        clear_margin_and_padding(document.body()?.dyn_ref::<web_sys::HtmlElement>()?);
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);
        let context_object = canvas.get_context("2d").ok()??;
        let context = context_object
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .ok()?;
        Some(WebBrowser {
            context,
            width,
            height,
        })
    }
}

impl game::Platform for WebBrowser {
    type Image = web_sys::HtmlImageElement;

    type ImageFuture = LoadedImageElement;

    fn draw(&self, image: &Self::Image, left: f64, top: f64, width: f64, height: f64) {
        let context = &self.context;
        let _ = context
            .draw_image_with_html_image_element_and_dw_and_dh(image, left, top, width, height);
    }

    fn get_width(&self) -> f64 {
        self.width
    }

    fn get_height(&self) -> f64 {
        self.height
    }

    fn get_image(path: &str) -> Self::ImageFuture {
        let element = web_sys::HtmlImageElement::new();
        match element {
            Ok(e) => {
                e.set_src(path);
                LoadedImageElement {
                    element: Some(e),
                    handler: None,
                }
            }
            _ => LoadedImageElement {
                element: None,
                handler: None,
            },
        }
    }
}

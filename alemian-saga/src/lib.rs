#![cfg_attr(feature = "strict", deny(warnings))]
#![feature(unboxed_closures)]
#![feature(fn_traits)]

use std::pin;
use std::task;

use async_trait::async_trait;
use bytes::Buf;
use futures::channel::mpsc;
use futures::SinkExt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use alemian_saga_core::Platform;

const HOST: &str = "http://localhost/";
const FONT: &str = "1.5rem serif";
const EVENT_QUEUE_CAPACITY: usize = 8;

// Entry Point; Construct WebBrowser object and run game
#[wasm_bindgen]
pub extern "C" fn start() {
    enable_stack_trace();
    wasm_bindgen_futures::spawn_local(run_game());
}

#[cfg(feature = "stack-trace")]
fn enable_stack_trace() {
    console_error_panic_hook::set_once();
}

#[cfg(not(feature = "stack-trace"))]
fn enable_stack_trace() {}

async fn run_game() {
    let (sender, receiver) = mpsc::channel(EVENT_QUEUE_CAPACITY);
    match WebBrowser::new(HOST, sender).await {
        Some(p) => alemian_saga_core::run(p, receiver).await,
        None => WebBrowser::log("Failed to initialize game state"),
    }
}

// Future that yields an HtmlImageElement once the element has been fully loaded
struct LoadedImageElement {
    element: Option<web_sys::HtmlImageElement>,
    handler: Option<wasm_bindgen::closure::Closure<dyn FnMut()>>,
}

// Implementation of Future trait for LoadedImageElement
impl std::future::Future for LoadedImageElement {
    type Output = Option<web_sys::HtmlImageElement>;
    fn poll(self: pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let future = self.get_mut();
        let element = future.element.as_mut();
        match element {
            Some(e) => {
                if e.complete() {
                    task::Poll::Ready(Some(future.element.take().unwrap()))
                } else {
                    // If the element isn't complete, set an onload handler to wake the waker
                    let waker = cx.waker().clone();
                    let closure = Box::new(move || waker.wake_by_ref()) as Box<dyn FnMut()>;
                    future.handler = Some(wasm_bindgen::closure::Closure::wrap(closure));
                    let onload = Some(future.handler.as_ref().unwrap().as_ref().unchecked_ref());
                    e.set_onload(onload);
                    task::Poll::Pending
                }
            }
            None => task::Poll::Ready(None),
        }
    }
}

async fn send_async(
    mut event_queue: mpsc::Sender<alemian_saga_core::Event<i32>>,
    event: alemian_saga_core::Event<i32>,
) {
    let _ = event_queue.feed(event).await;
}

fn send(
    event_queue: &mut mpsc::Sender<alemian_saga_core::Event<i32>>,
    event: alemian_saga_core::Event<i32>,
) {
    if let Err(_) = event_queue.try_send(event) {
        wasm_bindgen_futures::spawn_local(send_async(event_queue.clone(), event));
    }
}

// Platform type that abstracts away logic that's specific to a web browser/wasm environment
struct WebBrowser<'a> {
    canvas: web_sys::HtmlCanvasElement,
    context: web_sys::CanvasRenderingContext2d,
    web_client: reqwest::Client,
    host: &'a str,
    _keyboard_handler: Option<gloo_events::EventListener>,
    _resize_handler: gloo_events::EventListener,
    _mouse_handler: gloo_events::EventListener,
    _scroll_handler: gloo_events::EventListener,
}

// Constructor and helper functions for the WebBrowser type
impl<'a> WebBrowser<'a> {
    fn handle_resize() -> Option<()> {
        let canvas_element = web_sys::window()?.document()?.get_element_by_id("g")?;
        let canvas = canvas_element.dyn_ref::<web_sys::HtmlCanvasElement>()?;
        canvas.set_width(canvas.client_width() as u32);
        canvas.set_height(canvas.client_height() as u32);
        Some(())
    }

    async fn new(
        host: &'a str,
        mut event_queue: mpsc::Sender<alemian_saga_core::Event<i32>>,
    ) -> Option<WebBrowser<'a>> {
        // Get handlers for various items from the Html document
        let window = web_sys::window()?;
        let document = window.document()?;
        let canvas_element = document.get_element_by_id("g")?;
        let canvas = canvas_element
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .ok()?;
        let document_element = document.document_element()?;

        // For whatever reason css doesn't populate the width and height field,
        // so we have to do that manually
        canvas.set_width(canvas.client_width() as u32);
        canvas.set_height(canvas.client_height() as u32);

        // Create the WebBrowser object
        let context_object = canvas.get_context("2d").ok()??;
        let context = context_object
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .ok()?;
        context.set_font(FONT);
        let web_client = reqwest::Client::new();

        let mut mouse_event_queue = event_queue.clone();

        let mouse_handler =
            gloo_events::EventListener::new(&document_element, "mousemove", move |e| {
                if let Some(mouse_event) = e.dyn_ref::<web_sys::MouseEvent>() {
                    send(
                        &mut mouse_event_queue,
                        alemian_saga_core::Event::MouseMove(alemian_saga_core::Vector {
                            x: mouse_event.offset_x(),
                            y: mouse_event.offset_y(),
                        }),
                    );
                }
            });

        let mut scroll_event_queue = event_queue.clone();

        let scroll_handler =
            gloo_events::EventListener::new(&document_element, "wheel", move |e| {
                if let Some(wheel_event) = e.dyn_ref::<web_sys::WheelEvent>() {
                    let delta_y = wheel_event.delta_y();
                    if delta_y < 0.0 {
                        send(&mut scroll_event_queue, alemian_saga_core::Event::ZoomIn);
                    } else if delta_y > 0.0 {
                        send(&mut scroll_event_queue, alemian_saga_core::Event::ZoomOut);
                    }
                }
            });

        let mut resize_event_queue = event_queue.clone();

        let resize_handler = gloo_events::EventListener::new(&window, "resize", move |_| {
            Self::handle_resize();
            send(&mut resize_event_queue, alemian_saga_core::Event::Redraw);
        });

        let mut ret = WebBrowser {
            canvas,
            context,
            web_client,
            host,
            _keyboard_handler: None,
            _resize_handler: resize_handler,
            _mouse_handler: mouse_handler,
            _scroll_handler: scroll_handler,
        };

        let key_bindings = ret.get_keybindings().await?;

        ret._keyboard_handler = Some(gloo_events::EventListener::new(
            &document_element,
            "keydown",
            move |e| {
                if let Some(keyboard_event) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                    if let Some(&game_event) = key_bindings.get(&keyboard_event.key()) {
                        send(&mut event_queue, game_event);
                    }
                }
            },
        ));

        Some(ret)
    }

    async fn get_file_internal(
        &self,
        path: &str,
    ) -> Result<bytes::buf::Reader<bytes::Bytes>, reqwest::Error> {
        let response = self.web_client.get(&(self.host.to_owned() + path)).send();
        Ok(response.await?.bytes().await?.reader())
    }
}

// Implementation of the Platform trait for the WebBrowser type
#[async_trait(?Send)]
impl alemian_saga_core::Platform for WebBrowser<'_> {
    type Image = web_sys::HtmlImageElement;

    type InputType = String;

    type MouseDistance = i32;

    type ScreenDistance = f64;

    type File = bytes::buf::Reader<bytes::Bytes>;

    type ImageFuture = LoadedImageElement;

    type Instant = f64;

    type Duration = f64;

    fn now() -> Self::Instant {
        js_sys::Date::now()
    }

    fn duration_between(first: Self::Instant, second: Self::Instant) -> Self::Duration {
        second - first
    }

    fn nanoseconds(ns: usize) -> Self::Duration {
        ns as f64 * 0.000001
    }

    fn draw_primitive(&self, image: &Self::Image, left: f64, top: f64, width: f64, height: f64) {
        let context = &self.context;
        let _ = context
            .draw_image_with_html_image_element_and_dw_and_dh(image, left, top, width, height);
    }

    fn draw_text_primitive(&self, text: &str, x: f64, y: f64, max_width: f64) {
        let _ = self
            .context
            .fill_text_with_max_width(text, x, y + 10.0, max_width);
    }

    fn get_width(&self) -> f64 {
        self.canvas.client_width() as f64
    }

    fn get_height(&self) -> f64 {
        self.canvas.client_height() as f64
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

    async fn get_file(&self, path: &str) -> Result<Self::File, String> {
        match self.get_file_internal(path).await {
            Ok(ret) => Ok(ret),
            Err(err) => Err(err.to_string()),
        }
    }

    fn string_to_input(input: String) -> Self::InputType {
        input
    }

    fn log(msg: &str) {
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(msg));
    }
}

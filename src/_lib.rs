use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub extern fn start() {
    if run().is_some() {
        alert("It's working!");
    }
    else {
        alert("Something went wrong");
    }
}

enum GlContext {
    Nothing,
    WebGl1(js_sys::Object),
    WebGl2(js_sys::Object),
}

fn get_context_map_else<M: FnOnce(js_sys::Object) -> GlContext, F: FnOnce() -> GlContext>(canvas: &web_sys::HtmlCanvasElement, typ: &str, mapper: M, fallback: F) -> GlContext {
    canvas.get_context(typ).ok().flatten().map_or_else(fallback, mapper)
}

fn get_context1_else<F: FnOnce() -> GlContext>(canvas: &web_sys::HtmlCanvasElement, typ: &str, fallback: F) -> GlContext {
    get_context_map_else(canvas, typ, |c| GlContext::WebGl1(c), fallback)
}

fn get_context2_else<F: FnOnce() -> GlContext>(canvas: &web_sys::HtmlCanvasElement, typ: &str, fallback: F) -> GlContext {
    get_context_map_else(canvas, typ, |c| GlContext::WebGl2(c), fallback)
}

fn run() -> Option<()> {
    let canvas_element = web_sys::window()?.document()?.get_element_by_id("gameCanvas")?;
    let canvas = canvas_element.dyn_ref::<web_sys::HtmlCanvasElement>()?;
    let _context = get_context2_else(canvas, "webgl2", || get_context1_else(canvas, "webgl", || get_context2_else(canvas, "experimental-webgl2", || get_context1_else(canvas, "experimental-webgl", || GlContext::Nothing))));
    Some({})
}


use anyhow::{anyhow, Result};
use futures::Future;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlImageElement, Response, Window,
};

macro_rules! log {
    ( $( $t:tt)* ) => {
        web_sys::console::log_1(&format!( $($t)* ).into());
    }
}

pub fn window() -> Result<Window> {
    web_sys::window().ok_or_else(|| anyhow!("No Window Found"))
}

pub fn document() -> Result<Document> {
    window()?
        .document()
        .ok_or_else(|| anyhow!("No Document Found"))
}

pub fn canvas() -> Result<HtmlCanvasElement> {
    document()?
        .get_element_by_id("canvas")
        .ok_or_else(|| anyhow!("No Canvas Found"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|element| anyhow!("Error converting {:#?} to HtmlCanvasElement", element))
}

pub fn context() -> Result<CanvasRenderingContext2d> {
    canvas()?
        .get_context("2d")
        .map_err(|js_value| anyhow!("Error getting 2d context {:#?}", js_value))?
        .ok_or_else(|| anyhow!("No 2d Context found"))?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|elem| anyhow!("Error converting {:#?} to CanvasRenderingContext2d", elem))
}

pub fn spawn_local<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

pub async fn fetch_with_str(resource: &str) -> Result<JsValue> {
    JsFuture::from(window()?.fetch_with_str(resource))
        .await
        .map_err(|err| anyhow!("error fetching {:#?}", err))
}

pub async fn fetch_json(json_path: &str) -> Result<JsValue> {
    let resp = fetch_with_str(json_path)
        .await?
        .dyn_into::<Response>()
        .map_err(|elem| anyhow!("Error converting {:#?} to Response", elem))?;
    JsFuture::from(
        resp.json()
            .map_err(|err| anyhow!("Coud not get json: {:#?}", err))?,
    )
    .await
    .map_err(|err| anyhow!("err: {:#?}", err))
}

pub fn new_image() -> Result<HtmlImageElement> {
    web_sys::HtmlImageElement::new()
        .map_err(|err| anyhow!("failed to create new Image element: {:#?}", err))
}

pub fn closure_wrap<T: wasm_bindgen::closure::WasmClosure + ?Sized>(data: Box<T>) -> Closure<T> {
    Closure::wrap(data)
}

pub fn closure_once<F, A, C>(f: F) -> Closure<F::FnMut>
where
    F: 'static + wasm_bindgen::closure::WasmClosureFnOnce<A, C>,
{
    Closure::once(f)
}

pub fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) -> Result<i32> {
    window()?
        .request_animation_frame(f.as_ref().unchecked_ref())
        .map_err(|err| anyhow!("Cannot request animation frame {:#?}", err))
}

pub fn create_raf_closure(f: impl FnMut(f64) + 'static) -> Closure<dyn FnMut(f64)> {
    Closure::wrap(Box::new(f))
}

pub fn now() -> Result<f64> {
    Ok(window()?.performance().ok_or_else(|| anyhow!(""))?.now())
}

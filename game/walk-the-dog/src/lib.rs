use rand::thread_rng;
use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

#[derive(Deserialize)]
struct Cell {
    frame: Rect,
}

#[derive(Deserialize)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    wasm_bindgen_futures::spawn_local(async move {
        let json = fetch_json("rhb.json")
            .await
            .expect("could not fetch rhb.json");
        let sheet: Sheet = json.into_serde().expect("");

        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx.clone());
        let image = web_sys::HtmlImageElement::new().unwrap();
        let ok_callback = Closure::once(move || {
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
                success_tx.send(Ok(())).unwrap();
            }
        });
        let err_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                error_tx.send(Err(err)).unwrap();
            }
        });
        image.set_onload(Some(ok_callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(err_callback.as_ref().unchecked_ref()));
        image.set_src("rhb.png");
        let _ = success_rx.await.unwrap();
        let mut frame = -1;
        let interval_callback = Closure::wrap(Box::new(move || {
            context.clear_rect(0.0, 0.0, 600.0, 600.0);
            frame = (frame + 1) % 300;
            sierpinski(
                &context,
                [
                    (150.0 + (600.0 - frame as f64), 100.0),
                    (0.0 + (600.0 - frame as f64), 400.0),
                    (300.0 + (600.0 - frame as f64), 400.0),
                ],
                (255, 255, 255),
                3,
            );

            let frame_name = format!("Run ({}).png", (frame % 8 + 1));
            let sprite = sheet
                .frames
                .get(frame_name.as_str())
                .expect("Cell not found");
            context
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &image,
                    sprite.frame.x.into(),
                    sprite.frame.y.into(),
                    sprite.frame.w.into(),
                    sprite.frame.h.into(),
                    0.0 + ((frame * 2) % 600) as f64,
                    300.0,
                    sprite.frame.w.into(),
                    sprite.frame.h.into(),
                )
                .unwrap();
        }) as Box<dyn FnMut()>);
        window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                interval_callback.as_ref().unchecked_ref(),
                50,
            )
            .unwrap();
        interval_callback.forget();
    });
    Ok(())
}

async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(json_path)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;
    wasm_bindgen_futures::JsFuture::from(resp.json()?).await
}

fn draw_triangle(
    context: &web_sys::CanvasRenderingContext2d,
    points: [(f64, f64); 3],
    color: (u8, u8, u8),
) {
    let [top, left, right] = points;
    let color_str = format!("rgb({}, {}, {})", color.0, color.1, color.2);
    context.move_to(top.0, top.1);
    context.begin_path();
    context.line_to(left.0, left.1);
    context.line_to(right.0, right.1);
    context.line_to(top.0, top.1);
    context.close_path();
    context.stroke();
    context.set_fill_style(&wasm_bindgen::JsValue::from_str(&color_str));
    context.fill();
}

fn sierpinski(
    context: &web_sys::CanvasRenderingContext2d,
    points: [(f64, f64); 3],
    color: (u8, u8, u8),
    depth: i32,
) {
    if depth == 0 {
        return;
    }
    let depth = depth - 1;
    let [top, left, right] = points;
    draw_triangle(context, [top, left, right], color);
    let a = ((top.0 + left.0) / 2.0, (top.1 + left.1) / 2.0);
    let b = ((right.0 + top.0) / 2.0, (top.1 + right.1) / 2.0);
    let c = ((right.0 + left.0) / 2.0, (right.1 + left.1) / 2.0);
    let next_color = (
        // 255, 255,        255,
        thread_rng().gen_range(0..255),
        thread_rng().gen_range(0..255),
        thread_rng().gen_range(0..255),
    );
    sierpinski(context, [top, a, b], next_color, depth);
    sierpinski(context, [a, left, c], next_color, depth);
    sierpinski(context, [b, c, right], next_color, depth);
}

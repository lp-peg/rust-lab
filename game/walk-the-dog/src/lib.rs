use anyhow::{anyhow, Result};
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

#[macro_use]
mod browser;
mod engine;
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
    let context = browser::context().unwrap();
    browser::spawn_local(async move {
        let sheet: Sheet = browser::fetch_json("rhb.json")
            .await
            .expect("could not fetch rhb.json")
            .into_serde()
            .expect("");
        let image = engine::load_image("rhb.png").await.unwrap();

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
        browser::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                interval_callback.as_ref().unchecked_ref(),
                50,
            )
            .unwrap();
        interval_callback.forget();
    });
    Ok(())
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

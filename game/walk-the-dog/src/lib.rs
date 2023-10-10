use rand::thread_rng;
use rand::Rng;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;
// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
    let starting_point = [(300.0, 0.0), (0.0, 600.0), (600.0, 600.0)];
    sierpinski(&context, starting_point, (255, 255, 255), 6);
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
        thread_rng().gen_range(0..255),
        thread_rng().gen_range(0..255),
        thread_rng().gen_range(0..255),
    );
    sierpinski(context, [top, a, b], next_color, depth);
    sierpinski(context, [a, left, c], next_color, depth);
    sierpinski(context, [b, c, right], next_color, depth);
}

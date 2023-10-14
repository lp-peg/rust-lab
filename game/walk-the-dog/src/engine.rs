use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::channel::oneshot::channel;
use std::{cell::RefCell, rc::Rc, sync::Mutex};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

use crate::browser;

pub async fn load_image(source: &str) -> Result<HtmlImageElement> {
    let image = browser::new_image()?;

    let (tx, rx) = channel::<Result<()>>();
    let success_tx = Rc::new(Mutex::new(Some(tx)));
    let error_tx = Rc::clone(&success_tx);

    let success_callback = browser::closure_once(move || {
        if let Some(tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
            tx.send(Ok(())).unwrap();
        }
    });
    let error_callback = browser::closure_once(move || {
        if let Some(tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
            tx.send(Ok(())).unwrap();
        }
    });

    image.set_onload(Some(success_callback.as_ref().unchecked_ref()));
    image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
    image.set_src(source);
    rx.await??;
    Ok(image)
}

#[async_trait(?Send)]
pub trait Game {
    async fn initialize(&self) -> Result<Box<dyn Game>>;
    fn update(&mut self);
    fn draw(&self, renderer: &Renderer);
}

const FRAME_SIZE: f32 = 1.0 / 60.0 * 1000.0;

pub struct GameLoop {
    last_frame: f64,
    accumulated_delta: f32,
}

impl GameLoop {
    pub async fn start(game: impl Game + 'static) -> Result<()> {
        let mut game = game.initialize().await?;
        let mut game_loop = GameLoop {
            last_frame: browser::now()?,
            accumulated_delta: 0.0,
        };
        let renderer = Renderer {
            context: browser::context()?,
        };
        let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(browser::create_raf_closure(move |pref| {
            game_loop.accumulated_delta += (pref - game_loop.last_frame) as f32;
            while game_loop.accumulated_delta > FRAME_SIZE {
                game.update();
                game_loop.accumulated_delta -= FRAME_SIZE;
            }
            game_loop.last_frame = pref;
            game.draw(&renderer);
            browser::request_animation_frame(f.borrow().as_ref().unwrap()).unwrap();
        }));
        browser::request_animation_frame(g.borrow().as_ref().unwrap())
            .map_err(|err| anyhow!("{:#?}", err))?;
        Ok(())
    }
}

pub struct Renderer {
    context: CanvasRenderingContext2d,
}

impl Renderer {
    pub fn clear(&self, rect: &Rect) {
        self.context
            .clear_rect(rect.x.into(), rect.y.into(), rect.w.into(), rect.h.into());
    }
    pub fn draw_image(&self, image: &HtmlImageElement, frame: &Rect, dest: &Rect) {
        self.context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                frame.x.into(),
                frame.y.into(),
                frame.w.into(),
                frame.h.into(),
                dest.x.into(),
                dest.y.into(),
                dest.x.into(),
                dest.h.into(),
            )
            .unwrap();
    }
}

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

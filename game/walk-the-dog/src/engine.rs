use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::channel::{
    mpsc::{unbounded, UnboundedReceiver},
    oneshot::channel,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Mutex};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlImageElement, KeyboardEvent};

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
    fn update(&mut self, key_state: &KeyState);
    fn draw(&self, renderer: &Renderer);
}

const FRAME_SIZE: f32 = 1.0 / 60.0 * 1000.0;

pub struct GameLoop {
    last_frame: f64,
    accumulated_delta: f32,
}

impl GameLoop {
    pub async fn start(game: impl Game + 'static) -> Result<()> {
        let mut keyboard_event_receiver = prepare_input()?;
        let mut keyboard_state = KeyState::new();

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
            precess_input(&mut keyboard_state, &mut keyboard_event_receiver);
            game_loop.accumulated_delta += (pref - game_loop.last_frame) as f32;
            while game_loop.accumulated_delta > FRAME_SIZE {
                game.update(&keyboard_state);
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
        log!("{}", frame.y);
        self.context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                frame.x.into(),
                frame.y.into(),
                frame.w.into(),
                frame.h.into(),
                dest.x.into(),
                dest.y.into(),
                dest.w.into(),
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

enum KeyPress {
    KeyUp(web_sys::KeyboardEvent),
    KeyDown(web_sys::KeyboardEvent),
}

pub struct KeyState {
    pressed_keys: HashMap<String, web_sys::KeyboardEvent>,
}

impl KeyState {
    fn new() -> Self {
        KeyState {
            pressed_keys: HashMap::new(),
        }
    }
    pub fn is_pressed(&self, code: &str) -> bool {
        self.pressed_keys.contains_key(code)
    }
    pub fn set_pressed(&mut self, code: &str, event: web_sys::KeyboardEvent) {
        self.pressed_keys.insert(code.into(), event);
    }
    pub fn set_released(&mut self, code: &str) {
        self.pressed_keys.remove(code.into());
    }
}

fn prepare_input() -> Result<UnboundedReceiver<KeyPress>> {
    let (sender, receiver) = unbounded::<KeyPress>();
    let down_sender = Rc::new(RefCell::new(sender));
    let up_sender = Rc::clone(&down_sender);

    let on_key_down = browser::closure_wrap(Box::new(move |code: web_sys::KeyboardEvent| {
        down_sender
            .borrow_mut()
            .start_send(KeyPress::KeyDown(code))
            .unwrap();
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    let on_key_up = browser::closure_wrap(Box::new(move |code: web_sys::KeyboardEvent| {
        up_sender
            .borrow_mut()
            .start_send(KeyPress::KeyUp(code))
            .unwrap();
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    browser::canvas()
        .unwrap()
        .set_onkeydown(Some(on_key_down.as_ref().unchecked_ref()));
    browser::canvas()
        .unwrap()
        .set_onkeyup(Some(on_key_up.as_ref().unchecked_ref()));
    on_key_down.forget();
    on_key_up.forget();
    Ok(receiver)
}

fn precess_input(state: &mut KeyState, recv: &mut UnboundedReceiver<KeyPress>) {
    loop {
        match recv.try_next() {
            Ok(None) => break,
            Err(_) => break,
            Ok(Some(event)) => match event {
                KeyPress::KeyDown(e) => state.set_pressed(&e.code(), e),
                KeyPress::KeyUp(e) => state.set_released(&e.code()),
            },
        }
    }
}

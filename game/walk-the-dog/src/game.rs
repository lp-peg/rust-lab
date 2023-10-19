use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use web_sys::HtmlImageElement;

use crate::{
    browser,
    engine::{self, Point, Rect, Renderer},
};

#[derive(Deserialize, Clone)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

#[derive(Deserialize, Clone)]
struct Cell {
    frame: SheetRect,
}

#[derive(Deserialize, Clone)]
struct SheetRect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

const FLOOR: i16 = 475;

pub struct WalkTheDog {
    rhb: Option<RedHatBoy>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog { rhb: None }
    }
}

#[async_trait(?Send)]
impl engine::Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn engine::Game>> {
        let sheet: Option<Sheet> = browser::fetch_json("rhb.json").await?.into_serde()?;
        let image = Some(engine::load_image("rhb.png").await?);
        Ok(Box::new(WalkTheDog {
            rhb: Some(RedHatBoy::new(
                sheet.clone().ok_or_else(|| anyhow!("No Sheet Present"))?,
                image.clone().ok_or_else(|| anyhow!("No Image Present"))?,
            )),
        }))
    }
    fn update(&mut self, key_state: &engine::KeyState) {
        const SPEED: i16 = 5;
        let mut velocity = Point::zero();
        if key_state.is_down() {
            velocity.y += SPEED;
            self.rhb.as_mut().unwrap().reset();
        }
        if key_state.is_up() {
            velocity.y -= SPEED;
        }
        if key_state.is_right() {
            velocity.x += SPEED;
            self.rhb.as_mut().unwrap().run_right();
        }
        if key_state.is_left() {
            velocity.x -= SPEED;
        }
        self.rhb.as_mut().unwrap().update();
    }
    fn draw(&self, renderer: &engine::Renderer) {
        renderer.clear(&Rect {
            x: 0.0,
            y: 0.0,
            w: 600.0,
            h: 600.0,
        });
        self.rhb.as_ref().unwrap().draw(renderer)
    }
}

struct RedHatBoy {
    state_machine: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}

impl RedHatBoy {
    fn new(sheet: Sheet, image: HtmlImageElement) -> Self {
        RedHatBoy {
            state_machine: RedHatBoyStateMachine::Idle(RedHatBoyState::new()),
            sprite_sheet: sheet,
            image: image,
        }
    }
    fn draw(&self, renderer: &Renderer) {
        let frame_name = format!(
            "{} ({}).png",
            self.state_machine.frame_name(),
            (self.state_machine.context().frame / 3) + 1
        );
        let sprite = self.sprite_sheet.frames.get(&frame_name).unwrap();
        renderer.draw_image(
            &self.image,
            &Rect {
                x: sprite.frame.x.into(),
                y: sprite.frame.y.into(),
                w: sprite.frame.w.into(),
                h: sprite.frame.h.into(),
            },
            &Rect {
                x: self.state_machine.context().position.x.into(),
                y: self.state_machine.context().position.y.into(),
                w: sprite.frame.w.into(),
                h: sprite.frame.h.into(),
            },
        )
    }
    fn update(&mut self) {
        self.state_machine = self.state_machine.update();
    }
    fn reset(&mut self) {
        self.state_machine = RedHatBoyStateMachine::Idle(RedHatBoyState::new());
    }
    fn run_right(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Run);
    }
}

#[derive(Clone, Copy)]
struct RedHatBoyState<T> {
    context: RedHatBoyContext,
    _state: T, //未使用であることを表す。
}

impl RedHatBoyState<Idle> {
    fn new() -> Self {
        RedHatBoyState {
            context: RedHatBoyContext {
                frame: 0,
                position: Point { x: 0, y: FLOOR },
                velocity: Point { x: 0, y: 0 },
            },
            _state: Idle {},
        }
    }
}

#[derive(Clone, Copy)]
struct RedHatBoyContext {
    frame: u8,
    position: Point,
    velocity: Point,
}

impl RedHatBoyContext {
    fn update(mut self, frames: u8) -> Self {
        self.frame = (self.frame + 1) % frames;
        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;
        return self;
    }
    fn reset_frame(mut self) -> Self {
        self.frame = 0;
        return self;
    }
    fn reset_position(mut self) -> Self {
        self.position.x = 0;
        self.position.y = 0;
        return self;
    }
    fn run_right(mut self) -> Self {
        const SPEED: i16 = 3;
        self.velocity.x += SPEED;
        return self;
    }
}

// 異なる「状態を表す構造体」を enum でラップする。
#[derive(Clone, Copy)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
}

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            // 現在の状態が idle で、run イベントが渡ってきた場合
            (RedHatBoyStateMachine::Idle(state), Event::Run) => state.run().into(),
            _ => self,
        }
    }
    fn frame_name(&self) -> &str {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.frame_name(),
            RedHatBoyStateMachine::Running(state) => state.frame_name(),
        }
    }
    fn context(&self) -> &RedHatBoyContext {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.context(),
            RedHatBoyStateMachine::Running(state) => state.context(),
        }
    }
    fn update(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(mut state) => {
                state.update();
                RedHatBoyStateMachine::Idle(state)
            }
            RedHatBoyStateMachine::Running(mut state) => {
                state.update();
                RedHatBoyStateMachine::Running(state)
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Idle;

#[derive(Clone, Copy)]
struct Running;

enum Event {
    Run,
}

impl RedHatBoyState<Idle> {
    pub fn run(self) -> RedHatBoyState<Running> {
        RedHatBoyState {
            context: self.context.reset_frame().run_right(),
            _state: Running,
        }
    }
    pub fn frame_name(&self) -> &str {
        "Idle"
    }
    pub fn update(&mut self) {
        // &mut じゃないと、Copy トレイトによって値コピーになり、変更が反映されない。
        self.context = self.context().update(30);
    }
}

impl RedHatBoyState<Running> {
    pub fn frame_name(&self) -> &str {
        "Run"
    }
    pub fn update(&mut self) {
        self.context = self.context().update(24);
    }
}

impl<S> RedHatBoyState<S> {
    pub fn context(&self) -> &RedHatBoyContext {
        &self.context
    }
}

impl From<RedHatBoyState<Running>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Running>) -> RedHatBoyStateMachine {
        RedHatBoyStateMachine::Running(state)
    }
}

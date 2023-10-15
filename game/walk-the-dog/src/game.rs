use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use web_sys::HtmlImageElement;

use crate::{
    browser,
    engine::{self, Point},
};

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

#[derive(Deserialize)]
struct Cell {
    frame: SheetRect,
}

#[derive(Deserialize)]
struct SheetRect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

pub struct WalkTheDog {
    frame: u8,
    sheet: Option<Sheet>,
    image: Option<HtmlImageElement>,
    position: Point,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog {
            frame: 0,
            sheet: None,
            image: None,
            position: engine::Point { x: 0, y: 0 },
        }
    }
}

#[async_trait(?Send)]
impl engine::Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn engine::Game>> {
        let sheet: Sheet = browser::fetch_json("rhb.json").await?.into_serde()?;
        let image = engine::load_image("rhb.png").await?;
        Ok(Box::new(WalkTheDog {
            frame: self.frame,
            sheet: Some(sheet),
            image: Some(image),
            position: self.position,
        }))
    }
    fn update(&mut self, key_state: &engine::KeyState) {
        const FRAMES: u8 = 24;
        self.frame = (self.frame + 1) % FRAMES;
        const SPEED: i16 = 5;
        let mut velocity = Point::zero();
        if key_state.is_pressed("ArrowDown") {
            velocity.y += SPEED;
        }
        if key_state.is_pressed("ArrowUp") {
            velocity.y -= SPEED;
        }
        if key_state.is_pressed("ArrowRight") {
            velocity.x += SPEED;
        }
        if key_state.is_pressed("ArrowLeft") {
            velocity.x -= SPEED;
        }
        self.position.x += velocity.x;
        self.position.y += velocity.y;
    }
    fn draw(&self, renderer: &engine::Renderer) {
        let current_sprite = (self.frame / 3) + 1;
        let frame_name = format!("Run ({}).png", current_sprite);
        let sprite = self
            .sheet
            .as_ref()
            .and_then(|sheet| sheet.frames.get(&frame_name))
            .unwrap();
        renderer.clear(&engine::Rect {
            x: 0.0,
            y: 0.0,
            w: 600.0,
            h: 600.0,
        });
        self.image.as_ref().map(|image| {
            renderer.draw_image(
                &image,
                &engine::Rect {
                    x: sprite.frame.x.into(),
                    y: sprite.frame.y.into(),
                    w: sprite.frame.w.into(),
                    h: sprite.frame.h.into(),
                },
                &engine::Rect {
                    x: self.position.x.into(),
                    y: self.position.y.into(),
                    w: sprite.frame.w.into(),
                    h: sprite.frame.h.into(),
                },
            )
        });
    }
}

use std::collections::{HashMap, HashSet};

use image::Pixel;
use nannou::prelude::*;

use super::Tile;

#[derive(Clone)]
pub struct Cell {
    pub options: HashSet<usize>,
}

impl Cell {
    pub fn new(options: HashSet<usize>) -> Cell {
        Cell { options }
    }

    pub fn draw(&self, draw: &Draw, tiles: &[Tile], x: u32, y: u32, width: f32) {
        if self.options.len() == 1 {
            let chosen = self
                .options
                .iter()
                .copied()
                .next()
                .expect("There will be a value available for chosen index.");
            let image = &tiles[chosen].image;
            let center_pixel = image.get_pixel(1, 1);
            let [r, g, b] = center_pixel.channels() else {
                panic!("Wrong channel count!")
            };
            draw.rect()
                .x_y((x as f32 + 0.5) * width, (y as f32 + 0.5) * width)
                .w_h(width, width)
                .color(Rgb::from_components((*r, *g, *b)))
                .stroke_weight((width * 0.03).max(2.));
        } else {
            draw.rect()
                .x_y((x as f32 + 0.5) * width, (y as f32 + 0.5) * width)
                .w_h(width, width)
                .color(Rgb::from_components((100u8, 100u8, 100u8)))
                .stroke_weight((width * 0.03).max(2.));
        }
    }

    pub fn update_options(&mut self, available_options: &HashSet<usize>) -> Result<(), Exhausted> {
        let new_options = &self.options & available_options;
        if new_options.is_empty() {
            Err(Exhausted)
        } else {
            self.options = new_options;
            Ok(())
        }
    }
}

pub struct Exhausted;

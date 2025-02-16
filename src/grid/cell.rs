use std::collections::HashSet;

use nannou::prelude::*;

use crate::utils;

use super::Tile;

#[derive(Clone)]
pub struct Cell {
    pub options: HashSet<usize>,
    pub chosen: Option<usize>,
}

impl Cell {
    pub fn new(options: usize) -> Cell {
        Cell {
            options: (0..options).collect(),
            chosen: None,
        }
    }

    pub fn draw(&self, draw: &Draw, tiles: &[Tile], x: u32, y: u32, width: f32) {
        if let Some(chosen) = self.chosen {
            let image = &tiles[chosen].image;
            utils::draw_tile(draw, image, x, y, width);
        } else {
            draw.rect()
                .x_y((x as f32 + 0.5) * width, (y as f32 + 0.5) * width)
                .w_h(width, width)
                .color(Rgb::from_components((100u8, 100u8, 100u8)))
                .stroke_weight((width * 0.03).max(2.));
        }
    }
}

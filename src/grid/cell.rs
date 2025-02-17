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

    pub fn draw(
        &self,
        draw: &Draw,
        tiles: &[Tile],
        x: u32,
        y: u32,
        width: f32,
        weights: &HashMap<usize, u32>,
    ) {
        if self.options.is_empty() {
            draw.rect()
                .x_y((x as f32 + 0.5) * width, (y as f32 + 0.5) * width)
                .w_h(width, width)
                .color(Rgb::from_components((255u8, 0u8, 100u8)))
                .stroke_weight((width * 0.03).max(2.));
        } else {
            let mut sum_r = 0;
            let mut sum_g = 0;
            let mut sum_b = 0;
            let mut count = 0;
            for tile_index in self.options.iter() {
                let image = &tiles[*tile_index].image;
                let weight = weights[tile_index];
                let center_pixel = image.get_pixel(1, 1);
                let [r, g, b] = center_pixel.channels() else {
                    panic!("Wrong channel count!")
                };
                sum_r += *r as u32 * weight;
                sum_g += *g as u32 * weight;
                sum_b += *b as u32 * weight;
                count += weight;
            }
            sum_r /= count;
            sum_g /= count;
            sum_b /= count;
            draw.rect()
                .x_y((x as f32 + 0.5) * width, (y as f32 + 0.5) * width)
                .w_h(width, width)
                .color(Rgb::from_components((
                    sum_r as u8,
                    sum_g as u8,
                    sum_b as u8,
                )));
            // .stroke_weight((width * 0.03).max(2.));
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

use image::{GenericImageView, Pixel, RgbImage};

use nannou::prelude::*;

pub fn draw_tile(draw: &Draw, tile: &RgbImage, x_outer: u32, y_outer: u32, tile_width: f32) {
    let tile_size = tile.width();
    let cell_width = tile_width / tile_size as f32;
    for (x_inner, y_inner, pixel) in GenericImageView::pixels(tile) {
        let [r, g, b] = pixel.channels() else {
            panic!("Wrong channel count!")
        };
        draw.rect()
            .x_y(
                (x_outer as f32) * tile_width + (x_inner as f32 + 0.5) * cell_width,
                (y_outer as f32) * tile_width + (y_inner as f32 + 0.5) * cell_width,
            )
            .w_h(cell_width, cell_width)
            .color(Rgb::from_components((*r, *g, *b)))
            .stroke_weight((tile_width * 0.03).max(2.));
    }
    draw.rect()
        .x_y(
            (x_outer as f32 + 0.5) * tile_width,
            (y_outer as f32 + 0.5) * tile_width,
        )
        .w_h(cell_width * tile_size as f32, cell_width * tile_size as f32)
        .no_fill()
        .stroke_weight((cell_width * 0.06).max(4.))
        .stroke_color(DARKCYAN);
}

use std::{path::PathBuf, time::Duration};

use clap::Parser;
use nannou::{
    image::{self, GenericImageView, Pixel, RgbImage},
    prelude::*,
};
use ouroboros::self_referencing;

mod image_impls;
use image_impls::{LoopingSubImage, Tilable};

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    tile_size: Option<u32>,
}

#[self_referencing]
struct Model {
    image: RgbImage,
    // index: usize,
    // last_change: Duration,
    tile_size: u32,

    #[borrows(image)]
    #[covariant]
    tiles: Vec<LoopingSubImage<&'this RgbImage>>,
}

fn model(_app: &App) -> Model {
    let args = Args::parse();
    let tile_size = args.tile_size.unwrap_or(3);
    ModelBuilder {
        image: image::open(args.input).unwrap().into_rgb8(),
        // index: 0,
        // last_change: Duration::default(),
        tile_size,

        tiles_builder: |image_data: &RgbImage| image_data.looping_tiles(tile_size),
    }
    .build()
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // if update.since_start - *model.borrow_last_change() > Duration::from_millis(500) {
    //     model.with_mut(|fields| {
    //         *fields.last_change = update.since_start;
    //         *fields.index = (*fields.index + 1) % fields.tiles.len();
    //     })
    // }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(PLUM);
    let image = model.borrow_image();
    let tiles = model.borrow_tiles();
    let win = app.window_rect();
    let (frame_width, frame_height) = win.w_h();
    let tile_size = model.borrow_tile_size();
    let tile_count = (*tile_size * image.width()) as f32;
    let (pixel_width, pixel_height) = (frame_width / tile_count, frame_height / tile_count);
    let draw = draw
        .translate(Vec3::new(
            (pixel_width - frame_width) / 2.,
            (frame_height - pixel_height) / 2.,
            0.,
        ))
        .scale_y(-1.);
    for (i, tile) in tiles.iter().enumerate() {
        let x_outer = (i as u32 % image.width()) * tile_size;
        let y_outer = (i as u32 / image.width()) * tile_size;
        for (x_inner, y_inner, pixel) in tile.pixels() {
            draw.rect()
                .x_y(
                    (x_outer + x_inner) as f32 * pixel_width,
                    (y_outer + y_inner) as f32 * pixel_height,
                )
                .w_h(pixel_width, pixel_height)
                .color(Rgba::from_components(pixel.channels4()))
                .stroke_weight((pixel_width * 0.03).max(2.));
        }
        draw.rect()
            .x_y(
                (x_outer as f32 + 1.) * pixel_width,
                (y_outer as f32 + 1.) * pixel_height,
            )
            .w_h(
                pixel_width * *tile_size as f32,
                pixel_height * *tile_size as f32,
            )
            .no_fill()
            .stroke_weight((pixel_width * 0.06).max(4.))
            .stroke_color(DARKCYAN);
    }
    draw.to_frame(app, &frame).unwrap();
}

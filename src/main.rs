use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use image::{self, EncodableLayout, ExtendedColorType, ImageBuffer};
use nannou::prelude::*;
use rand::rngs::ThreadRng;

mod args;
mod grid;
mod image_impls;
mod tile;

use args::Args;
use grid::{cell::Exhausted, Grid};
use image_impls::Tilable;
use tile::Tile;

fn main() {
    let args = Args::parse();
    if args.display {
        nannou::app(|_| model())
            .update(|_, model, _| update(model))
            .simple_window(view)
            .run();
    } else {
        let mut model = model();
        loop {
            update(&mut model);
            if !model.collapsing {
                return;
            }
        }
    }
}

struct Model {
    tiles: Vec<Tile>,
    grid: Grid,
    rng: ThreadRng,
    collapsing: bool,
    output: Option<PathBuf>,
}

fn model() -> Model {
    //_app: &App
    let args = Args::parse();
    let tile_size = args.tile_size;
    if tile_size % 2 != 1 {
        panic!("tile size must be odd")
    }
    let image = image::open(args.input).unwrap().into_rgb8();
    let tiles: Vec<Tile> = image
        .tiles(tile_size)
        .map(|tile_view| Tile {
            image: tile_view.to_image(),
            neighbors: Default::default(),
        })
        .collect();

    let mut weights: HashMap<usize, u32> = HashMap::new();
    for (i, outer_tile) in tiles.iter().enumerate() {
        let outer_up_view = outer_tile.up_view();
        let outer_down_view = outer_tile.down_view();
        let outer_left_view = outer_tile.left_view();
        let outer_right_view = outer_tile.right_view();
        let mut neighbors = outer_tile.neighbors.borrow_mut();
        let real_i = tiles
            .iter()
            .enumerate()
            .find(|(_, tile)| tile.image == outer_tile.image)
            .expect("Will find itself if nothing else")
            .0;
        let original = real_i == i;
        *weights.entry(real_i).or_default() += 1;
        if original {
            if outer_up_view == outer_down_view {
                neighbors.up.insert(i);
                neighbors.down.insert(i);
            }
            if outer_right_view == outer_left_view {
                neighbors.right.insert(i);
                neighbors.left.insert(i);
            }
        }
        for (j, tile) in tiles.iter().enumerate().skip(i + 1) {
            let inner_up_view = tile.up_view();
            let inner_down_view = tile.down_view();
            let inner_left_view = tile.left_view();
            let inner_right_view = tile.right_view();
            let mut inner_neighbors = tile.neighbors.borrow_mut();
            if inner_down_view == outer_up_view {
                neighbors.up.insert(j);
                inner_neighbors.down.insert(real_i);
            }
            if inner_up_view == outer_down_view {
                neighbors.down.insert(j);
                inner_neighbors.up.insert(real_i);
            }
            if inner_left_view == outer_right_view {
                neighbors.right.insert(j);
                inner_neighbors.left.insert(real_i);
            }
            if inner_right_view == outer_left_view {
                neighbors.left.insert(j);
                inner_neighbors.right.insert(real_i);
            }
        }
    }
    Model {
        grid: Grid::new(args.output_width, args.output_height, weights),
        tiles,
        rng: rand::rng(),
        collapsing: true,
        output: args.output,
    }
}

fn update(model: &mut Model) {
    if model.collapsing {
        let result = model.grid.collapse(&model.tiles, &mut model.rng);
        match result {
            Ok(collapsing) => model.collapsing = collapsing,
            Err(Exhausted) => {
                model.grid.regenerate();
            }
        }
        if !model.collapsing {
            println!("Collapsing finished");
            if let Some(output) = &model.output {
                let grid = &model.grid;
                let image_buffer =
                    ImageBuffer::from_fn(grid.width() as u32, grid.height() as u32, |x, y| {
                        grid.get((x as usize, y as usize))
                            .map(|cell| {
                                let tile_index = cell.options.iter().next().expect(
                                    "Finished collapse must mean all cells have one option",
                                );
                                let image = &model.tiles[*tile_index].image;
                                *image.get_pixel(image.width() / 2, image.height() / 2)
                            })
                            .expect("All tiles will have a center pixel")
                    });
                let _ = image::save_buffer(
                    output,
                    image_buffer.as_bytes(),
                    image_buffer.width(),
                    image_buffer.height(),
                    ExtendedColorType::Rgb8,
                );
            }
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(PLUM);
    let grid = &model.grid;
    let tiles = &model.tiles;
    let win = app.window_rect();
    let (frame_width, frame_height) = win.w_h();
    let tile_width = frame_width / (grid.width() as u32) as f32;
    let draw = draw
        .translate(Vec3::new(-frame_width / 2., frame_height / 2., 0.))
        .scale_y(-1.);
    for (x, y, cell) in grid.cells() {
        cell.draw(&draw, tiles, x as u32, y as u32, tile_width);
    }
    draw.to_frame(app, &frame).unwrap();
}

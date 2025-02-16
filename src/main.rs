use core::time::{self, Duration};

use clap::Parser;
use image::{self};
use nannou::prelude::*;
use rand::rngs::ThreadRng;

mod args;
mod grid;
mod image_impls;
mod tile;
mod utils;

use args::Args;
use grid::Grid;
use image_impls::Tilable;
use tile::Tile;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    tiles: Vec<Tile>,
    grid: Grid,
    rng: ThreadRng,
    collapsing: bool,
    last_update: Duration,
}

fn model(_app: &App) -> Model {
    let args = Args::parse();
    let tile_size = args.tile_size;
    let image = image::open(args.input).unwrap().into_rgb8();
    let tiles: Vec<Tile> = image
        .tiles(tile_size)
        .map(|tile_view| Tile {
            image: tile_view.to_image(),
            neighbors: Default::default(),
        })
        .collect();

    for (i, outer_tile) in tiles.iter().enumerate() {
        let outer_up_view = outer_tile.up_view();
        let outer_down_view = outer_tile.down_view();
        let outer_left_view = outer_tile.left_view();
        let outer_right_view = outer_tile.right_view();
        let mut neighbors = outer_tile.neighbors.borrow_mut();
        if outer_up_view == outer_down_view {
            neighbors.up.push(i);
            neighbors.down.push(i);
        }
        if outer_right_view == outer_left_view {
            neighbors.right.push(i);
            neighbors.left.push(i);
        }
        for (j, tile) in tiles.iter().enumerate().skip(i + 1) {
            let inner_up_view = tile.up_view();
            let inner_down_view = tile.down_view();
            let inner_left_view = tile.left_view();
            let inner_right_view = tile.right_view();
            let mut inner_neighbors = tile.neighbors.borrow_mut();
            if inner_down_view == outer_up_view {
                neighbors.up.push(j);
                inner_neighbors.down.push(i);
            }
            if inner_up_view == outer_down_view {
                neighbors.down.push(j);
                inner_neighbors.up.push(i);
            }
            if inner_left_view == outer_right_view {
                neighbors.right.push(j);
                inner_neighbors.left.push(i);
            }
            if inner_right_view == outer_left_view {
                neighbors.left.push(j);
                inner_neighbors.right.push(i);
            }
        }
    }
    Model {
        grid: Grid::new(args.output_width, args.output_height, tiles.len()),
        tiles,
        rng: rand::rng(),
        collapsing: true,
        last_update: Duration::from_secs(0),
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    if model.collapsing && (update.since_start - model.last_update) > Duration::from_millis(500) {
        model.last_update = update.since_start;
        model.collapsing = model.grid.collapse(&model.tiles, &mut model.rng);
        if !model.collapsing {
            println!("Collapsing finished");
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

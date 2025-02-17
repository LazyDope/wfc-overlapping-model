use std::path::PathBuf;

use clap::Parser;
use image::{self, EncodableLayout, ExtendedColorType, ImageBuffer};
use nannou::prelude::*;
use rand::{rngs::ThreadRng, Rng};

mod args;
mod grid;
mod image_impls;
mod tile;

use args::Args;
use grid::{cell::Exhausted, Grid};
use image_impls::Tilable;
use tile::{Direction, Tile};

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

struct Model<T> {
    tiles: Vec<Tile>,
    grid: Grid,
    rng: T,
    collapsing: bool,
    repeating: bool,
    output: Option<PathBuf>,
}

fn model() -> Model<ThreadRng> {
    let args = Args::parse();
    let tile_size = args.tile_size;
    if tile_size % 2 != 1 {
        panic!("tile size must be odd")
    }
    let image = image::open(args.input).unwrap().into_rgb8();
    let border_style = args.border_style;
    let mut tiles: Vec<Tile> = image
        .tiles(tile_size, border_style)
        .map(|tile_view| Tile {
            image: tile_view.to_image(),
            neighbors: Default::default(),
            frequency: 0,
            border_style,
        })
        .collect();

    for outer_index in 0..tiles.len() {
        let outer_tile = &tiles[outer_index];
        let original_index = tiles
            .iter()
            .enumerate()
            .find(|(_, tile)| tile.image == outer_tile.image)
            .expect("Will find itself if nothing else")
            .0;
        let original = original_index == outer_index;
        tiles[original_index].frequency += 1;
        let original_tile = &tiles[original_index];
        let mut neighbors = original_tile.neighbors.borrow_mut();
        if original {
            for dir in [Direction::Up, Direction::Right] {
                let opp_dir = dir.opposing();
                if original_tile.view_in_direction(dir) == original_tile.view_in_direction(opp_dir)
                {
                    neighbors[dir].insert(outer_index);
                    neighbors[opp_dir].insert(outer_index);
                }
            }
        }
        for (inner_index, inner_tile) in tiles.iter().enumerate().skip(outer_index + 1) {
            let mut inner_neighbors = inner_tile.neighbors.borrow_mut();
            for dir in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ] {
                let opp_dir = dir.opposing();
                let outer_view = original_tile.view_in_direction(dir);
                let inner_view = inner_tile.view_in_direction(opp_dir);
                if inner_view == outer_view {
                    if original {
                        neighbors[dir].insert(inner_index);
                    }
                    inner_neighbors[opp_dir].insert(original_index);
                }
            }
        }
    }
    let options = tiles
        .iter()
        .enumerate()
        .filter(|(_, tile)| tile.frequency != 0)
        .map(|(i, _)| i)
        .collect();
    Model {
        grid: Grid::new(
            args.output_width,
            args.output_height.unwrap_or(args.output_width),
            options,
            args.max_depth,
        ),
        tiles,
        rng: rand::rng(),
        collapsing: true,
        repeating: args.repeating,
        output: args.output,
    }
}

fn update<T: Rng>(model: &mut Model<T>) {
    if model.collapsing {
        let result = model.grid.collapse(&model.tiles, &mut model.rng);
        match result {
            Ok(true) => (),
            Ok(false) => {
                if model.repeating {
                    model.grid.regenerate(false)
                } else {
                    model.collapsing = false
                }
            }
            Err(Exhausted) => {
                model.grid.regenerate(true);
            }
        }
        if !model.collapsing && result.is_ok() {
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

fn view<T>(app: &App, model: &Model<T>, frame: Frame) {
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

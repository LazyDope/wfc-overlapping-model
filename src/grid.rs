use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use rand::prelude::*;

pub mod cell;

use crate::tile::Tile;
use cell::Cell;

#[derive(Clone)]
pub struct Grid {
    buf: Vec<Cell>,
    height: usize,
    width: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize, options: usize) -> Grid {
        let buf = (0..(width * height)).map(|_| Cell::new(options)).collect();
        Self { buf, height, width }
    }

    pub fn get(&self, index: (usize, usize)) -> Option<&Cell> {
        if index.0 >= self.width || index.1 >= self.height {
            return None;
        }
        Some(&self.buf[index.0 + index.1 * self.width])
    }

    pub fn get_index(&self, index: usize) -> Option<&Cell> {
        if index >= self.buf.len() {
            return None;
        }
        Some(&self.buf[index])
    }

    pub fn get_mut(&mut self, index: (usize, usize)) -> Option<&mut Cell> {
        if index.0 >= self.width || index.1 >= self.height {
            return None;
        }
        Some(&mut self.buf[index.0 + index.1 * self.width])
    }

    pub fn get_index_mut(&mut self, index: usize) -> Option<&mut Cell> {
        if index >= self.buf.len() {
            return None;
        }
        Some(&mut self.buf[index])
    }

    pub fn height(&self) -> usize {
        self.height
    }
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn collapse(&mut self, tiles: &[Tile], rng: &mut ThreadRng) -> bool {
        let width = self.width();

        let index = {
            let mut cells: Vec<(usize, &mut Cell)> = self
                .buf
                .iter_mut()
                .enumerate()
                .filter(|(_, cell)| cell.chosen.is_none())
                .collect();

            if cells.is_empty() {
                return false;
            }
            let result = cells.len() == 1;
            fn calc_entropy(tiles: &[Tile], cell: &Cell) -> f64 {
                let mut tile_counts = HashMap::new();
                for tile_index in cell.options.iter() {
                    *tile_counts
                        .entry(tiles[*tile_index].image.clone())
                        .or_default() += 1;
                }
                -tile_counts
                    .into_values()
                    .map(|count: u32| {
                        let p = count as f64 / cell.options.len() as f64;
                        p * p.log2()
                    })
                    .sum::<f64>()
            }

            let mut min = f64::MAX;
            let mut min_i = None;
            for (i, (_, cell)) in cells.iter_mut().enumerate() {
                let entropy = calc_entropy(tiles, cell);
                if entropy < min {
                    min = entropy;
                    min_i = Some(i);
                }
            }

            let (index, cell) = &mut cells[min_i.expect("No cells in grid")];
            println!("Collapsing {index}");
            if let Some(chosen) = cell.options.drain().choose(rng) {
                cell.chosen = Some(chosen)
            } else {
                return false;
            };
            if result {
                return false;
            }
            *index
        };
        if let Some(up_cell) = index
            .checked_sub(width)
            .and_then(|index| self.get_index_mut(index))
        {
            up_cell.options =
                &up_cell.options & &tiles[index].neighbors.borrow().up.iter().copied().collect();
        }
        if let Some(down_cell) = index
            .checked_add(width)
            .and_then(|index| self.get_index_mut(index))
        {
            down_cell.options = &down_cell.options
                & &tiles[index]
                    .neighbors
                    .borrow()
                    .down
                    .iter()
                    .copied()
                    .collect();
        }
        if let Some(left_cell) = index
            .checked_sub(1)
            .and_then(|index| self.get_index_mut(index))
        {
            left_cell.options = &left_cell.options
                & &tiles[index]
                    .neighbors
                    .borrow()
                    .left
                    .iter()
                    .copied()
                    .collect();
        }
        if let Some(right_cell) = index
            .checked_add(1)
            .and_then(|index| self.get_index_mut(index))
        {
            right_cell.options = &right_cell.options
                & &tiles[index]
                    .neighbors
                    .borrow()
                    .right
                    .iter()
                    .copied()
                    .collect();
        }
        true
    }

    pub fn cells(&self) -> Cells<'_> {
        Cells { grid: self, i: 0 }
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = Cell;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.get(index).expect("Index out of range")
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.get_mut(index).expect("Index out of range")
    }
}

pub struct Cells<'grid> {
    grid: &'grid Grid,
    i: usize,
}

impl<'grid> Iterator for Cells<'grid> {
    type Item = (usize, usize, &'grid Cell);

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.i % self.grid.width();
        let y = self.i / self.grid.width();
        self.i += 1;
        self.grid.get((x, y)).map(|cell| (x, y, cell))
    }
}

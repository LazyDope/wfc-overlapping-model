use std::{
    collections::HashMap,
    iter,
    ops::{Index, IndexMut},
};

use rand::prelude::*;

pub mod cell;

use crate::tile::Tile;
use cell::{Cell, Exhausted};

#[derive(Clone)]
pub struct Grid {
    buf: Vec<Cell>,
    height: usize,
    width: usize,
    options: HashMap<usize, u32>,
}

impl Grid {
    pub fn new(width: usize, height: usize, options: HashMap<usize, u32>) -> Grid {
        let buf = (0..(width * height))
            .map(|_| Cell::new(options.clone()))
            .collect();
        Self {
            buf,
            height,
            width,
            options,
        }
    }

    pub fn regenerate(&mut self) {
        self.buf = (0..(self.width * self.height))
            .map(|_| Cell::new(self.options.clone()))
            .collect();
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

    pub fn collapse(&mut self, tiles: &[Tile], rng: &mut ThreadRng) -> Result<bool, Exhausted> {
        let (grid_index, options) = {
            let mut cells: Vec<(usize, &mut Cell)> = self
                .buf
                .iter_mut()
                .enumerate()
                .filter(|(_, cell)| cell.options.len() > 1)
                .collect();

            if cells.is_empty() {
                return Ok(false);
            }
            let result = cells.len() == 1;
            fn calc_entropy(tiles: &[Tile], cell: &Cell) -> f64 {
                let mut tile_counts = vec![0u32; tiles.len()];
                let mut total = 0;
                for (tile_index, count) in cell.options.iter() {
                    tile_counts[*tile_index] += count;
                    total += count;
                }
                -tile_counts
                    .into_iter()
                    .filter(|&v| v != 0)
                    .map(|count: u32| {
                        let p = count as f64 / total as f64;
                        p * p.log2()
                    })
                    .sum::<f64>()
            }

            let mut min = f64::MAX;
            let mut min_indexes = Vec::new();
            for (i, (_, cell)) in cells.iter().enumerate() {
                let entropy = calc_entropy(tiles, cell);
                if entropy == min {
                    min_indexes.push(i);
                } else if entropy < min {
                    min = entropy;
                    min_indexes = vec![i];
                }
            }

            let (index, cell) = &mut cells[*min_indexes.choose(rng).expect("No cells in grid")];
            if let Some(chosen_index) = cell
                .options
                .drain()
                .flat_map(|(index, count)| iter::repeat_n(index, count as usize))
                .choose(rng)
            {
                cell.options.insert(chosen_index, 1);
                if result {
                    return Ok(false);
                }
                (*index, cell.options.clone())
            } else {
                println!("Failed to collapse {index}");
                return Err(Exhausted);
            }
        };
        self.update_neighbors(tiles, grid_index, options, 0)?;
        Ok(true)
    }

    pub fn cells(&self) -> Cells<'_> {
        Cells { grid: self, i: 0 }
    }

    fn update_neighbors(
        &mut self,
        tiles: &[Tile],
        grid_index: usize,
        options: HashMap<usize, u32>,
        mut depth: usize,
    ) -> Result<(), Exhausted> {
        if depth > 5 {
            return Ok(());
        }
        depth += 1;
        let width = self.width();
        let available_indexes = &options;
        if let Some((up_index, up_cell)) = grid_index
            .checked_sub(width)
            .and_then(|index| self.get_index_mut(index).map(|cell| (index, cell)))
            .take_if(|(_, up_cell)| up_cell.options.len() != 1)
        {
            let count_before = up_cell.options.len();
            let mut available_options = HashMap::with_capacity(tiles.len());
            for tile_index in available_indexes.keys() {
                available_options.extend(tiles[*tile_index].neighbors.borrow().up.iter());
            }
            up_cell.update_options(&available_options)?;
            if count_before != up_cell.options.len() {
                let options = up_cell.options.clone();
                self.update_neighbors(tiles, up_index, options, depth)?;
            }
        }
        if let Some((down_index, down_cell)) = grid_index
            .checked_add(width)
            .and_then(|index| self.get_index_mut(index).map(|cell| (index, cell)))
            .take_if(|(_, down_cell)| down_cell.options.len() != 1)
        {
            let count_before = down_cell.options.len();
            let mut available_options = HashMap::with_capacity(tiles.len());
            for tile_index in available_indexes.keys() {
                available_options.extend(tiles[*tile_index].neighbors.borrow().down.iter());
            }
            down_cell.update_options(&available_options)?;
            if count_before != down_cell.options.len() {
                let options = down_cell.options.clone();
                self.update_neighbors(tiles, down_index, options, depth)?;
            }
        }
        if let Some((left_index, left_cell)) = grid_index
            .checked_sub(1)
            .and_then(|index| {
                if index % self.width() != (self.width() - 1) {
                    self.get_index_mut(index).map(|cell| (index, cell))
                } else {
                    None
                }
            })
            .take_if(|(_, left_cell)| left_cell.options.len() != 1)
        {
            let count_before = left_cell.options.len();
            let mut available_options = HashMap::with_capacity(tiles.len());
            for tile_index in available_indexes.keys() {
                available_options.extend(tiles[*tile_index].neighbors.borrow().left.iter());
            }
            left_cell.update_options(&available_options)?;
            if count_before != left_cell.options.len() {
                let options = left_cell.options.clone();
                self.update_neighbors(tiles, left_index, options, depth)?;
            }
        }
        if let Some((right_index, right_cell)) = grid_index
            .checked_add(1)
            .and_then(|index| {
                if index % self.width() != 0 {
                    self.get_index_mut(index).map(|cell| (index, cell))
                } else {
                    None
                }
            })
            .take_if(|(_, right_cell)| right_cell.options.len() != 1)
        {
            let count_before = right_cell.options.len();
            let mut available_options = HashMap::with_capacity(tiles.len());
            for tile_index in available_indexes.keys() {
                available_options.extend(tiles[*tile_index].neighbors.borrow().right.iter());
            }
            right_cell.update_options(&available_options)?;
            if count_before != right_cell.options.len() {
                let options = right_cell.options.clone();
                self.update_neighbors(tiles, right_index, options, depth)?;
            }
        }
        Ok(())
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
        let result = self.grid.get_index(self.i).map(|cell| (x, y, cell));
        self.i += 1;
        result
    }
}

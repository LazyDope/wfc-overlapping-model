use std::{
    collections::HashSet,
    ops::{Index, IndexMut},
};

use rand::{distr::weighted::WeightedIndex, prelude::*};

pub mod cell;

use crate::tile::{Direction, Tile};
use cell::{Cell, Exhausted};

#[derive(Clone)]
pub struct Grid {
    buf: Vec<Cell>,
    height: usize,
    width: usize,
    options: HashSet<usize>,
    attempts: u32,
    initial_max_depth: usize,
}

impl Grid {
    pub fn new(
        width: usize,
        height: usize,
        options: HashSet<usize>,
        initial_max_depth: usize,
    ) -> Grid {
        let buf = (0..(width * height))
            .map(|_| Cell::new(options.clone()))
            .collect();
        Self {
            buf,
            height,
            width,
            options,
            initial_max_depth,
            attempts: 0,
        }
    }

    pub fn regenerate(&mut self, increase_attempts: bool) {
        self.buf = (0..(self.width * self.height))
            .map(|_| Cell::new(self.options.clone()))
            .collect();
        if increase_attempts {
            self.attempts += 1;
        }
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

    pub fn collapse<T: Rng>(&mut self, tiles: &[Tile], rng: &mut T) -> Result<bool, Exhausted> {
        {
            let mut cells: Vec<(usize, &mut Cell)> = self
                .buf
                .iter_mut()
                .enumerate()
                .filter(|(_, cell)| cell.options.len() > 1)
                .collect();

            if cells.is_empty() {
                return Ok(false);
            }
            // precalculate whether this will be the last cell to compute to that we don't need to filter the vector again.
            let last_cell = cells.len() == 1;

            let mut min = f64::MAX;
            let mut min_indexes = Vec::new();
            for (i, (_, cell)) in cells.iter().enumerate() {
                let entropy = cell.calculate_entropy(tiles);
                if entropy == min {
                    min_indexes.push(i);
                } else if entropy < min {
                    min = entropy;
                    min_indexes = vec![i];
                }
            }

            let (index, cell) = &mut cells[*min_indexes.choose(rng).expect("No cells in grid")];
            let dist = WeightedIndex::new(
                cell.options
                    .iter()
                    .map(|&tile_index| tiles[tile_index].frequency as usize),
            )
            .expect("This distribution should always succeed at being created");
            let chosen_index = cell
                .options
                .drain()
                .nth(dist.sample(rng))
                .ok_or(Exhausted)?;
            cell.options.insert(chosen_index);
            if last_cell {
                return Ok(false);
            }
            let (grid_index, options) = (*index, cell.options.clone());
            self.update_neighbors(tiles, grid_index, options, 0)?;
        };
        Ok(true)
    }

    pub fn cells(&self) -> Cells<'_> {
        Cells { grid: self, i: 0 }
    }

    pub fn index_in_direction(&self, index: usize, direction: Direction) -> Option<usize> {
        match direction {
            Direction::Up => index.checked_sub(self.width()),
            Direction::Down => index.checked_add(self.width()),
            Direction::Left => index
                .checked_sub(1)
                .take_if(|index| *index % self.width() != self.width() - 1),
            Direction::Right => index
                .checked_add(1)
                .take_if(|index| *index % self.width() != 0),
        }
    }

    fn max_depth(&self) -> usize {
        (self.initial_max_depth * 2usize.pow(self.attempts + 1)).min(self.width() + self.height())
    }

    fn update_neighbors(
        &mut self,
        tiles: &[Tile],
        grid_index: usize,
        options: HashSet<usize>,
        mut depth: usize,
    ) -> Result<(), Exhausted> {
        if depth > self.max_depth() {
            return Ok(());
        }
        let available_indexes = &options;
        for direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            if let Some((neighbor_index, neighbor_cell)) = self
                .index_in_direction(grid_index, direction)
                .and_then(|index| self.get_index_mut(index).map(|cell| (index, cell)))
                .take_if(|(_, neighbor_cell)| neighbor_cell.options.len() != 1)
            {
                let old_len = neighbor_cell.options.len();
                let mut available_options = HashSet::with_capacity(tiles.len());
                for tile_index in available_indexes.iter() {
                    available_options
                        .extend(tiles[*tile_index].neighbors.borrow()[direction].iter());
                }
                neighbor_cell.update_options(&available_options)?;
                let new_len = neighbor_cell.options.len();
                if old_len != new_len {
                    let options = neighbor_cell.options.clone();
                    if new_len != 1 {
                        depth += 1
                    }
                    self.update_neighbors(tiles, neighbor_index, options, depth)?;
                }
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

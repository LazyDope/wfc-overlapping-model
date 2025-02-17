use crate::image_impls::{LoopingSubImage, LoopingView};

use std::{
    cell::RefCell,
    collections::HashSet,
    ops::{Index, IndexMut},
};

use image::RgbImage;

#[derive(Default, Debug)]
pub struct Directions<T> {
    pub up: T,
    pub down: T,
    pub left: T,
    pub right: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Tile {
    pub image: RgbImage,
    pub neighbors: RefCell<Directions<HashSet<usize>>>,
}

impl Tile {
    pub fn view_in_direction(&self, direction: Direction) -> LoopingSubImage<&RgbImage> {
        match direction {
            Direction::Up => self.up_view(),
            Direction::Down => self.down_view(),
            Direction::Left => self.left_view(),
            Direction::Right => self.right_view(),
        }
    }

    pub fn up_view(&self) -> LoopingSubImage<&RgbImage> {
        self.image
            .looping_view(0, 0, self.image.width(), self.image.height().div_ceil(2))
    }
    pub fn down_view(&self) -> LoopingSubImage<&RgbImage> {
        self.image.looping_view(
            0,
            self.image.height() / 2,
            self.image.width(),
            self.image.height().div_ceil(2),
        )
    }
    pub fn left_view(&self) -> LoopingSubImage<&RgbImage> {
        self.image
            .looping_view(0, 0, self.image.width().div_ceil(2), self.image.height())
    }
    pub fn right_view(&self) -> LoopingSubImage<&RgbImage> {
        self.image.looping_view(
            self.image.width() / 2,
            0,
            self.image.width().div_ceil(2),
            self.image.height(),
        )
    }
}

impl Direction {
    pub fn opposing(&self) -> Direction {
        use Direction::*;
        match self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left,
        }
    }
}

impl<T> Index<Direction> for Directions<T> {
    type Output = T;

    fn index(&self, index: Direction) -> &Self::Output {
        match index {
            Direction::Up => &self.up,
            Direction::Down => &self.down,
            Direction::Left => &self.left,
            Direction::Right => &self.right,
        }
    }
}

impl<T> IndexMut<Direction> for Directions<T> {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        match index {
            Direction::Up => &mut self.up,
            Direction::Down => &mut self.down,
            Direction::Left => &mut self.left,
            Direction::Right => &mut self.right,
        }
    }
}

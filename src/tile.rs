use crate::image_impls::{LoopingSubImage, LoopingView};

use std::cell::RefCell;

use image::RgbImage;

#[derive(Default)]
pub(crate) struct Directions<T> {
    pub(crate) up: T,
    pub(crate) down: T,
    pub(crate) left: T,
    pub(crate) right: T,
}

pub(crate) struct Tile {
    pub(crate) image: RgbImage,
    pub(crate) neighbors: RefCell<Directions<Vec<usize>>>,
}

impl Tile {
    pub(crate) fn up_view(&self) -> LoopingSubImage<&RgbImage> {
        self.image.looping_view(0, 0, self.image.width(), 2)
    }
    pub(crate) fn down_view(&self) -> LoopingSubImage<&RgbImage> {
        self.image
            .looping_view(0, self.image.height() - 2, self.image.width(), 2)
    }
    pub(crate) fn left_view(&self) -> LoopingSubImage<&RgbImage> {
        self.image.looping_view(0, 0, 2, self.image.height())
    }
    pub(crate) fn right_view(&self) -> LoopingSubImage<&RgbImage> {
        self.image
            .looping_view(self.image.width() - 2, 0, 2, self.image.height())
    }
}

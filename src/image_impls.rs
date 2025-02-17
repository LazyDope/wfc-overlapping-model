use std::ops::Deref;

use clap::ValueEnum;
use image::{GenericImageView, ImageBuffer, Pixel, SubImage};

pub trait Tilable: Sized {
    fn tiles(&self, tile_size: u32, border_style: BorderStyle) -> Tiles<'_, Self>;
}

#[derive(Copy, Clone, ValueEnum, Default)]
pub enum BorderStyle {
    #[default]
    Looping,
    Clamped,
}

impl BorderStyle {
    pub fn check_borders(&self, value: u32, add: i64, buf_size: u32) -> u32 {
        match self {
            BorderStyle::Looping => (value as i64 + add).rem_euclid(buf_size as i64) as u32,
            BorderStyle::Clamped => (value as i64 + add).clamp(0, buf_size as i64 - 1) as u32,
        }
    }
}

#[derive(Copy, Clone)]
pub struct LoopingSubImage<I> {
    image: I,
    border_style: BorderStyle,
    xoffset: i64,
    yoffset: i64,
    width: u32,
    height: u32,
}

pub struct Tiles<'img, I> {
    image: &'img I,
    x: i64,
    y: i64,
    tile_size: u32,
    border_style: BorderStyle,
}

impl<T> Tilable for T
where
    T: GenericImageView,
{
    fn tiles(&self, tile_size: u32, border_style: BorderStyle) -> Tiles<'_, Self> {
        Tiles {
            image: self,
            x: -(tile_size as i64 / 2),
            y: -(tile_size as i64 / 2),
            tile_size,
            border_style,
        }
    }
}
type DerefPixel<I> = <<I as Deref>::Target as GenericImageView>::Pixel;
type DerefSubpixel<I> = <DerefPixel<I> as Pixel>::Subpixel;

impl<'img, I> Iterator for Tiles<'img, I>
where
    I: GenericImageView,
{
    type Item = LoopingSubImage<&'img I>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= (self.image.height() - self.tile_size / 2) as i64 {
            return None;
        }
        let result = self.image.looping_view(
            self.x,
            self.y,
            self.tile_size,
            self.tile_size,
            self.border_style,
        );
        self.x += 1;
        if self.x >= (self.image.width() - self.tile_size / 2) as i64 {
            self.x -= self.image.width() as i64;
            self.y += 1;
        }
        Some(result)
    }
}

impl<I> LoopingSubImage<I>
where
    I: Deref,
    I::Target: GenericImageView + Sized,
{
    pub fn to_image(&self) -> ImageBuffer<DerefPixel<I>, Vec<DerefSubpixel<I>>> {
        let mut out = ImageBuffer::new(self.width, self.height);

        for (x, y, pixel) in self.pixels() {
            out.put_pixel(x, y, pixel);
        }

        out
    }
}

impl<I> GenericImageView for LoopingSubImage<I>
where
    I: Deref,
    I::Target: GenericImageView + Sized,
{
    type Pixel = <<I as Deref>::Target as GenericImageView>::Pixel;

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        self.image.get_pixel(
            self.border_style
                .check_borders(x, self.xoffset, self.image.width()),
            self.border_style
                .check_borders(y, self.yoffset, self.image.height()),
        )
    }

    fn view(&self, x: u32, y: u32, width: u32, height: u32) -> SubImage<&Self> {
        let x = self
            .border_style
            .check_borders(x, self.xoffset, self.image.width());
        let y = self
            .border_style
            .check_borders(y, self.yoffset, self.image.height());
        SubImage::new(self, x, y, width, height)
    }
}

impl<I> PartialEq for LoopingSubImage<I>
where
    I: Deref,
    I::Target: GenericImageView + Sized,
    DerefPixel<I>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.dimensions() == other.dimensions()
            && self
                .pixels()
                .zip(other.pixels())
                .all(|((.., l), (.., r))| l == r)
    }
}

pub trait LoopingView: GenericImageView {
    fn looping_view(
        &self,
        xoffset: i64,
        yoffset: i64,
        xstride: u32,
        ystride: u32,
        border_style: BorderStyle,
    ) -> LoopingSubImage<&Self>;
}

impl<T> LoopingView for T
where
    T: GenericImageView,
{
    fn looping_view(
        &self,
        xoffset: i64,
        yoffset: i64,
        xstride: u32,
        ystride: u32,
        border_style: BorderStyle,
    ) -> LoopingSubImage<&Self> {
        LoopingSubImage {
            image: self,
            xoffset,
            yoffset,
            width: xstride,
            height: ystride,
            border_style,
        }
    }
}

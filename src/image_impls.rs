use std::ops::Deref;

use image::{GenericImageView, ImageBuffer, Pixel, SubImage};

pub trait Tilable: Sized {
    fn tiles(&self, tile_size: u32) -> Tiles<'_, Self>;
}

#[derive(Copy, Clone)]
pub struct LoopingSubImage<I> {
    image: I,
    xoffset: u32,
    yoffset: u32,
    width: u32,
    height: u32,
}

pub struct Tiles<'img, I> {
    image: &'img I,
    x: u32,
    y: u32,
    tile_size: u32,
}

impl<T> Tilable for T
where
    T: GenericImageView,
{
    fn tiles(&self, tile_size: u32) -> Tiles<'_, Self> {
        Tiles {
            image: self,
            x: 0,
            y: 0,
            tile_size,
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
        if self.y >= self.image.height() {
            return None;
        }
        let result = self
            .image
            .looping_view(self.x, self.y, self.tile_size, self.tile_size);
        self.x += 1;
        if self.x >= self.image.width() {
            self.x = 0;
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
            (x + self.xoffset) % self.image.width(),
            (y + self.yoffset) % self.image.height(),
        )
    }

    fn view(&self, x: u32, y: u32, width: u32, height: u32) -> SubImage<&Self> {
        let x = (self.xoffset + x) % self.image.width();
        let y = (self.yoffset + y) % self.image.height();
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
        xoffset: u32,
        yoffset: u32,
        xstride: u32,
        ystride: u32,
    ) -> LoopingSubImage<&Self>;
}

impl<T> LoopingView for T
where
    T: GenericImageView,
{
    fn looping_view(
        &self,
        xoffset: u32,
        yoffset: u32,
        xstride: u32,
        ystride: u32,
    ) -> LoopingSubImage<&Self> {
        LoopingSubImage {
            image: self,
            xoffset,
            yoffset,
            width: xstride,
            height: ystride,
        }
    }
}

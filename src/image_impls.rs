use std::ops::Deref;

use nannou::image::{GenericImageView, SubImage};

pub trait Tilable {
    type Tile: GenericImageView;

    fn looping_tiles(self, tile_size: u32) -> Vec<Self::Tile>;
}

#[derive(Copy, Clone)]
pub struct LoopingSubImage<I> {
    image: I,
    xoffset: u32,
    yoffset: u32,
    xstride: u32,
    ystride: u32,
}

impl<'a, T> Tilable for &'a T
where
    T: GenericImageView,
{
    type Tile = LoopingSubImage<&'a T::InnerImageView>;

    fn looping_tiles(self, tile_size: u32) -> Vec<Self::Tile> {
        let mut buf = Vec::with_capacity(self.width() as usize * self.height() as usize);
        for (x, y, _) in self.pixels() {
            buf.push(self.looping_view(x, y, tile_size, tile_size));
        }
        buf
    }
}

impl<I> GenericImageView for LoopingSubImage<I>
where
    I: Deref,
    I::Target: GenericImageView + Sized,
{
    type Pixel = <<I as Deref>::Target as GenericImageView>::Pixel;

    type InnerImageView = I::Target;

    fn dimensions(&self) -> (u32, u32) {
        (self.xstride, self.ystride)
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        (self.xoffset, self.yoffset, self.xstride, self.ystride)
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        self.image.get_pixel(
            (x + self.xoffset) % self.image.width(),
            (y + self.yoffset) % self.image.height(),
        )
    }

    fn view(&self, x: u32, y: u32, width: u32, height: u32) -> SubImage<&Self::InnerImageView> {
        let x = (self.xoffset + x) % self.image.width();
        let y = (self.yoffset + y) % self.image.height();
        SubImage::new(self.inner(), x, y, width, height)
    }

    fn inner(&self) -> &Self::InnerImageView {
        &self.image
    }
}

pub trait LoopingView: GenericImageView {
    fn looping_view(
        &self,
        xoffset: u32,
        yoffset: u32,
        xstride: u32,
        ystride: u32,
    ) -> LoopingSubImage<&Self::InnerImageView>;
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
    ) -> LoopingSubImage<&Self::InnerImageView> {
        LoopingSubImage {
            image: self.inner(),
            xoffset,
            yoffset,
            xstride,
            ystride,
        }
    }
}

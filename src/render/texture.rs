use gfx::{Factory, Resources};
use gfx::format::{Rgba8, TextureFormat};
use gfx::handle::ShaderResourceView;
use gfx::texture::{AaMode, Kind};
use image;
use std::ops::{Deref, DerefMut};
use std::path::Path;

pub struct Texture<R, T>(ShaderResourceView<R, T::View>)
    where R: Resources,
          T: TextureFormat;

impl<R, T> Texture<R, T>
    where R: Resources,
          T: TextureFormat
{
    pub fn into_inner(self) -> ShaderResourceView<R, T::View> {
        self.0
    }
}

impl<R> Texture<R, Rgba8>
    where R: Resources
{
    pub fn max1x1<F>(factory: &mut F) -> Self
        where F: Factory<R>
    {
        let max = u8::max_value();
        let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(
            Kind::D2(1, 1, AaMode::Single),
            &[&[max, max, max, max]]).unwrap();
        view.into()
    }

    pub fn from_file<F, P>(factory: &mut F, path: P) -> Self
        where F: Factory<R>,
              P: AsRef<Path>
    {
        let data = image::open(path).unwrap().to_rgba();
        let (width, height) = data.dimensions();
        let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(
            Kind::D2(width as u16, height as u16, AaMode::Single),
            &[data.into_vec().as_slice()]).unwrap();
        view.into()
    }
}

impl<R, T> From<ShaderResourceView<R, T::View>> for Texture<R, T>
    where R: Resources,
          T: TextureFormat
{
    fn from(view: ShaderResourceView<R, T::View>) -> Self {
        Texture(view)
    }
}

impl<R, T> Deref for Texture<R, T>
    where R: Resources,
          T: TextureFormat
{
    type Target = ShaderResourceView<R, T::View>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R, T> DerefMut for Texture<R, T>
    where R: Resources,
          T: TextureFormat
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

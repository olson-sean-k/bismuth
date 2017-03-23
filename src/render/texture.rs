use gfx::{self, Factory, Resources};
use gfx::format::{Rgba8, TextureFormat};
use gfx::handle::{Sampler, ShaderResourceView};
use gfx::texture::{AaMode, FilterMethod, Kind, SamplerInfo, WrapMode};
use image;
use std::path::Path;

#[derive(Clone)]
pub struct Texture<R, T>
    where R: Resources,
          T: TextureFormat
{
    pub surface: gfx::handle::Texture<R, T::Surface>,
    pub view: ShaderResourceView<R, T::View>,
    pub sampler: Sampler<R>,
}

impl<R, T> Texture<R, T>
    where R: Resources,
          T: TextureFormat,
          T::View: Clone
{
    pub fn to_pipeline_data(&self) -> (ShaderResourceView<R, T::View>, Sampler<R>) {
        (self.view.clone(), self.sampler.clone())
    }
}

impl<R> Texture<R, Rgba8>
    where R: Resources
{
    pub fn white<F>(factory: &mut F) -> Self
        where F: Factory<R>
    {
        let max = u8::max_value();
        let (surface, view) = factory.create_texture_immutable_u8::<Rgba8>(
            Kind::D2(1, 1, AaMode::Single),
            &[&[max, max, max, max]]).unwrap();
        Texture {
            surface: surface,
            view: view,
            sampler: factory.create_sampler(
                SamplerInfo::new(FilterMethod::Trilinear, WrapMode::Tile)),
        }
    }

    pub fn from_file<F, P>(factory: &mut F, path: P) -> Self
        where F: Factory<R>,
              P: AsRef<Path>
    {
        // TODO: Return a `Result` and expose any errors from `image::open`.
        let data = image::open(path).unwrap().to_rgba();
        let (width, height) = data.dimensions();
        let (surface, view) = factory.create_texture_immutable_u8::<Rgba8>(
            Kind::D2(width as u16, height as u16, AaMode::Single),
            &[data.into_vec().as_slice()]).unwrap();
        Texture {
            surface: surface,
            view: view,
            sampler: factory.create_sampler(
                SamplerInfo::new(FilterMethod::Trilinear, WrapMode::Tile)),
        }
    }
}

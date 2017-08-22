use gfx::{self, Factory, Resources};
use gfx::format::{R8_G8_B8_A8, Rgba8, Srgb, TextureChannel, TextureFormat, Unorm};
use gfx::handle::{Sampler, ShaderResourceView};
use gfx::texture::{AaMode, FilterMethod, Kind, SamplerInfo, WrapMode};
use image;
use std::path::Path;

use super::error::*;

pub trait NormalizedChannel {}
pub trait UnsignedChannel {}

impl NormalizedChannel for Srgb {}
impl NormalizedChannel for Unorm {}

impl UnsignedChannel for Srgb {}
impl UnsignedChannel for Unorm {}

#[derive(Clone)]
pub struct Texture<R, T>
where
    R: Resources,
    T: TextureFormat,
{
    pub surface: gfx::handle::Texture<R, T::Surface>,
    pub view: ShaderResourceView<R, T::View>,
    pub sampler: Sampler<R>,
}

impl<R, T> Texture<R, T>
where
    R: Resources,
    T: TextureFormat,
{
    fn new(
        surface: gfx::handle::Texture<R, T::Surface>,
        view: ShaderResourceView<R, T::View>,
        sampler: Sampler<R>,
    ) -> Self {
        Texture {
            surface: surface,
            view: view,
            sampler: sampler,
        }
    }
}

impl<R, T> Texture<R, T>
where
    R: Resources,
    T: TextureFormat,
    T::View: Clone,
{
    pub fn to_pipeline_data(&self) -> (ShaderResourceView<R, T::View>, Sampler<R>) {
        (self.view.clone(), self.sampler.clone())
    }
}

impl<R, C> Texture<R, (R8_G8_B8_A8, C)>
where
    R: Resources,
    C: NormalizedChannel + TextureChannel + UnsignedChannel,
    (R8_G8_B8_A8, C): TextureFormat,
{
    pub fn from_file<F, P>(factory: &mut F, path: P) -> Result<Self>
    where
        F: Factory<R>,
        P: AsRef<Path>,
    {
        let data = image::open(path)?.to_rgba();
        let (width, height) = data.dimensions();
        let (surface, view) = factory.create_texture_immutable_u8::<(R8_G8_B8_A8, C)>(
            Kind::D2(width as u16, height as u16, AaMode::Single),
            &[data.into_vec().as_slice()],
        )?;
        Ok(Texture::new(
            surface,
            view,
            factory.create_sampler(SamplerInfo::new(FilterMethod::Trilinear, WrapMode::Tile)),
        ))
    }
}

impl<R> Texture<R, Rgba8>
where
    R: Resources,
{
    #[inline(always)]
    pub fn white<F>(factory: &mut F) -> Result<Self>
    where
        F: Factory<R>,
    {
        Self::grayscale(factory, u8::max_value())
    }

    #[inline(always)]
    pub fn black<F>(factory: &mut F) -> Result<Self>
    where
        F: Factory<R>,
    {
        Self::grayscale(factory, u8::min_value())
    }

    pub fn grayscale<F>(factory: &mut F, value: u8) -> Result<Self>
    where
        F: Factory<R>,
    {
        let (surface, view) = factory.create_texture_immutable_u8::<Rgba8>(
            Kind::D2(1, 1, AaMode::Single),
            &[&[value, value, value, value]],
        )?;
        Ok(Texture::new(
            surface,
            view,
            factory.create_sampler(SamplerInfo::new(FilterMethod::Trilinear, WrapMode::Tile)),
        ))
    }
}

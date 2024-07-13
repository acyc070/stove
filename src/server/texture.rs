use super::*;

#[derive(Asset, TypePath)]
pub struct UTex(Image);

impl Into<Image> for UTex {
    fn into(self) -> Image {
        self.0
    }
}

pub struct UTexLoader;

impl bevy::asset::AssetLoader for UTexLoader {
    type Asset = UTex;

    type Settings = ();

    type Error = unreal_asset::Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader<'_>,
        _: &'a Self::Settings,
        ctx: &'a mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        use bevy::asset::AsyncReadExt;
        let mut buf = vec![];
        reader.read_to_end(&mut buf).await?;
        let asset = unreal_asset::Asset::new(
            std::io::Cursor::new(buf),
            ctx.read_asset_bytes(ctx.path().with_extension("uexp"))
                .await
                .ok()
                .map(std::io::Cursor::new),
            unreal_asset::engine_version::EngineVersion::VER_UE5_1,
            None,
        )?;
        match extras::get_tex_info(asset, None) {
            Ok((false, width, height, data)) => Ok(UTex(Image {
                data,
                texture_descriptor: bevy::render::render_resource::TextureDescriptor {
                    label: None,
                    size: bevy::render::render_resource::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: bevy::render::render_resource::TextureDimension::D2,
                    format: bevy::render::render_resource::TextureFormat::Rgba8Unorm,
                    usage: bevy::render::render_resource::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[bevy::render::render_resource::TextureFormat::Rgba8Unorm],
                },
                sampler: bevy::render::texture::ImageSampler::Descriptor(
                    bevy::render::texture::ImageSamplerDescriptor {
                        address_mode_u: bevy::render::texture::ImageAddressMode::Repeat,
                        address_mode_v: bevy::render::texture::ImageAddressMode::Repeat,
                        address_mode_w: bevy::render::texture::ImageAddressMode::Repeat,
                        ..default()
                    },
                ),
                ..default()
            })),
            _ => Err(unreal_asset::Error::invalid_file(
                "couldn't parse texture".into(),
            )),
        }
    }
}

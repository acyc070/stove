use super::*;

#[derive(Asset, TypePath)]
pub struct UMesh {
    mesh: Handle<Mesh>,
    mat: Option<Handle<StandardMaterial>>,
}

pub struct UMeshLoader;

impl bevy::asset::AssetLoader for UMeshLoader {
    type Asset = UMesh;

    type Settings = ();

    type Error = unreal_asset::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _: &'a Self::Settings,
        ctx: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            use bevy::asset::AsyncReadExt;
            let mut buf = vec![];
            reader.read_to_end(&mut buf).await?;
            let asset = unreal_asset::Asset::new(
                std::io::Cursor::new(buf),
                None,
                unreal_asset::engine_version::EngineVersion::VER_UE5_1,
                None,
            )?;
            let (positions, indices, uvs, mats, _mat_data) = extras::get_mesh_info(asset)?;
            async fn open(
                ctx: &mut bevy::asset::LoadContext<'_>,
                mat: &String,
            ) -> Result<unreal_asset::Asset<std::io::Cursor<Vec<u8>>>, unreal_asset::Error>
            {
                unreal_asset::Asset::new(
                    std::io::Cursor::new(
                        ctx.read_asset_bytes(mat.clone() + ".uasset")
                            .await
                            .map_err(|_| unreal_asset::Error::no_data("no material".into()))?,
                    ),
                    ctx.read_asset_bytes(mat.clone() + "uexp")
                        .await
                        .ok()
                        .map(std::io::Cursor::new),
                    unreal_asset::engine_version::EngineVersion::VER_UE5_1,
                    None,
                )
            }
            let first = mats.first();
            let tex = match first {
                Some(mat) => extras::get_tex_paths(unreal_asset::Asset::new(
                    std::io::Cursor::new(
                        ctx.read_asset_bytes(mat.clone() + ".uasset")
                            .await
                            .map_err(|_| unreal_asset::Error::no_data("no material".into()))?,
                    ),
                    ctx.read_asset_bytes(mat.clone() + "uexp")
                        .await
                        .ok()
                        .map(std::io::Cursor::new),
                    unreal_asset::engine_version::EngineVersion::VER_UE5_1,
                    None,
                )?)
                .first(),
                None => None,
            };
            let image = match tex {
                Some(mat) => extras::get_tex_info(
                    unreal_asset::Asset::new(
                        std::io::Cursor::new(
                            ctx.read_asset_bytes(mat.clone() + ".uasset")
                                .await
                                .map_err(|_| unreal_asset::Error::no_data("no material".into()))?,
                        ),
                        ctx.read_asset_bytes(mat.clone() + "uexp")
                            .await
                            .ok()
                            .map(std::io::Cursor::new),
                        unreal_asset::engine_version::EngineVersion::VER_UE5_1,
                        None,
                    )?,
                    ctx.read_asset_bytes(mat.clone() + "uptnl")
                        .await
                        .ok()
                        .map(std::io::Cursor::new),
                )
                .ok(),
                None => None,
            };
            Ok(UMesh {
                mesh: ctx.add_labeled_asset(
                    "mesh".into(),
                    Mesh::new(
                        bevy::render::render_resource::PrimitiveTopology::TriangleList,
                        default(),
                    )
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                    .with_inserted_attribute(
                        Mesh::ATTRIBUTE_UV_0,
                        uvs.into_iter().map(|uv| uv[0]).collect::<Vec<_>>(),
                    )
                    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices)),
                ),
                mat: image.map(|(_, width, height, data)| {
                    ctx.add_labeled_asset("material".into(), StandardMaterial {
                    base_color_texture: Some(ctx.add_labeled_asset("texture".into(), Image {
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
                            view_formats: &[
                                bevy::render::render_resource::TextureFormat::Rgba8Unorm,
                            ],
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
                    unlit: true,
                    ..default()
                })
                }),
            })
        })
    }
}

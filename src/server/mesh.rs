use super::*;

#[derive(Asset, TypePath)]
pub struct UMesh {
    mesh: Handle<Mesh>,
    mat: Handle<unlit::Unlit>,
}

pub struct UMeshLoader;

impl bevy::asset::AssetLoader for UMeshLoader {
    type Asset = UMesh;

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
            None,
            unreal_asset::engine_version::EngineVersion::VER_UE5_1,
            None,
        )?;
        let (positions, indices, uvs, mats, _mat_data) = extras::get_mesh_info(asset)?;
        // let mut image = None;
        // 'outer: for mat in mats {
        //     let paths = extras::get_tex_paths(open(ctx, &mat).await?);
        //     for path in paths {
        //         if let Ok((false, width, height, data)) = extras::get_tex_info(
        //             open(ctx, &mat).await?,
        //             ctx.read_asset_bytes(mat.clone() + "uptnl")
        //                 .await
        //                 .ok()
        //                 .map(std::io::Cursor::new),
        //         ) {
        //             image = Some((width, height, data));
        //             break 'outer;
        //         }
        //     }
        // }
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
            mat: ctx.load(mats.into_iter().next().unwrap_or_default()),
        })
    }
}

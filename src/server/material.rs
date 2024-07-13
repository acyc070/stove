use super::*;

pub struct UMatLoader;

impl bevy::asset::AssetLoader for UMatLoader {
    type Asset = unlit::Unlit;

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
        let paths = extras::get_tex_paths(asset);
        match paths.first() {
            Some(path) => Ok(unlit::Unlit {
                texture: ctx.load(path.to_string() + ".uasset"),
            }),
            None => Err(unreal_asset::Error::no_data(
                "no texture for the material could be found".into(),
            )),
        }
    }
}

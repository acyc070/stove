use super::*;

#[derive(Asset, TypePath)]
pub struct UMat(Vec<String>);

pub struct UMatLoader;

impl bevy::asset::AssetLoader for UMatLoader {
    type Asset = UMat;

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
                ctx.read_asset_bytes(ctx.path().with_extension("uexp")).await.ok().map(std::io::Cursor::new),
                unreal_asset::engine_version::EngineVersion::VER_UE5_1,
                None,
            )?;
            let paths = extras::get_tex_paths(asset);
            match paths.first(){
                Some(path) => {
                    let bulk = ctx.read_asset_bytes(path.to_string() + ".uptnl").await.ok();
                    extras::get_tex_info(unre, )
                },
                None => ,
            }
            ctx.read_asset_bytes()
            Ok(UMat())
        })
    }
}

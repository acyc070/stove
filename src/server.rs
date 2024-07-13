use super::*;

mod material;
mod mesh;
mod texture;

async fn open(
    ctx: &mut bevy::asset::LoadContext<'_>,
    mat: &String,
) -> Result<unreal_asset::Asset<std::io::Cursor<Vec<u8>>>, unreal_asset::Error> {
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

pub struct PakSystem(pub Vec<(std::path::PathBuf, repak::PakReader)>);

impl bevy::asset::io::AssetReader for PakSystem {
    async fn read<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<Box<bevy::asset::io::Reader<'a>>, bevy::asset::io::AssetReaderError> {
        let str = path.to_string_lossy();
        match self.0.iter().find(|(_, pak)| {
            // this'll be loaded for each pak
            str.starts_with("Game/Content") && pak.files().iter().any(|file| file.as_str() == str)
        }) {
            Some((file, pak)) => match std::fs::File::open(&file) {
                Ok(mut file) => {
                    // implement caching later or read directly from pak
                    let reader: Box<bevy::asset::io::Reader> = Box::new(
                        bevy::asset::io::VecReader::new(pak.get(&str, &mut file).unwrap()),
                    );
                    Ok(reader)
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        Err(bevy::asset::io::AssetReaderError::NotFound(path.into()))
                    } else {
                        Err(e.into())
                    }
                }
            },
            None => Err(bevy::asset::io::AssetReaderError::NotFound(path.into())),
        }
    }

    async fn read_meta<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<Box<bevy::asset::io::Reader<'a>>, bevy::asset::io::AssetReaderError> {
        Err(bevy::asset::io::AssetReaderError::NotFound(path.into()))
    }

    async fn read_directory<'a>(
        &'a self,
        _: &'a std::path::Path,
    ) -> Result<Box<bevy::asset::io::PathStream>, bevy::asset::io::AssetReaderError> {
        todo!()
    }

    async fn is_directory<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<bool, bevy::asset::io::AssetReaderError> {
        let str = path.to_string_lossy();
        Ok(path.is_dir()
            && self.0.iter().any(|(_, pak)| {
                pak.files()
                    .iter()
                    .any(|file| file.starts_with(str.as_ref()))
            }))
    }
}

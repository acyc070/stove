use super::*;

pub struct PakSystem(pub Vec<(std::path::PathBuf, repak::PakReader)>);

impl bevy::asset::io::AssetReader for PakSystem {
    fn read<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> bevy::utils::BoxedFuture<
        'a,
        Result<Box<bevy::asset::io::Reader<'a>>, bevy::asset::io::AssetReaderError>,
    > {
        Box::pin(async move {
            let str = path.to_string_lossy();
            match self.0.iter().find(|(_, pak)| {
                // this'll be loaded for each pak
                str.starts_with("Game/Content")
                    && pak.files().iter().any(|file| file.as_str() == str)
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
        })
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> bevy::utils::BoxedFuture<
        'a,
        Result<Box<bevy::asset::io::Reader<'a>>, bevy::asset::io::AssetReaderError>,
    > {
        Box::pin(async { Err(bevy::asset::io::AssetReaderError::NotFound(path.into())) })
    }

    fn read_directory<'a>(
        &'a self,
        _: &'a std::path::Path,
    ) -> bevy::utils::BoxedFuture<
        'a,
        Result<Box<bevy::asset::io::PathStream>, bevy::asset::io::AssetReaderError>,
    > {
        todo!()
    }

    fn is_directory<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> bevy::utils::BoxedFuture<'a, Result<bool, bevy::asset::io::AssetReaderError>> {
        Box::pin(async {
            let str = path.to_string_lossy();
            Ok(path.is_dir()
                && self.0.iter().any(|(_, pak)| {
                    pak.files()
                        .iter()
                        .any(|file| file.starts_with(str.as_ref()))
                }))
        })
    }
}

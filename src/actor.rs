use std::fs::File;
use unreal_asset::{
    cast,
    error::Error,
    exports::{Export, ExportBaseTrait, ExportNormalTrait},
    properties::{Property, PropertyDataTrait},
    reader::asset_trait::AssetTrait,
    types::{FName, PackageIndex},
    Asset,
};

mod delete;
mod duplicate;
mod transform;
mod transplant;
mod ui;

pub enum DrawType {
    Mesh(String),
    Cube,
}

pub struct Actor {
    export: usize,
    transform: usize,
    pub name: String,
    pub class: String,
    pub draw_type: DrawType,
}

impl Actor {
    fn index(&self) -> PackageIndex {
        PackageIndex::new(self.export as i32 + 1)
    }
    pub fn new(asset: &Asset<File>, package: PackageIndex) -> Result<Self, Error> {
        if package.index == 0 {
            return Err(Error::invalid_package_index(
                "actor was null reference".to_string(),
            ));
        }
        let export = package.index as usize - 1;
        let Some(ex) = asset.get_export(package) else {
            return Err(Error::invalid_package_index(format!(
                "failed to find actor at index {}",
                package.index
            )))
        };
        let Some(norm) = ex.get_normal_export() else {
            return Err(Error::no_data(format!("actor at index {} failed to parse", package.index)))
        };
        let name = norm.base_export.object_name.content.clone();
        let class = asset
            .get_import(norm.base_export.class_index)
            .map(|import| import.object_name.content.clone())
            .unwrap_or_default();
        let draw_type = match norm
            .base_export
            .create_before_serialization_dependencies
            .iter()
            .filter_map(|i| asset.get_export(*i))
            .filter_map(Export::get_normal_export)
            .find(|i| {
                asset
                    .get_import(i.get_base_export().class_index)
                    .filter(|import| import.object_name.content == "StaticMeshComponent")
                    .is_some()
            }) {
            Some(norm) => norm
                .properties
                .iter()
                .find_map(|prop| {
                    cast!(Property, ObjectProperty, prop)
                        .filter(|prop| prop.get_name().content == "StaticMesh")
                })
                .and_then(|obj| asset.get_import(obj.value))
                .and_then(|import| asset.get_import(import.outer_index))
                .map_or(DrawType::Cube, |path| {
                    DrawType::Mesh(path.object_name.content.clone())
                }),
            None => DrawType::Cube,
        };
        // normally these are further back so reversed should be a bit faster
        for prop in norm.properties.iter().rev() {
            match prop.get_name().content.as_str() {
                // of course this wouldn't be able to be detected if all transforms were left default
                "RelativeLocation" | "RelativeRotation" | "RelativeScale3D" => {
                    return Ok(Self {
                        export,
                        transform: export,
                        name,
                        class,
                        draw_type,
                    })
                }
                "RootComponent" => {
                    if let Property::ObjectProperty(obj) = prop {
                        if obj.value.is_export() {
                            return Ok(Self {
                                export,
                                transform: obj.value.index as usize - 1,
                                name,
                                class,
                                draw_type,
                            });
                        }
                    }
                }
                _ => continue,
            }
        }
        Err(Error::no_data(format!(
            "couldn't find transform component for {}",
            &norm.base_export.object_name.content
        )))
    }

    /// gets all exports related to the given actor
    fn get_actor_exports(&self, asset: &Asset<File>, offset: usize) -> Vec<Export> {
        // get references to all the actor's children
        let mut child_indexes: Vec<PackageIndex> = asset.exports[self.export]
            .get_base_export()
            .create_before_serialization_dependencies
            .iter()
            .filter(|dep| dep.is_export())
            // dw PackageIndex is just a wrapper around i32 which is cloned by default anyway
            .cloned()
            .collect();
        // add the top-level actor reference
        child_indexes.insert(0, self.index());

        // get all the exports from those indexes
        let mut children: Vec<Export> = child_indexes
            .iter()
            .filter_map(|index| asset.get_export(*index))
            // i'm pretty sure i have to clone here so i can modify then insert data
            .cloned()
            .collect();

        let package_offset = (offset + 1) as i32;
        // update export references to what they will be once added
        for (i, child_index) in child_indexes.into_iter().enumerate() {
            for child in children.iter_mut() {
                on_export_refs(child, |index| {
                    if index == &child_index {
                        index.index = package_offset + i as i32;
                    }
                });
            }
        }
        children
    }
}

/// gets all actor exports within a map (all exports direct children of PersistentLevel)
pub fn get_actors(asset: &Asset<File>) -> Vec<PackageIndex> {
    match asset
        .exports
        .iter()
        .find_map(|ex| cast!(Export, LevelExport, ex))
    {
        Some(level) => level
            .actors
            .iter()
            .filter(|index| index.is_export())
            .copied()
            .collect(),
        None => Vec::new(),
    }
}

/// creates and assigns a unique name
fn give_unique_name(orig: &mut FName, asset: &mut Asset<File>) {
    // for the cases where the number is unnecessary
    if asset.search_name_reference(&orig.content).is_none() {
        *orig = asset.add_fname(&orig.content);
        return;
    }
    let mut name = orig.content.clone();
    let mut id: u16 = match name.rfind(|ch: char| ch.to_digit(10).is_none()) {
        Some(index) if index != name.len() - 1 => {
            name.drain(index + 1..).collect::<String>().parse().unwrap()
        }
        _ => 1,
    };
    while asset
        .search_name_reference(&format!("{}{}", &name, id))
        .is_some()
    {
        id += 1;
    }
    *orig = asset.add_fname(&(name + &id.to_string()))
}

/// on all possible export references
fn on_export_refs(export: &mut Export, mut func: impl FnMut(&mut PackageIndex)) {
    if let Some(norm) = export.get_normal_export_mut() {
        for prop in norm.properties.iter_mut() {
            update_props(prop, &mut func);
        }
    }
    let export = export.get_base_export_mut();
    export
        .create_before_create_dependencies
        .iter_mut()
        .for_each(&mut func);
    export
        .create_before_serialization_dependencies
        .iter_mut()
        .for_each(&mut func);
    export
        .serialization_before_create_dependencies
        .iter_mut()
        .for_each(&mut func);
    func(&mut export.outer_index);
}

/// on any possible references stashed away in properties
fn update_props(prop: &mut Property, func: &mut impl FnMut(&mut PackageIndex)) {
    match prop {
        Property::ObjectProperty(obj) => {
            func(&mut obj.value);
        }
        Property::ArrayProperty(arr) => {
            for entry in arr.value.iter_mut() {
                update_props(entry, func);
            }
        }
        Property::MapProperty(map) => {
            for val in map.value.values_mut() {
                update_props(val, func);
            }
        }
        Property::SetProperty(set) => {
            for entry in set.value.value.iter_mut() {
                update_props(entry, func);
            }
            for entry in set.removed_items.value.iter_mut() {
                update_props(entry, func);
            }
        }
        Property::DelegateProperty(del) => func(&mut del.value.object),
        Property::MulticastDelegateProperty(del) => {
            for delegate in del.value.iter_mut() {
                func(&mut delegate.object)
            }
        }
        Property::MulticastSparseDelegateProperty(del) => {
            for delegate in del.value.iter_mut() {
                func(&mut delegate.object)
            }
        }
        Property::MulticastInlineDelegateProperty(del) => {
            for delegate in del.value.iter_mut() {
                func(&mut delegate.object)
            }
        }
        Property::StructProperty(struc) => {
            for entry in struc.value.iter_mut() {
                update_props(entry, func);
            }
        }
        _ => (),
    }
}

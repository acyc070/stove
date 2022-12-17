use unreal_asset::{
    cast,
    exports::{Export, ExportBaseTrait},
    Asset,
};

impl super::Actor {
    /// delete an actor from a map
    pub fn delete(&self, map: &mut Asset) {
        let val = self.export as i32 + 1;
        if let Some(level) = map
            .exports
            .iter_mut()
            .find_map(|ex| cast!(Export, LevelExport, ex))
        {
            level
                .index_data
                .remove(level.index_data.iter().position(|&i| i == val).unwrap());
            let pos = level
                .get_base_export()
                .create_before_serialization_dependencies
                .iter()
                .position(|&i| i.index == val)
                .unwrap();
            level
                .get_base_export_mut()
                .create_before_serialization_dependencies
                .remove(pos);
        }
    }
}

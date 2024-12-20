use super::*;

impl Actor {
    /// delete an actor from a map
    pub fn delete(&self, map: &mut Asset) {
        let val = self.index();
        if let Some(level) = map
            .asset_data
            .exports
            .iter_mut()
            .find_map(|ex| cast!(Export, LevelExport, ex))
        {
            level
                .actors
                .remove(level.actors.iter().position(|i| i == &val).unwrap());
            if let Some(pos) = level
                .get_base_export()
                .create_before_serialization_dependencies
                .iter()
                .position(|i| i == &val)
            {
                level
                    .get_base_export_mut()
                    .create_before_serialization_dependencies
                    .remove(pos);
            }
        }
    }
}

use unreal_asset::{cast, exports::ExportNormalTrait, properties::Property, Asset};

impl super::Actor {
    pub fn location(&self, map: &Asset) -> glam::Vec3 {
        map.exports[self.transform]
            .get_normal_export()
            .map(|norm| {
                norm.properties
                    .iter()
                    .rev()
                    .find_map(|prop| {
                        if let Property::StructProperty(struc) = prop {
                            if &struc.name.content == "RelativeLocation" {
                                return cast!(Property, VectorProperty, &struc.value[0]);
                            }
                        }
                        None
                    })
                    .map(|pos| glam::vec3(pos.value.x.0, pos.value.z.0, pos.value.y.0) * 0.01)
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    pub fn rotation(&self, map: &Asset) -> glam::Vec3 {
        map.exports[self.transform]
            .get_normal_export()
            .map(|norm| {
                norm.properties
                    .iter()
                    .rev()
                    .find_map(|prop| {
                        if let Property::StructProperty(struc) = prop {
                            if &struc.name.content == "RelativeRotation" {
                                return cast!(Property, RotatorProperty, &struc.value[0]);
                            }
                        }
                        None
                    })
                    .map(|rot| glam::vec3(rot.value.x.0, rot.value.z.0, rot.value.y.0))
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    pub fn scale(&self, map: &Asset) -> glam::Vec3 {
        map.exports[self.transform]
            .get_normal_export()
            .map(|norm| {
                norm.properties
                    .iter()
                    .rev()
                    .find_map(|prop| {
                        if let Property::StructProperty(struc) = prop {
                            if &struc.name.content == "RelativeScale3D" {
                                return cast!(Property, VectorProperty, &struc.value[0]);
                            }
                        }
                        None
                    })
                    .map(|rot| glam::vec3(rot.value.x.0, rot.value.z.0, rot.value.y.0))
                    .unwrap_or(glam::Vec3::ONE)
            })
            .unwrap_or(glam::Vec3::ONE)
    }

    pub fn model_matrix(&self, map: &Asset) -> glam::Mat4 {
        let rot = self.rotation(map);
        glam::Mat4::from_scale_rotation_translation(
            self.scale(map),
            glam::Quat::from_euler(
                glam::EulerRot::XYZ,
                rot.x.to_radians(),
                rot.y.to_radians(),
                rot.z.to_radians(),
            ),
            self.location(map),
        )
    }
}

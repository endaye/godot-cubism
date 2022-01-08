use cubism::{
    core::model::{Drawable, Parameter, Part},
    json::model::{
        Expression, FileReferences, Group, GroupTarget, HitArea, Layout, Model3, Motion, Motions,
    },
    model::UserModel,
};
use gdnative::prelude::*;
use std::{fs::File, path::PathBuf};

use crate::dict_helpers::*;

#[derive(NativeClass, Copy, Clone, Default)]
#[user_data(MutexData<CubismLoaderFactory>)]
#[inherit(Reference)]
pub struct CubismLoaderFactory;

#[methods]
impl CubismLoaderFactory {
    fn new(_owner: &Reference) -> Self {
        Self
    }

    #[export]
    pub fn cubism_loader(
        &self,
        _owner: &Reference,
        path: String,
    ) -> Instance<CubismLoader, Unique> {
        let res_path = PathBuf::from(path);

        let json = Model3::from_reader(File::open(&res_path).expect("Unable to open file"))
            .expect("Unable to read model3 json");

        let model = UserModel::from_model3_json(&res_path).expect("Unable to load model");

        CubismLoader {
            model,
            json,
            res_path,
        }
        .emplace()
    }
}

// #[derive(NativeClass)]
// #[inherit(Reference)]
// #[user_data(user_data::MutexData<CubismLoader>)]
// pub struct CubismLoader {
//     model: Option<UserModel>,
//     json: Option<Model3>,
//     res_path: Option<PathBuf>, // This might be a relative path?
// }
#[derive(NativeClass)]
#[inherit(Reference)]
#[no_constructor]
#[user_data(user_data::MutexData<CubismLoader>)]
pub struct CubismLoader {
    model: UserModel,
    json: Model3,
    res_path: PathBuf, // This might be a relative path?
}

unsafe impl Sync for CubismLoader {}
unsafe impl Send for CubismLoader {}

#[methods]
impl CubismLoader {
    #[export]
    pub fn json(&self, _owner: &Reference) -> Dictionary {
        let d = Dictionary::new();

        let json = &self.json;

        d.insert("version", json.version as i32);
        d.insert("file_references", {
            let data = &json.file_references;
            let d = Dictionary::new();

            d.insert(
                "moc",
                data.moc
                    .as_ref()
                    .unwrap_or(&PathBuf::from("invalid"))
                    .to_str()
                    .unwrap_or("invalid"),
            );
            d.insert::<_, Vec<&str>>(
                "textures",
                data.textures
                    .iter()
                    .map(|x| x.to_str().unwrap_or("invalid"))
                    .collect(),
            );
            d.insert(
                "pose",
                data.pose
                    .as_ref()
                    .unwrap_or(&PathBuf::from("invalid"))
                    .to_str()
                    .unwrap_or("invalid"),
            );
            d.insert(
                "physics",
                data.physics
                    .as_ref()
                    .unwrap_or(&PathBuf::from("invalid"))
                    .to_str()
                    .unwrap_or("invalid"),
            );
            d.insert::<_, Vec<Dictionary>>(
                "expressions",
                data.expressions
                    .iter()
                    .map(|x| {
                        let d = Dictionary::new();

                        d.insert("name", x.name.clone());
                        d.insert("file", x.file.to_str().unwrap_or("invalid"));

                        d.into_shared()
                    })
                    .collect(),
            );
            d.insert("motions", {
                let data = &data.motions;
                let d = Dictionary::new();

                d.insert::<_, Vec<Dictionary>>(
                    "idle",
                    data.idle
                        .iter()
                        .map(|x| create_dict_from_motion(&x))
                        .collect(),
                );
                d.insert::<_, Vec<Dictionary>>(
                    "tap_body",
                    data.tap_body
                        .iter()
                        .map(|x| create_dict_from_motion(&x))
                        .collect(),
                );
                d.insert::<_, Vec<Dictionary>>(
                    "pinch_in",
                    data.pinch_in
                        .iter()
                        .map(|x| create_dict_from_motion(&x))
                        .collect(),
                );
                d.insert::<_, Vec<Dictionary>>(
                    "pinch_out",
                    data.pinch_out
                        .iter()
                        .map(|x| create_dict_from_motion(&x))
                        .collect(),
                );
                d.insert::<_, Vec<Dictionary>>(
                    "shake",
                    data.shake
                        .iter()
                        .map(|x| create_dict_from_motion(&x))
                        .collect(),
                );
                d.insert::<_, Vec<Dictionary>>(
                    "flick_head",
                    data.flick_head
                        .iter()
                        .map(|x| create_dict_from_motion(&x))
                        .collect(),
                );

                d.into_shared()
            });
            d.insert(
                "user_data",
                data.user_data
                    .as_ref()
                    .unwrap_or(&PathBuf::from("invalid"))
                    .to_str()
                    .unwrap_or("invalid"),
            );

            d.into_shared()
        });
        d.insert::<_, Vec<Dictionary>>(
            "groups",
            json.groups
                .iter()
                .map(|x| {
                    let d = Dictionary::new();

                    d.insert(
                        "target",
                        match x.target {
                            GroupTarget::Parameter => "Parameter",
                            GroupTarget::Part => "Part",
                        },
                    );
                    d.insert("name", x.name.clone());
                    d.insert::<_, Vec<String>>("ids", x.ids.iter().map(|id| id.clone()).collect());

                    d.into_shared()
                })
                .collect(),
        );
        d.insert::<_, Vec<Dictionary>>(
            "hit_area",
            json.hit_areas
                .iter()
                .map(|ha| {
                    let d = Dictionary::new();

                    d.insert("name", ha.name.clone());
                    d.insert("id", ha.id.clone());

                    d.into_shared()
                })
                .collect(),
        );
        if let Some(data) = json.layout {
            d.insert("layout", {
                let d = Dictionary::new();

                d.insert("center_x", data.center_x);
                d.insert("center_y", data.center_y);
                d.insert("x", data.x);
                d.insert("y", data.y);
                d.insert("width", data.width);
                d.insert("height", data.height);

                d.into_shared()
            })
        }

        d.into_shared()
    }

    #[export]
    pub fn res_path(&self, _owner: &Reference) -> &str {
        self.res_path.to_str().unwrap_or("invalid")
    }

    #[export]
    pub fn moc(&self, _owner: &Reference) -> Dictionary {
        let d = Dictionary::new();

        let model = self.model.model();
        let moc = model.moc();

        d.insert::<_, Vec<&str>>(
            "parameter_ids",
            moc.parameter_ids().iter().map(|x| x.clone()).collect(),
        );
        d.insert::<_, Vec<f32>>("parameter_max", moc.parameter_max().to_vec());
        d.insert::<_, Vec<f32>>("parameter_min", moc.parameter_min().to_vec());
        d.insert::<_, Vec<f32>>("parameter_default", moc.parameter_default().to_vec());
        d.insert("parameter_count", moc.parameter_count() as i32);

        d.insert::<_, Vec<&str>>(
            "part_ids",
            moc.part_ids().iter().map(|x| x.clone()).collect(),
        );
        d.insert("part_count", moc.part_count() as i32);

        d.insert::<_, Vec<&str>>(
            "drawable_ids",
            moc.drawable_ids().iter().map(|x| x.clone()).collect(),
        );
        d.insert::<_, Vec<Vec<i32>>>(
            "drawable_indices",
            moc.drawable_indices()
                .iter()
                .map(|x| x.iter().map(|y| *y as i32).collect())
                .collect(),
        );
        d.insert::<_, Vec<Vec<i32>>>(
            "drawable_masks",
            moc.drawable_masks()
                .iter()
                .map(|x| x.iter().map(|y| *y as i32).collect())
                .collect(),
        );
        d.insert("is_masked", moc.is_masked());
        d.insert::<_, Vec<i32>>(
            "drawable_texture_indices",
            moc.drawable_texture_indices().to_vec(),
        );
        d.insert::<_, Vec<i32>>(
            "drawable_vertex_counts",
            moc.drawable_vertex_counts().to_vec(),
        );
        d.insert("drawable_count", moc.drawable_count() as i32);

        d.into_shared()
    }

    // Parameters
    //#region

    #[export]
    pub fn parameter(&self, _owner: &Reference, param_name: String) -> Dictionary {
        match self.model.parameter(&param_name) {
            Some(parameter) => create_dict_from_parameter(&parameter),
            None => Dictionary::new_shared(),
        }
    }

    #[export]
    pub fn parameters(&self, _owner: &Reference) -> VariantArray {
        let a = VariantArray::new();

        for p in self.model.parameters() {
            a.push(create_dict_from_parameter(&p));
        }

        a.into_shared()
    }

    //#endregion

    // Parts
    //#region

    #[export]
    pub fn part(&self, _owner: &Reference, part_name: String) -> Dictionary {
        match self.model.part(&part_name) {
            Some(part) => create_dict_from_part(&part),
            None => Dictionary::new_shared(),
        }
    }

    #[export]
    pub fn parts(&self, _owner: &Reference) -> VariantArray {
        let a = VariantArray::new();

        for part in self.model.parts() {
            a.push(create_dict_from_part(&part));
        }

        a.into_shared()
    }

    //#endregion

    // Drawables
    //#region

    #[export]
    pub fn drawable(&self, _owner: &Reference, drawable_name: String) -> Dictionary {
        match self.model.drawable(&drawable_name) {
            Some(drawable) => create_dict_from_drawable(&drawable),
            None => Dictionary::new_shared(),
        }
    }

    #[export]
    pub fn drawables(&self, _owner: &Reference) -> VariantArray {
        let a = VariantArray::new();

        for drawable in self.model.drawables() {
            a.push(create_dict_from_drawable(&drawable));
        }

        a.into_shared()
    }

    //#endregion

    #[export]
    pub fn update(&mut self, _owner: &Reference, delta: f32) {
        self.model.update(delta);
    }
}

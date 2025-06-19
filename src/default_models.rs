use std::sync::LazyLock;

use crate::state::AnimModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultModels {
    Human,
    Calibration,
    Calibration2,
}
impl DefaultModels {
    pub const ALL: [DefaultModels; 3] = [DefaultModels::Human, DefaultModels::Calibration, DefaultModels::Calibration2];
    pub fn get(&self) -> AnimModel {
        match self {
            DefaultModels::Human => HUMAN.clone(),
            DefaultModels::Calibration => CALIBRATION.clone(),
            DefaultModels::Calibration2 => CALIBRATION2.clone(),
        }
    }
}
impl ToString for DefaultModels {
    fn to_string(&self) -> String {
        match self {
            DefaultModels::Human => "Human".into(),
            DefaultModels::Calibration => "Calibration".into(),
            DefaultModels::Calibration2 => "Calibration2".into(),
        }
    }
}

pub static HUMAN: LazyLock<AnimModel> = LazyLock::new(|| {
    let human_str = include_str!("../assets/models/Human/HumanBody.gltf");
    let human_gltf = gltf::json::Root::from_str(human_str).expect("Failed to load model");
    let human_bin = include_bytes!("../assets/models/Human/HumanBody.bin");
    AnimModel {
        gltf: human_gltf,
        bins: vec![("HumanBody.bin".into(), human_bin.to_vec())],
    }
});

pub static CALIBRATION: LazyLock<AnimModel> = LazyLock::new(|| {
    let human_str = include_str!("../assets/models/Calibration/Calibration.gltf");
    let human_gltf = gltf::json::Root::from_str(human_str).expect("Failed to load model");
    let human_bin = include_bytes!("../assets/models/Calibration/Calibration.bin");
    AnimModel {
        gltf: human_gltf,
        bins: vec![("Calibration.bin".into(), human_bin.to_vec())],
    }
});

pub static CALIBRATION2: LazyLock<AnimModel> = LazyLock::new(|| {
    let human_str = include_str!("../assets/models/Calibration2/Calibration2.gltf");
    let human_gltf = gltf::json::Root::from_str(human_str).expect("Failed to load model");
    let human_bin = include_bytes!("../assets/models/Calibration2/Calibration2.bin");
    AnimModel {
        gltf: human_gltf,
        bins: vec![("Calibration2.bin".into(), human_bin.to_vec())],
    }
});

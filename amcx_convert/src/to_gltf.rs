use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use ahrs::Ahrs;
use amcx_core::{Model, Sensor};
use byteorder::{LittleEndian, WriteBytesExt};
use gltf::{
    animation::{Interpolation, Property},
    json::{
        accessor::{ComponentType, GenericComponentType}, animation::{Channel, Sampler, Target}, buffer::{self, View}, scene, validation::Checked, Accessor, Animation, Buffer, Index, Root, Value
    },
};
use nalgebra::{Quaternion, Unit, UnitQuaternion, Vector3};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertingError {}

pub type Joint = String;
pub struct Instruction<'a> {
    pub animation_name: Option<String>,
    pub skin_name: Option<String>,
    pub mapping: Option<Vec<(Sensor, Joint)>>,
    pub save_path: &'a Path,
    pub gltf_save_name: String,
    pub bin_save_name: String,
}

pub fn convert(
    mut gltf_model: Root,
    amcx_model: &Model,
    instruction: Instruction,
) -> Result<(), ConvertingError> {
    let Instruction {
        animation_name,
        skin_name,
        mapping,
        save_path,
        gltf_save_name,
        bin_save_name,
    } = instruction;

    let nodes = match &skin_name {
        Some(name) => gltf_model.skins.iter().find(|skin| {
            skin.name
                .as_ref()
                .is_some_and(|skin_name| skin_name == name)
        }),
        None => gltf_model.skins.iter().next(),
    }
    .unwrap()
    .joints
    .iter()
    .map(|index| {
        (
            gltf_model.get(index.clone()).unwrap().name.clone().unwrap(),
            index.clone(),
        )
    });

    let hash_map: HashMap<Joint, Index<gltf::json::Node>> = HashMap::from_iter(nodes);
    let map: Vec<_> = if let Some(mapping) = mapping {
        amcx_model
            .iter()
            .map(|(sensor, stream)| {
                let node_name = mapping
                    .iter()
                    .find_map(|(sensor_to, node_name)| (sensor == sensor_to).then_some(node_name))
                    .unwrap();
                let index = hash_map.get(node_name).unwrap();
                (index, stream)
            })
            .collect()
    } else {
        amcx_model
            .iter()
            .map(|(name, stream)| (hash_map.get(name).unwrap(), stream))
            .collect()
    };

    let mut buf = Vec::new();
    let count = amcx_model.first().unwrap().1.len();

    let mut timestamps = amcx_model
        .iter()
        .next()
        .unwrap()
        .1
        .iter()
        .map(|record| record.timestamp.as_secs_f32());

    let buffer = gltf_model.push(Buffer {
        byte_length: 0u64.into(), // calculated later
        name: None,
        uri: Some(bin_save_name.clone().into()),
        extensions: None,
        extras: Default::default(),
    });

    for timestamp in timestamps.clone() {
        buf.write_f32::<LittleEndian>(timestamp).unwrap();
    }
    let min_time = timestamps.next().unwrap();
    let max_time = timestamps.last().unwrap();

    let view = View {
        name: Some("timestamps".into()),
        buffer,
        byte_length: buf.len().into(),
        byte_offset: None,
        target: None,
        byte_stride: None,
        extensions: None,
        extras: Default::default(),
    };
    let accessor = Accessor {
        buffer_view: Some(gltf_model.push(view)),
        byte_offset: None,
        count: dbg!(count.into()),
        component_type: Checked::Valid(GenericComponentType(ComponentType::F32)),
        type_: Checked::Valid(gltf::json::accessor::Type::Scalar),
        min: Some([min_time].into()),
        max: Some([max_time].into()),
        name: None,
        normalized: false,
        sparse: None,
        extensions: None,
        extras: Default::default(),
    };
    let input = gltf_model.push(accessor);

    let view_begin = buf.len();
    let view = gltf_model.push(View {
        name: Some("rotations".into()),
        buffer,
        byte_length: 0u64.into(), // calculated later
        byte_offset: Some(view_begin.into()),
        target: None,
        byte_stride: None,
        extensions: None,
        extras: Default::default(),
    });

    let mut outputs = Vec::new();
    for (index, stream) in map {
        let begin = buf.len() - view_begin;
        let initial_rotation = match gltf_model.get(index.clone()).unwrap().rotation.as_ref() {
            Some(scene::UnitQuaternion([x, y, z, w])) => {
                UnitQuaternion::new_unchecked(Quaternion::from([*x as f64, *y as f64, *z as f64, *w as f64]))
            },
            None => UnitQuaternion::identity(),
        } ;
        //let initial_quaternion = 
        let mut ahrs: ahrs::Madgwick<f64> = ahrs::Madgwick::new_with_quat(0.01, 0.1, initial_rotation);
        for record in stream {
            let gyro = Vector3::from_iterator(record.sample.gyr.iter().map(|f| *f as f64));
            let accel = Vector3::from_iterator(record.sample.acc.iter().map(|f| *f as f64));
            let q_ahrs = ahrs.update_imu(&gyro, &accel).unwrap().quaternion();

            //let q_rot = Quaternion::new(0.7071, 0.7071, 0.0, 0.0); // 90° X-rotation
            // Compose rotations: gltf = adjustment * ahrs
            let q_gltf = /*q_rot */ q_ahrs;
            // Then: Reorder components to [x, y, z, w]
            [q_gltf.i, q_gltf.j, q_gltf.k, q_gltf.w]
                .iter()
                .for_each(|c| {
                    buf.write_f32::<LittleEndian>(*c as f32).unwrap();
                });
        }
        let accessor = Accessor {
            buffer_view: Some(view),
            byte_offset: Some(begin.into()),
            count: count.into(),
            component_type: Checked::Valid(GenericComponentType(ComponentType::F32)),
            type_: Checked::Valid(gltf::json::accessor::Type::Vec4),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
            extensions: None,
            extras: Default::default(),
        };
        let output = gltf_model.push(accessor);
        outputs.push((index.clone(), output));
    }

    gltf_model.buffer_views.get_mut(view.value()).map(|view| {
        view.byte_length = (buf.len() - view.byte_offset.unwrap().0 as usize).into();
    });
    gltf_model.buffers.get_mut(buffer.value()).map(|buffer| {
        buffer.byte_length = buf.len().into();
    });

    let mut samplers = Vec::new();
    let mut channels = Vec::new();

    for (node, output) in outputs {
        let sampler = Index::push(
            &mut samplers,
            Sampler {
                input,
                interpolation: Checked::Valid(Interpolation::Linear),
                output,
                extensions: None,
                extras: Default::default(),
            },
        );
        channels.push(Channel {
            sampler,
            target: Target {
                node,
                path: Checked::Valid(Property::Rotation),
                extensions: None,
                extras: Default::default(),
            },
            extensions: None,
            extras: Default::default(),
        });
    }

    let animation = Animation {
        name: animation_name,
        channels,
        samplers,
        extensions: None,
        extras: Default::default(),
    };

    gltf_model.push(animation);

    let mut gltf_save_path = save_path.to_owned();
    gltf_save_path.push(gltf_save_name);
    let gltf_save = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(gltf_save_path)
        .unwrap();
    gltf_model.to_writer_pretty(gltf_save).unwrap();
    let mut bin_save_path = save_path.to_owned();
    bin_save_path.push(bin_save_name);
    let mut bin_save = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(bin_save_path)
        .unwrap();
    bin_save.write_all(&buf).unwrap();
    Ok(())
}

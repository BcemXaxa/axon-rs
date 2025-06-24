use std::collections::{HashMap, HashSet};

use ahrs::{Ahrs, AhrsError};
use amcx_core::{Model, Record, Sample};
use byteorder::{LittleEndian, WriteBytesExt};
use gltf::{
    animation::{Interpolation, Property},
    json::{
        Accessor, Animation, Buffer, Index, Node, Root,
        accessor::{ComponentType, GenericComponentType},
        animation::{Channel, Sampler, Target},
        buffer::View,
        scene,
        validation::Checked,
    },
};
use nalgebra::{Matrix3, Rotation, Rotation3, Unit, UnitQuaternion, Vector3};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertingError {
    #[error("Sensor {0} is not coupled with any joint")]
    SensorNotCoupled(String),
    #[error("AhrsError")]
    AhrsError(AhrsError),
    #[error("Unrecoverable sensor data")]
    UnrecoverableSensorData,
}

pub type Joint = String;

pub fn convert(
    mut gltf_model: Root,
    bin_name: &str,
    amcx_model: &Model,
    calibration: Option<&Model>,
) -> Result<(Root, Vec<u8>), ConvertingError> {
    let mut bin = Vec::new();
    let count = amcx_model.first().unwrap().1.len();

    let mut timestamps = amcx_model[0]
        .1
        .iter()
        .map(|record| record.timestamp.as_secs_f32());

    let buffer = gltf_model.push(Buffer {
        byte_length: 0u64.into(), // calculated later
        name: None,
        uri: Some(bin_name.into()),
        extensions: None,
        extras: Default::default(),
    });

    for timestamp in timestamps.clone() {
        bin.write_f32::<LittleEndian>(timestamp).unwrap();
    }
    let min_time = timestamps.next().unwrap();
    let max_time = timestamps.last().unwrap();

    let view = View {
        name: Some("timestamps".into()),
        buffer,
        byte_length: bin.len().into(),
        byte_offset: None,
        target: None,
        byte_stride: None,
        extensions: None,
        extras: Default::default(),
    };
    let accessor = Accessor {
        buffer_view: Some(gltf_model.push(view)),
        byte_offset: None,
        count: count.into(),
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

    let view_begin = bin.len();
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

    let rotations = calculate_rotations(amcx_model, &gltf_model, calibration)?;
    let mut outputs = Vec::new();
    for (index, stream) in rotations {
        if stream.is_empty() {
            continue;
        }

        let begin = bin.len() - view_begin;
        stream.into_iter().for_each(|q| {
            q.coords.iter().for_each(|c| {
                bin.write_f32::<LittleEndian>(*c).unwrap();
            })
        });

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
        view.byte_length = (bin.len() - view.byte_offset.unwrap().0 as usize).into();
    });
    gltf_model.buffers.get_mut(buffer.value()).map(|buffer| {
        buffer.byte_length = bin.len().into();
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
        channels,
        samplers,
        name: None,
        extensions: None,
        extras: Default::default(),
    };
    gltf_model.push(animation);

    Ok((gltf_model, bin))
}

fn calculate_rotations(
    model: &Model,
    root: &Root,
    calibration: Option<&Model>,
) -> Result<HashMap<Index<Node>, Vec<UnitQuaternion<f32>>>, ConvertingError> {
    let skin = root.skins.iter().next().unwrap();
    let get_joints: HashMap<&str, Index<Node>> = skin
        .joints
        .iter()
        .filter_map(|index| {
            let name = root.get(*index).unwrap().name.as_ref();
            name.map(|name| (name.as_str(), index.to_owned()))
        })
        .collect();

    let calibrators = Calibrator::new(calibration.unwrap_or(&Vec::new()))?;
    let mut indexed_calibrators = HashMap::new();
    for (sensor, calibrator) in calibrators {
        let index = get_joints
            .get(sensor.as_str())
            .ok_or(ConvertingError::SensorNotCoupled(sensor.into()))?;
        indexed_calibrators.insert(index.clone(), calibrator);
    }

    let mut joints_with_stream = HashMap::new();
    for (sensor, stream) in model {
        let index = get_joints
            .get(sensor.as_str())
            .ok_or(ConvertingError::SensorNotCoupled(sensor.into()))?;
        joints_with_stream.insert(index.clone(), stream);
    }

    let sample_count = model[0].1.len();

    let mut static_orientation = HashMap::new();
    let mut joint_rotations = HashMap::new();
    for index in skin.joints.iter().cloned() {
        match root.get(index).unwrap().rotation {
            Some(s) => {
                static_orientation.insert(index, UnitQuaternion::from_quaternion(s.0.into()));
            }
            None => {
                static_orientation.insert(index, UnitQuaternion::identity());
            }
        }

        let mut rotations = match joints_with_stream.remove(&index) {
            Some(stream) => process_ahrs(stream)?,
            None => vec![UnitQuaternion::identity(); sample_count],
        };
        if let Some(calibrator) = indexed_calibrators.get(&index) {
            rotations
                .iter_mut()
                .for_each(|q| *q = calibrator.calibrate(*q));
        }
        joint_rotations.insert(index, rotations);
    }

    let tree = NodeTree::new(root);
    for tree_root in tree {
        tree_root.for_each_bf(&mut |node| {
            let s = static_orientation.get(&node.index).unwrap().clone();
            for child in &node.children {
                let child_orientation = static_orientation.get_mut(&child.index).unwrap();
                *child_orientation = s * (*child_orientation);
            }
            joint_rotations
                .get_mut(&node.index)
                .unwrap()
                .iter_mut()
                .for_each(|q| {
                    *q = (*q) * s;
                });
        });
        tree_root.for_each_df(&mut |node| {
            let parent_rotations_inv: Vec<_> = joint_rotations
                .get(&node.index)
                .unwrap()
                .into_iter()
                .map(|q| q.inverse())
                .collect();
            let children = node.children.iter().map(|node| &node.index);
            for child in children {
                let zip = joint_rotations
                    .get_mut(child)
                    .unwrap()
                    .iter_mut()
                    .zip(parent_rotations_inv.iter());
                for (q, r_inv) in zip {
                    *q = r_inv * (*q);
                }
            }
        });
    }

    Ok(joint_rotations)
}

fn process_ahrs(stream: &[Record]) -> Result<Vec<UnitQuaternion<f32>>, ConvertingError> {
    let sample_count = stream.len();
    let total_time = stream.last().map_or(0.0, |r| r.timestamp.as_secs_f32());
    let avg_delta = total_time / sample_count as f32;

    let mut rotations = Vec::with_capacity(sample_count);
    let mut ahrs = ahrs::Madgwick::new(avg_delta, 0.0);

    let mut previous = None;
    for mut record in stream {
        if record.sample.acc == [0.0, 0.0, 0.0] {
            record = previous.ok_or(ConvertingError::UnrecoverableSensorData)?;
        } else {
            previous = Some(record);
        }

        let gyro = record.sample.gyr.clone().into();
        let accel = record.sample.acc.clone().into();
        let q_ahrs = ahrs
            .update_imu(&gyro, &accel)
            .map_err(ConvertingError::AhrsError)?;
        rotations.push(*q_ahrs);
    }
    Ok(rotations)
}

struct NodeTree {
    index: Index<Node>,
    children: Vec<NodeTree>,
}
impl NodeTree {
    fn new(root: &Root) -> Vec<NodeTree> {
        let skin = root.skins.iter().next().unwrap();
        let all_nodes = skin.joints.clone();

        let mut with_children: HashMap<Index<Node>, Vec<Index<Node>>> = all_nodes
            .iter()
            .cloned()
            .map(|index| {
                let children = root
                    .get(index)
                    .unwrap()
                    .children
                    .clone()
                    .unwrap_or_default();
                (index, children)
            })
            .collect();

        let children: HashSet<_> = with_children
            .values()
            .flat_map(|children| children.iter())
            .collect();
        let mut roots = Vec::new();
        for node in all_nodes {
            if !children.contains(&node) {
                roots.push(node);
            }
        }

        let result = roots
            .into_iter()
            .map(|root| NodeTree::build_tree(root, &mut with_children))
            .collect();
        result
    }

    fn build_tree(
        root: Index<Node>,
        with_children: &mut HashMap<Index<Node>, Vec<Index<Node>>>,
    ) -> NodeTree {
        let children = with_children
            .remove(&root)
            .unwrap_or_default()
            .into_iter()
            .map(|index| NodeTree::build_tree(index, with_children))
            .collect();

        NodeTree {
            index: root,
            children,
        }
    }

    fn for_each_df<F>(&self, f: &mut F)
    where
        F: FnMut(&Self),
    {
        for child in &self.children {
            child.for_each_df(f);
        }
        f(self);
    }

    fn for_each_bf<F>(&self, f: &mut F)
    where
        F: FnMut(&Self),
    {
        f(self);
        for child in &self.children {
            child.for_each_bf(f);
        }
    }
}

struct Calibrator {
    q_local_to_global: UnitQuaternion<f32>,
}

impl Calibrator {
    fn new(reference_model: &Model) -> Result<HashMap<String, Calibrator>, ConvertingError> {
        let mut calibrators = HashMap::new();

        for (sensor, stream) in reference_model {
            // get "down"
            let stationary_time = 0.5;
            let approx_y: Vector3<f32> = stream
                .iter()
                .map_while(|r| {
                    (r.timestamp.as_secs_f32() < stationary_time)
                        .then_some(Vector3::from(r.sample.acc))
                })
                .sum();
            let approx_y = -approx_y.normalize();
            // get rotation -> axis
            let new_z = process_ahrs(stream)?
                .last()
                .unwrap()
                .axis()
                .unwrap()
                .into_inner(); // already normalized
            // crossproduct third axis
            let new_x = new_z.cross(&approx_y).normalize();
            // orthogonalize and normalize
            let new_y = new_z.cross(&new_x).normalize();

            // convert to quaternion
            let rotation_matrix =
                Rotation3::from_matrix(&Matrix3::from_columns(&[new_x, new_y, new_z]));
            let quat = UnitQuaternion::from_rotation_matrix(&rotation_matrix);

            calibrators.insert(
                sensor.into(),
                Calibrator {
                    q_local_to_global: quat,
                },
            );
        }
        Ok(calibrators)
    }

    fn calibrate(&self, q: UnitQuaternion<f32>) -> UnitQuaternion<f32> {
        self.q_local_to_global.inverse() * q * self.q_local_to_global
    }
}

struct Transition {
    dt: f32,
    gyro: [f32; 3],
    acc: [f32; 3],
}
#[derive(Default)]
struct Settings {
    gyro_fs: GyroFS,
    acc_fs: AccFS,
}
impl Settings {
    const BITS: u32 = 16;
    fn new(gyro_fs: GyroFS, acc_fs: AccFS) -> Self {
        Self { gyro_fs, acc_fs }
    }
}

#[derive(Default)]
enum GyroFS {
    #[default]
    Gyro250ds,
    Gyro500ds,
    Gyro1000ds,
    Gyro2000ds,
}
#[derive(Default)]
enum AccFS {
    #[default]
    Acc2g,
    Acc4g,
    Acc8g,
    Acc16g,
}

// fn agd_to_gltf() {
//     let mut root = gltf::json::Root::default();
//     root.push(value)
// }
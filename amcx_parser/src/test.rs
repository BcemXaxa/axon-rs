use crate::raw_parsing;
use approx::assert_relative_eq;

#[test]
fn config_valid() {
    todo!()
}
#[test]
fn config_invalid() {
    let invalid_block = "[ BITS=16 ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
    let invalid_duplicate = "?[ BITS=16 ACC_FS=2 GYRO_FS=250 BITS=16 CLOCK=micro ]";
    let invalid_key_value = "?[ UNKNOWN:10 BITS=16 ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
    let invalid_key = "?[ UNKNOWN=10 BITS=16 ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
    let invalid_bits = "?[ BITS=15 ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
    let invalid_acc = "?[ BITS=16 ACC_FS=3 GYRO_FS=250 CLOCK=micro ]";
    let invalid_gyro = "?[ BITS=16 ACC_FS=2 GYRO_FS=20 CLOCK=micro ]";
    let invalid_clock = "?[ BITS=16 ACC_FS=2 GYRO_FS=250 CLOCK=sec ]";
    let invalid_missing = "?[ BITS=16 GYRO_FS=250 CLOCK=micro ]";
    todo!()
}

#[test]
fn reference_valid() {
    todo!()
}

#[test]
fn reference_invalid() {
    todo!()
}

#[test]
fn sample_valid() {
    todo!()
}

#[test]
fn sample_invalid() {
    todo!()
}

#[test]
fn duration_valid() {
    todo!()
}

#[test]
fn duration_invalid() {
    todo!()
}

#[test]
fn parsing_valid() {
    todo!()
}

#[test]
fn parsing_invalid() {
    todo!()
}

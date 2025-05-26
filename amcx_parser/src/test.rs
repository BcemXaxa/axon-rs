use super::*;
use approx::assert_relative_eq;

#[test]
fn config_valid() {
    let valid_config = "?[ BITS=16 ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
    let config = parse_config(Some((0, valid_config))).unwrap();
    assert_eq!(config.bits, 16);
    assert_eq!(config.clock, Clock::Microseconds);
    assert_relative_eq!(config.acc_fs, 2.0);
    assert_relative_eq!(config.gyro_fs, 250.0);

    let valid_config_spacing = "?[BITS=16   ACC_FS=2    GYRO_FS=250 CLOCK=micro]";
    let config = parse_config(Some((0, valid_config_spacing))).unwrap();
    assert_eq!(config.bits, 16);
    assert_eq!(config.clock, Clock::Microseconds);
    assert_relative_eq!(config.acc_fs, 2.0);
    assert_relative_eq!(config.gyro_fs, 250.0);

    let valid_config_default_bits = "?[ ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
    let config = parse_config(Some((0, valid_config_default_bits))).unwrap();
    assert_eq!(config.bits, Config::BITS_DEFAULT);
    assert_eq!(config.clock, Clock::Microseconds);
    assert_relative_eq!(config.acc_fs, 2.0);
    assert_relative_eq!(config.gyro_fs, 250.0);
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

    let invalid = [
        invalid_block,
        invalid_duplicate,
        invalid_key_value,
        invalid_key,
        invalid_bits,
        invalid_acc,
        invalid_gyro,
        invalid_clock,
        invalid_missing,
    ];

    for inv in invalid {
        assert!(parse_config(Some((1, inv))).is_err())
    }
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

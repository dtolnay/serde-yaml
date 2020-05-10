use serde_yaml::{Number, Value};

#[test]
fn test_nan() {
    let pos_nan = serde_yaml::from_str::<Value>("NaN").unwrap();
    let neg_nan = serde_yaml::from_str::<Value>("-NaN").unwrap();
    assert_eq!(pos_nan, pos_nan);
    assert_eq!(neg_nan, neg_nan);
    assert_ne!(pos_nan, neg_nan);

    let significand_mask = 0xF_FFFF_FFFF_FFFF;
    let bits = (f64::NAN.to_bits() ^ significand_mask) | 1;
    let different_pos_nan = Value::Number(Number::from(f64::from_bits(bits)));
    assert_eq!(pos_nan, different_pos_nan);
}

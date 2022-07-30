#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let data = data.get(..10240).unwrap_or(data);
    let _ = serde_yaml::from_slice::<serde_yaml::Value>(data);
});

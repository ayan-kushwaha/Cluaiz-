use candle_core::Device;
use std::fs;

#[test]
fn test_mask() {
    let device = Device::new_cuda(0).unwrap_or(Device::Cpu);
    let mask = candle_transformers::utils::build_causal_mask(4, 0, &device).unwrap();
    let s = format!("Mask DType: {:?}\nMask: {}", mask.dtype(), mask);
    fs::write("tests/mask_output.txt", s).unwrap();
}

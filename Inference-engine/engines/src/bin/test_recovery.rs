use cluaize_shared::HardwareGovernor;

fn main() {
    println!("🧪 [Test] Attempting to load Cluaize Truth...");
    match HardwareGovernor::load_system_control() {
        Ok(control) => {
            println!("✅ [Test] Load Success! Cluaize Root: {}", control.context.cluaize_root);
        },
        Err(e) => {
            println!("❌ [Test] Load Failed (as expected if recovery failed): {}", e);
        }
    }
}

mod common;

use sj_emu::Core;

#[test]
fn core_loads_and_names_itself() {
    if !common::core_path().exists() {
        eprintln!("skipped: core not built, run scripts/build-core.sh");
        return;
    }
    let save_dir = std::env::temp_dir().join("sj-core-loads");
    let core = Core::load(
        &common::software_config(save_dir),
        Box::new(|_| std::ptr::null()),
    )
    .unwrap();
    assert!(core.library_name().contains("melonDS"));
}

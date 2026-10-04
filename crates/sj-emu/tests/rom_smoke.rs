mod common;

use sj_emu::{Core, Frame};
use sj_game::game_api::GameApi;

const FRAMES_TO_RUN: u32 = 600;

#[test]
#[ignore = "needs roms/sj_usa.nds and a built core"]
fn rom_boots_and_produces_video_audio_and_ram() {
    let rom_path = common::repo_root().join("roms/sj_usa.nds");
    if !rom_path.exists() || !common::core_path().exists() {
        eprintln!("skipped: ROM or core missing");
        return;
    }
    let save_dir = std::env::temp_dir().join("sj-rom-smoke");
    std::fs::create_dir_all(&save_dir).unwrap();
    let mut core = Core::load(
        &common::software_config(save_dir),
        Box::new(|_| std::ptr::null()),
    )
    .unwrap();
    let av_info = core.load_game(&rom_path).unwrap();
    assert!(av_info.fps > 59.0 && av_info.fps < 61.0);

    let mut last_frame = None;
    let mut audio_samples = 0;
    for _ in 0..FRAMES_TO_RUN {
        let output = core.run_frame();
        audio_samples += output.stereo_samples.len();
        last_frame = output.frame.or(last_frame);
    }

    let Some(Frame::Software {
        width_px,
        height_px,
        xrgb_pixels,
    }) = last_frame
    else {
        panic!("no software frame produced");
    };
    assert_eq!((width_px, height_px), (256, 384));
    assert!(
        xrgb_pixels.iter().any(|pixel| pixel & 0x00FF_FFFF != 0),
        "screen is all black"
    );
    assert!(audio_samples > 0);
    assert_eq!(core.main_ram().len(), 4 * 1024 * 1024);
    assert!(core.main_ram().iter().any(|byte| *byte != 0));

    let state = core.save_state().unwrap();
    core.run_frame();
    core.load_state(&state).unwrap();
}

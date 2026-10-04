use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::savestate::{age_text, saved_states};

// How close to the top edge, in points, the pointer has to be to bring the bar up.
const REVEAL_ZONE_PT: f32 = 28.0;

pub enum MenuAction {
    LoadState { label: String, path: PathBuf },
}

// A menu bar that stays out of the game's way: it appears while the pointer is at the top edge
// of the window, and stays while one of its menus is open.
pub struct MenuBar {
    menu_open: bool,
}

impl MenuBar {
    pub fn new() -> Self {
        Self { menu_open: false }
    }

    pub fn show(&mut self, root: &mut egui::Ui, save_dir: &Path) -> Option<MenuAction> {
        let near_top = root
            .ctx()
            .input(|input| input.pointer.hover_pos())
            .is_some_and(|pos| pos.y <= REVEAL_ZONE_PT);
        if !near_top && !self.menu_open {
            return None;
        }
        let mut action = None;
        let bar = egui::Panel::top("menu_bar").show(root, |ui| {
            egui::MenuBar::new()
                .ui(ui, |ui| {
                    ui.menu_button("Load state", |ui| action = load_state_menu(ui, save_dir))
                        .inner
                })
                .inner
        });
        self.menu_open = bar.inner.is_some();
        action
    }
}

fn load_state_menu(ui: &mut egui::Ui, save_dir: &Path) -> Option<MenuAction> {
    let states = saved_states(save_dir, SystemTime::now());
    if states.is_empty() {
        ui.label("No saved states yet");
        return None;
    }
    for state in states {
        let text = format!("{}  ·  {}", state.label, age_text(state.age));
        if ui.button(text).clicked() {
            ui.close();
            return Some(MenuAction::LoadState {
                label: state.label,
                path: state.path,
            });
        }
    }
    None
}
